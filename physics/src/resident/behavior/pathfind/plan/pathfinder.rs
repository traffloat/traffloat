use std::cmp;
use std::collections::{BinaryHeap, HashMap};

use bevy::ecs::entity::Entity;
use bevy::math::{Vec2, Vec3};
use ordered_float::OrderedFloat;

use crate::vehicle;

pub struct Pathfinder {
    heap: BinaryHeap<(cmp::Reverse<Cost>, NodeRef)>,
    visited: HashMap<NodeRef, VisitedNode>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct NodeRef {
    building: Entity,
    entrance: NodeEntrance,
    travel_mode: TravelMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum NodeEntrance {
    Corridor(Entity),
    Rail{rail:Entity, corridor:Entity},
}

impl NodeEntrance {
    fn corridor(self) -> Entity {
        match self {
            NodeEntrance::Corridor(corridor) | NodeEntrance::Rail{corridor, ..} => corridor,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum TravelMode {
    Walking,
    Driving{vehicle_type: vehicle::TypeId}
}

struct VisitedNode {
    previous: PreviousNodeRef,
}

enum PreviousNodeRef {
    Start,
    Node(NodeRef),
}

#[derive(Debug, Clone, Copy)]
struct Cost {
    time: OrderedFloat<f32>,
    astar_heuristic: OrderedFloat<f32>,
}

impl PartialEq for Cost {
    fn eq(&self, other: &Self) -> bool {
        self.time + self.astar_heuristic == other.time + other.astar_heuristic
    }
}

impl Eq for Cost {}

impl Ord for Cost {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (self.time + self.astar_heuristic).cmp(&(other.time + other.astar_heuristic))
    }
}

impl PartialOrd for Cost {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Pathfinder {
    #[tracing::instrument(skip_all, ret)]
    pub fn find(&mut self, world: &impl WorldContext, goal: &impl Goal) -> Option<FoundPath> {
        let (node_ref, cost) = loop {
            match self.next(world, goal) {
                NextResult::NoSolution => return None,
                NextResult::Found(node_ref, cost) => {
                    break (node_ref, cost);
                }
                NextResult::Continue => {}
            }
        };

        let mut path = vec![node_ref];
        loop {
            let last = path.last().expect("path never gets shortened");
            let visit = self.visited.get(last).expect("next and previous must reference visited node");
            match visit.previous {
                PreviousNodeRef::Start => break ,
                PreviousNodeRef::Node(prev) => path.push(prev),
            }
        }
        Some(FoundPath{ reversed:path, cost })
    }

    fn next(&mut self, world: &impl WorldContext, goal: &impl Goal) -> NextResult {
        let Some((cmp::Reverse(cost), node)) = self.heap.pop() else { return NextResult::NoSolution };
        if goal.is_building(node.building) {
            return NextResult::Found(node, cost);
        }

        let travel_mode = node.travel_mode; // TODO fan out to parked vehicles

        for adj in world.get_adjacent_corridors(node.building) {
            if goal.is_corridor(adj.corridor) {
                return NextResult::Found(node, cost);
            }
            if adj.corridor == node.entrance.corridor() && !goal.is_rail_in(adj.corridor) {
                // TODO: && did not board a vehicle

                // turning back would not yield a better path.
                continue;
            }

            let Some(peer_endpoint) = adj.peer_endpoint else {
                // If the corridor has no peer building, this would be a dead end.
                // There is no reason to enter the corridor unless the goal is a rail inside.
                if !goal.is_rail_in(adj.corridor) {
                    continue;
                }

                if let TravelMode::Driving{vehicle_type} = travel_mode {
                    for member in world.get_member_rails(adj.corridor, vehicle_type) {
                        if goal.is_rail(member.rail) {
                            return NextResult::Found(node, cost);
                        }
                    }
                }

                // The corridor has no peer building, and the rail is not accessible to us.
                // This most likely means we need to switch a vehicle first.
                continue;
            };

            let local_cost = world.building_bypass_cost(travel_mode, node.building, node.entrance, adj.local_endpoint_interior_pos);

            if let TravelMode::Driving{vehicle_type} = travel_mode {
                for member in world.get_member_rails(adj.corridor, vehicle_type) {
                    if goal.is_rail(member.rail) {
                        return NextResult::Found(node, cost);
                    }

                    let cost = Cost {
                        time: local_cost + world.rail_motion_cost(member.rail, travel_mode, adj.length),
                        astar_heuristic: goal.astar_heuristic(peer_endpoint.building),
                    };

                    self.heap.push((
                            cmp::Reverse(cost),
                            NodeRef {
                                building: peer_endpoint.building,
                                entrance: NodeEntrance::Rail{rail:member.rail, corridor:adj.corridor},
                                travel_mode: travel_mode,
                            },
                    ));
                }
            }

            if goal.allows_walking() {
                self.heap.push((
                    cmp::Reverse(cost),
                    NodeRef {
                        building: peer_endpoint.building,
                        entrance: NodeEntrance::Corridor(adj.corridor),
                        travel_mode: TravelMode::Walking,
                    },
                ));
            }
        }
        NextResult::Continue
    }
}

#[derive(Debug)]
pub struct FoundPath {
    /// Nodes in reverse order.
    pub reversed: Vec<NodeRef>,
    pub cost: Cost,
}

enum NextResult {
    NoSolution,
    Found(NodeRef, Cost),
    Continue,
}

pub trait WorldContext {
    /// Returns the corridors adjacent to `building` that are not closed.
    fn get_adjacent_corridors(&self, building: Entity) -> impl Iterator<Item = AdjacentCorridor> + '_;

    /// Returns the rails in `corridor` that may be used by `vehicle` in this context.
    fn get_member_rails(&self, corridor: Entity, vehicle_type: vehicle::TypeId) -> impl Iterator<Item = MemberRail> + '_;

    /// Returns the vehicles parked in `building` that may be driven in this context.
    fn get_parked_vehicles(&self, building: Entity) -> impl Iterator<Item = ParkedVehicle> + '_;

    /// Returns the cost of moving from `entrance` to `exit` inside `building`,
    /// subject to vehicle or walking speed.
    fn building_bypass_cost(&self, travel_mode: TravelMode, building: Entity, entrance: NodeEntrance, exit: Vec3) -> OrderedFloat<f32>;

    fn rail_motion_cost(&self, rail: Entity, travel_mode: TravelMode, distance: f32) -> OrderedFloat<f32>;

    fn astar_heuristic(&self, pos: Vec2) -> OrderedFloat<f32>;
}

pub struct AdjacentCorridor {
    pub corridor: Entity,
    pub local_endpoint_interior_pos: Vec3,
    pub peer_endpoint: Option<AdjacentPeerEndpoint>,
    pub length:   f32,
}

pub struct AdjacentPeerEndpoint {
    pub building: Entity,
    /// Heuristic position of the peer building, only used for A\*.
    pub heuristic_pos: Vec2,
    pub interior_pos: Vec3,
}

pub struct MemberRail {
    pub rail: Entity,
}

pub struct ParkedVehicle {
    pub vehicle: Entity,
}

pub trait Goal{
    fn is_corridor(&self, corridor: Entity) -> bool;

    fn is_building(&self, building: Entity) -> bool;

    fn is_rail_in(&self, corridor: Entity) -> bool;

    fn is_rail(&self, rail: Entity) -> bool;

    fn astar_heuristic(&self, pos: Vec2) -> OrderedFloat<f32>;

    fn allows_walking(&self) -> bool;
}
