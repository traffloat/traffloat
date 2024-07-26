pub use serde;

pub mod save;

pub mod proto {
    include!(concat!(env!("OUT_DIR"), "/traffloat.base.rs"));
}
