use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    let project_name = "en";
    
    let args: Vec<String> = env::args().collect();
    let target_dir = if args.len() > 1 {
        args[1].clone()
    } else {
        "WebBuild".to_string()
    };
    
    let build_dir = PathBuf::from(target_dir);

    println!("Starting Web (WASM) build in {}...", build_dir.display());

    if build_dir.exists() { fs::remove_dir_all(&build_dir).unwrap(); }
    fs::create_dir_all(&build_dir).unwrap();

    let status = Command::new("cargo")
        .args(["build", "--release", "--target", "wasm32-unknown-unknown"])
        .status()
        .expect("Failed to run cargo build");

    if !status.success() { 
        eprintln!("❌ Build failed!");
        return; 
    }

    let wasm_src = format!("target/wasm32-unknown-unknown/release/{}.wasm", project_name);
    fs::copy(&wasm_src, build_dir.join("game.wasm")).expect("Failed to copy WASM");

    let html_content = format!(r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8">
    <title>{}</title>
    <style>
        html, body, canvas {{ margin: 0; padding: 0; width: 100%; height: 100%; overflow: hidden; background: black; }}
    </style>
</head>
<body>
    <canvas id="glcanvas" tabindex='1'></canvas>
    <script src="https://not-fl3.github.io/miniquad-samples/mq_js_bundle.js"></script>
    <script>load("game.wasm");</script>
</body>
</html>
"#, project_name);

    fs::write(build_dir.join("index.html"), html_content).unwrap();

    copy_dir_recursive(Path::new("assets"), &build_dir.join("assets")).unwrap();

    if Path::new("Scene.bin").exists() {
        fs::copy("Scene.bin", build_dir.join("Scene.bin")).unwrap();
    }

    println!("✅ Web build ready in {}!", build_dir.display());
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_name = entry.file_name().into_string().unwrap();
        if file_name == "ase" { continue; }
        
        if entry.file_type()?.is_dir() {
            copy_dir_recursive(&entry.path(), &dst.join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.join(entry.file_name()))?;
        }
    }
    Ok(())
}