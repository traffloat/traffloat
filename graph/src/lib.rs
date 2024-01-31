//! The connection between buildings in Traffloat is represented as a directed spatial graph,
//! where buildings are "nodes" and corridors linking them are "edges".

pub mod attribute;
pub mod duct;
pub mod edge;
pub mod node;

pub use attribute::Attribute;
pub use duct::Duct;
pub use edge::Edge;
pub use node::Node;
