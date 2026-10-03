use std::collections::{BinaryHeap, HashMap, hash_map};
use std::{cmp, hash};

use bevy::ecs::entity::Entity;
use bevy::math::{Vec2, Vec3};
use ordered_float::OrderedFloat;

use crate::vehicle;

#[derive(Default)]
pub struct Pathfinder {
    heap:    BinaryHeap<(cmp::Reverse<AstarCost>, NodeRef)>,
    nodes: HashMap<NodeRef, VisitedNode>,
}

/// Represents a step in a path where
/// the subject has just entered a building from an ajacent corridor or rail.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct NodeRef {
    building:    Entity,
    travel_mode: TravelMode,
    entrance:    NodeEntrance,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum NodeEntrance {
    /// The path started here, or a vehicle change took place here.
    Interior(InteriorNodeEntrance),
    /// The subject walked into the building from this corridor.
    Corridor(Entity),
    /// The subject drove into the building from this rail.
    Rail { rail: Entity },
}

#[derive(Debug, Clone, Copy)]
struct InteriorNodeEntrance {
    interior_pos: Vec3,
}

// InteriorNodeEntrance can only be used at the start or during vehicle switch,
// so it is impossible for multiple interior entrance nodes to
// have the same `NodeRef::building` and `NodeRef::entrance`,
impl PartialEq for InteriorNodeEntrance {
    fn eq(&self, other: &Self) -> bool { true }
}
impl Eq for InteriorNodeEntrance {}
impl PartialOrd for InteriorNodeEntrance {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> { Some(cmp::Ordering::Equal) }
}
impl Ord for InteriorNodeEntrance {
    fn cmp(&self, other: &Self) -> cmp::Ordering { cmp::Ordering::Equal }
}
impl hash::Hash for InteriorNodeEntrance {
    fn hash<H: hash::Hasher>(&self, _: &mut H) {}
}

