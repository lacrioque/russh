pub mod model;
pub mod paths;
pub mod resolve;

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
