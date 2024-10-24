use std::env;
use std::process::Command;

// from https://github.com/dranov/RustCallLean

const LEAN_LIB_DIR: &str = "lib";
const LEAN_BUILD_DIR: &str = "lib/.lake/build/lib";

fn main() {
    let cwd = env::current_dir().expect("Failed to get current directory");

    env::set_current_dir(LEAN_LIB_DIR)
        .expect("Failed to change directory to Lean library directory");
    let _lake_output = Command::new("lake")
        .arg("build")
        .output()
        .expect("Failed to build lake project");
    env::set_current_dir(cwd.clone())
        .expect("Failed to change directory back to original directory");

    let default_callee_path = cwd
        .join(LEAN_BUILD_DIR)
        .canonicalize()
        .expect("Failed to get canonical path");

    let callee_root = default_callee_path.display();
    println!("cargo::rustc-link-search={callee_root}");

    // set the environment for `cargo run`
    // https://stackoverflow.com/a/51799351
    println!("cargo:rustc-env=LD_LIBRARY_PATH={callee_root}");
}
