use nalgebra::Vector3;

dynec::archetype! {
    /// A node is a building in the game.
    pub Node;
}

/// Reference position of the node.
#[dynec::comp(of = Node, required)]
pub struct Position(pub Vector3<f64>);
