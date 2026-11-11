use std::{env, fs, path::PathBuf, process::Command};
fn main() {
    let _ = rsphysics_sixs_source::VERSION;
    let source = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap())
        .join("../models/atmosphere/sixs/fortran");
    println!("cargo:rerun-if-changed={}", source.display());
    for name in ["FC", "FFLAGS", "PYO3_PYTHON", "PYTHON"] {
        println!("cargo:rerun-if-env-changed={name}");
    }
    let output = PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("sixs");
    let python = env::var_os("PYO3_PYTHON")
        .or_else(|| env::var_os("PYTHON"))
        .unwrap_or_else(|| "python".into());
    let result = Command::new(python)
        .arg(source.join("build_tools/library.py"))
        .args(["native", "--output"])
        .arg(&output)
        .output()
        .expect("Python is required to build the original 6SV2.1 Fortran C ABI");
    if !result.status.success() {
        panic!(
            "6SV2.1 library build failed. Install GNU Fortran and set FC to its executable.\n{}\n{}",
            String::from_utf8_lossy(&result.stdout),
            String::from_utf8_lossy(&result.stderr)
        );
    }
    let os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let extension = match os.as_str() {
        "windows" => "dll",
        "macos" => "dylib",
        _ => "so",
    };
    for (variable, program) in [
        ("RSP_SIXS_LIBRARY", "sixsV2.1"),
        ("RSP_SIXS_LUT_LIBRARY", "sixsV2.1-lutaero"),
    ] {
        println!(
            "cargo:rustc-env={variable}={}",
            output.join(format!("{program}.{extension}")).display()
        );
    }
    fs::write(output.join("adapter-revision.txt"), "direct-c-abi-v1").unwrap();
    let mut assets = String::from("static RUNTIME_ASSETS: &[(&str, &[u8])] = &[\n");
    if let Ok(entries) = fs::read_dir(output.join("runtime")) {
        for entry in entries {
            let path = entry.unwrap().path();
            assets.push_str(&format!(
                "({:?}, include_bytes!({:?})),\n",
                path.file_name().unwrap().to_str().unwrap(),
                path.to_str().unwrap()
            ));
        }
    }
    assets.push_str("];\n");
    fs::write(
        PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("runtime-assets.rs"),
        assets,
    )
    .unwrap();
}