impl NodeEntrance {
    fn corridor(self, world: &impl WorldContext) -> Option<Entity> {
        match self {
            Self::Interior(_) => None,
            Self::Corridor(corridor) => Some(corridor),
            Self::Rail { rail } => world.get_rail_corridor(rail),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TravelMode {
    Walking,
    Driving { vehicle_type: vehicle::TypeId },
}

struct VisitedNode {
    previous: PreviousNodeRef,
    cost:     TimeCost,
}

enum PreviousNodeRef {
    Start,
    Node(NodeRef),
}

#[derive(Debug, Clone, Copy, Default)]
struct AstarCost {
    time:            TimeCost,
    astar_heuristic: OrderedFloat<f32>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
struct TimeCost(OrderedFloat<f32>);

impl PartialEq for AstarCost {
    fn eq(&self, other: &Self) -> bool {
        self.time.0 + self.astar_heuristic == other.time.0 + other.astar_heuristic
    }
}

impl Eq for AstarCost {}

impl Ord for AstarCost {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (self.time.0 + self.astar_heuristic).cmp(&(other.time.0 + other.astar_heuristic))
    }
}

impl PartialOrd for AstarCost {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> { Some(self.cmp(other)) }
}

impl Pathfinder {
    pub fn add_initial_building(&mut self, building: Entity, interior_pos: Vec3, travel_mode: TravelMode) {
        self.heap.push((Default::default(), NodeRef {
            building,
            travel_mode,
            entrance: NodeEntrance::Interior(InteriorNodeEntrance { interior_pos }),
        }));
    }

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
        let mut steps = 0;
        loop {
            steps += 1;
            let last = path.last().expect("path never gets shortened");
            let visit =
                self.nodes.get(last).expect("next and previous must reference nodes");
            match visit.previous {
                PreviousNodeRef::Start => break,
                PreviousNodeRef::Node(prev) => path.push(prev),
            }
        }
        tracing::debug!("Pathfinding complete", pathfind.steps = steps, pathfind.heap_len = self.heap.len());
        Some(FoundPath { reversed: path, cost })
    }

    fn next(&mut self, world: &impl WorldContext, goal: &impl Goal) -> NextResult {
        let Some((cmp::Reverse(cost), node)) = self.heap.pop() else {
            return NextResult::NoSolution;
        };
        if goal.is_building(node.building) {
            return NextResult::Found(node, cost);
        }

        let travel_mode = node.travel_mode;

        for vehicle in world.get_parked_vehicles(node.building) {
            if travel_mode != (TravelMode::Driving { vehicle_type: vehicle.ty }) {
                self.push(
                    node,
                    NodeRef {
                        building:    node.building,
                        entrance:    NodeEntrance::Interior(InteriorNodeEntrance {
                            interior_pos: vehicle.interior_pos,
                        }),
                        travel_mode: TravelMode::Driving { vehicle_type: vehicle.ty },
                    },
                    AstarCost {
                        time:            TimeCost(
                            cost.time.0 + world.alight_cost(travel_mode) + vehicle.switch_cost,
                        ),
                        // theoretically unchanged in the same building
                        astar_heuristic: cost.astar_heuristic,
                    },
                );
            }
        }
        if goal.allows_walking() {
            self.push(
                node,
                NodeRef {
                    building:    node.building,
                    entrance:    NodeEntrance::Interior(InteriorNodeEntrance {
                        interior_pos: world.node_entrance_interior_pos(node.entrance),
                    }),
                    travel_mode: TravelMode::Walking,
                },
                AstarCost {
                    time:            TimeCost(cost.time.0 + world.alight_cost(travel_mode)),
                    // theoretically unchanged in the same building
                    astar_heuristic: cost.astar_heuristic,
                },
            );
        }

        for adj in world.get_adjacent_corridors(node.building) {
            if goal.is_corridor(adj.corridor) {
                return NextResult::Found(node, cost);
            }
            if Some(adj.corridor) == node.entrance.corridor(world) && !goal.is_rail_in(adj.corridor)
            {
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

                if let TravelMode::Driving { vehicle_type } = travel_mode {
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

            let local_cost = world.building_bypass_cost(
                travel_mode,
                node.building,
                node.entrance,
                adj.local_endpoint_interior_pos,
            );

            if let TravelMode::Driving { vehicle_type } = travel_mode {
                for member in world.get_member_rails(adj.corridor, vehicle_type) {
                    if goal.is_rail(member.rail) {
                        return NextResult::Found(node, cost);
                    }

                    let cost = AstarCost {
                        time:            TimeCost(
                            local_cost
                                + world.rail_motion_cost(member.rail, travel_mode, adj.length),
                        ),
                        astar_heuristic: goal.astar_heuristic(peer_endpoint.heuristic_pos),
                    };

                    self.push(
                        node,
                        NodeRef {
                            building: peer_endpoint.building,
                            entrance: NodeEntrance::Rail { rail: member.rail },
                            travel_mode,
                        },
                        cost,
                    )
                }
            }

            if goal.allows_walking() {
                self.push(
                    node,
                    NodeRef {
                        building:    peer_endpoint.building,
                        entrance:    NodeEntrance::Corridor(adj.corridor),
                        travel_mode: TravelMode::Walking,
                    },
                    cost,
                );
            }
        }
        NextResult::Continue
    }

    fn push(&mut self, prev: NodeRef, next: NodeRef, cost: AstarCost) {
        match self.nodes.entry(next) {
            hash_map::Entry::Vacant(entry) => {
                entry.insert(VisitedNode {
                    previous: PreviousNodeRef::Node(prev),
                    cost:     cost.time,
                });
            }
            hash_map::Entry::Occupied(mut entry) => {
                if entry.get().cost <= cost.time {
                    return;
                }
                entry.insert(VisitedNode {
                    previous: PreviousNodeRef::Node(prev),
                    cost:     cost.time,
                });
            }
        }

        self.heap.push((cmp::Reverse(cost), next));
    }
}

#[derive(Debug)]
pub struct FoundPath {
    /// Nodes in reverse order.
    pub reversed: Vec<NodeRef>,
    pub cost:     AstarCost,
}

enum NextResult {
    NoSolution,
    Found(NodeRef, AstarCost),
    Continue,
}

pub trait WorldContext {
    /// Returns the corridors adjacent to `building` that are not closed.
    fn get_adjacent_corridors(
        &self,
        building: Entity,
    ) -> impl Iterator<Item = AdjacentCorridor> + '_;

    /// Returns the rails in `corridor` that may be used by `vehicle` in this context.
    fn get_member_rails(
        &self,
        corridor: Entity,
        vehicle_type: vehicle::TypeId,
    ) -> impl Iterator<Item = MemberRail> + '_;

    /// Returns the vehicles parked in `building` that may be driven in this context.
    fn get_parked_vehicles(&self, building: Entity) -> impl Iterator<Item = ParkedVehicle> + '_;

    /// Returns the corridor owning `rail`.
    fn get_rail_corridor(&self, rail: Entity) -> Option<Entity>;

    /// Computes the cost of moving from `entrance` to `exit` inside `building`,
    /// subject to vehicle or walking speed.
    fn building_bypass_cost(
        &self,
        travel_mode: TravelMode,
        building: Entity,
        entrance: NodeEntrance,
        exit: Vec3,
    ) -> OrderedFloat<f32>;

    /// Resolves a [`NodeEntrance`] into a position inside the building.
    fn node_entrance_interior_pos(&self, entrance: NodeEntrance) -> Vec3;

    /// Computes the cost of moving along `rail` with `travel_mode` for `distance`.
    fn rail_motion_cost(
        &self,
        rail: Entity,
        travel_mode: TravelMode,
        distance: f32,
    ) -> OrderedFloat<f32>;

    /// Cost of switching the travel mode to `TravelMode::Walking`.
    fn alight_cost(&self, vehicle_ty: TravelMode) -> OrderedFloat<f32>;
}

pub struct AdjacentCorridor {
    pub corridor:                    Entity,
    pub local_endpoint_interior_pos: Vec3,
    pub peer_endpoint:               Option<AdjacentPeerEndpoint>,
    pub length:                      f32,
}

pub struct AdjacentPeerEndpoint {
    pub building:      Entity,
    /// Heuristic position of the peer building, only used for A\*.
    pub heuristic_pos: Vec2,
    pub interior_pos:  Vec3,
}

pub struct MemberRail {
    pub rail: Entity,
}

pub struct ParkedVehicle {
    pub vehicle:      Entity,
    pub ty:           vehicle::TypeId,
    pub interior_pos: Vec3,
    pub switch_cost:  OrderedFloat<f32>,
}

pub trait Goal {
    fn is_corridor(&self, corridor: Entity) -> bool;

    fn is_building(&self, building: Entity) -> bool;

    fn is_rail_in(&self, corridor: Entity) -> bool;

    fn is_rail(&self, rail: Entity) -> bool;

    fn astar_heuristic(&self, pos: Vec2) -> OrderedFloat<f32>;

    fn allows_walking(&self) -> bool;
}
