use static_files::resource_dir;
use std::process::Command;

fn main() -> std::io::Result<()> {
    #[cfg(debug_assertions)]
    println!("cargo:rerun-if-changed=../frontend");

    let install_output = Command::new("pnpm")
        .args(["i", "--frozen-lockfile", "--ignore-scripts"])
        .output()?;
    if !install_output.status.success() {
        panic!("pnpm install failed: {:?}", install_output);
    }
    let build_output = Command::new("pnpm").args(["run", "build"]).output()?;
    if !build_output.status.success() {
        panic!("pnpm build failed: {:?}", build_output);
    }

    resource_dir("./src/dist/assets").build()?;

    Ok(())
}
