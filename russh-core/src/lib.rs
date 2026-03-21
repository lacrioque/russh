pub mod config;
pub mod model;
pub mod paths;
pub mod resolve;
pub mod ssh;
pub mod validate;

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
