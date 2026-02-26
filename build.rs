use std::process::Command;
use std::fs;

fn main() {
    let ase_dir = "assets/ase";
    let out_dir = "assets/sprites";

    fs::create_dir_all(out_dir).unwrap();

    let mut exported_files = Vec::new();

    if let Ok(entries) = fs::read_dir(ase_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("aseprite") || 
               path.extension().and_then(|s| s.to_str()) == Some("ase") 
            {
                let file_name = path.file_stem().unwrap().to_str().unwrap();
                let png_path = format!("{}/{}.png", out_dir, file_name);
                let json_path = format!("{}/{}.json", out_dir, file_name);

                exported_files.push(file_name.to_string());

                println!("cargo:warning=Exporting Aseprite: {}", file_name);

                let status = Command::new("aseprite")
                    .arg("-b")
                    .arg(&path)
                    .arg("--sheet")
                    .arg(&png_path)
                    .arg("--data")
                    .arg(&json_path)
                    .arg("--format")
                    .arg("json-array")
                    .arg("--list-tags")  
                    .arg("--list-slices") 
                    .status();

                if let Err(e) = status {
                    println!("cargo:warning=Failed to run Aseprite: {}", e);
                }
            }
        }
    }

    let index_json = format!("[\n  {}\n]", exported_files.iter().map(|s| format!("\"{}\"", s)).collect::<Vec<_>>().join(",\n  "));
    fs::write(format!("{}/index.json", out_dir), index_json).unwrap();

    println!("cargo:rerun-if-changed=assets/ase");
}