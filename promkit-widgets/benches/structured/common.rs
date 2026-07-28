use std::{
    env,
    path::{Path, PathBuf},
};

pub const VIEWPORT_WIDTH: u16 = 120;
pub const VIEWPORT_HEIGHT: u16 = 40;

pub fn fixture_path(environment_variable: &str, filename: &str) -> Option<PathBuf> {
    let path = env::var_os(environment_variable)
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("benches")
                .join(filename)
        });

    if path.is_file() {
        Some(path)
    } else {
        eprintln!(
            "skipping {filename}: set {environment_variable} or place the fixture at {}",
            path.display()
        );
        None
    }
}
