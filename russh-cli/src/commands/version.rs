use russh_core::paths::config_path;

pub fn run() {
    println!("russh {}", env!("CARGO_PKG_VERSION"));
    match config_path(None) {
        Some(p) => println!("config: {}", p.display()),
        None => println!("config: (could not determine path)"),
    }
}
