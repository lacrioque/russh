pub mod config;
pub mod model;
pub mod paths;
pub mod resolve;
pub mod ssh;
pub mod sync;
pub mod validate;

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Test utilities shared across modules.
#[cfg(test)]
pub(crate) mod test_util {
    use std::sync::Mutex;

    /// Global mutex to serialize tests that mutate environment variables.
    /// Rust tests run in parallel by default, and `env::set_var`/`env::remove_var`
    /// affect the entire process, causing race conditions.
    pub(crate) static ENV_MUTEX: Mutex<()> = Mutex::new(());
}
