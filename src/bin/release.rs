use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    let args: Vec<String> = env::args().collect();
    let target_dir = if args.len() > 1 {
        args[1].clone()
    } else {
        "ReleaseBuild".to_string()
    };

    println!("Building release version of the game...");
    
    let status = Command::new("cargo")
        .arg("build")
        .arg("--release")
        .status()
        .expect("Failed to run cargo build");

    if !status.success() {
        eprintln!("Build failed!");
        return;
    }

    println!("📁 Creating release folder at: {}", target_dir);
    let _ = fs::remove_dir_all(&target_dir); 
    fs::create_dir_all(&target_dir).unwrap();

    let exe_name = if cfg!(windows) { "en.exe" } else { "en" };
    let exe_src = format!("target/release/{}", exe_name);
    let exe_dst = format!("{}/{}", target_dir, exe_name);
    
    if Path::new(&exe_src).exists() {
        fs::copy(&exe_src, &exe_dst).unwrap();
    } else {
        eprintln!("Executable not found at {}! Make sure your package name in Cargo.toml is 'en'", exe_src);
        return;
    }

    if Path::new("Scene.bin").exists() {
        fs::copy("Scene.bin", format!("{}/Scene.bin", target_dir)).unwrap();
    }

    println!("Copying assets...");
    copy_assets(Path::new("assets"), Path::new(&format!("{}/assets", target_dir))).unwrap();

    println!("Done! Your game is ready to ship in: {}", target_dir);
}

fn copy_assets(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let file_name = entry.file_name();

        if file_type.is_dir() {
            if file_name == "ase" { continue; }
            copy_assets(&entry.path(), &dst.join(file_name))?;
        } else {
            fs::copy(entry.path(), dst.join(file_name))?;
        }
    }
    Ok(())
}