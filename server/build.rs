use std::{env, path::Path, process::Command};

fn main() {
    println!("cargo:rerun-if-changed=templates");
    println!("cargo:rerun-if-changed=assets");

    std::fs::remove_dir_all("build").unwrap_or_default();

    let out_dir = env::var("OUT_DIR").unwrap();

    Command::new("tailwindcss")
        .args([
            "-c",
            "tailwind.config.js",
            "-i",
            "assets/styles/index.css",
            "-o",
            "./build/index.css",
            "--minify",
        ])
        .status()
        .expect("failed to run tailwindcss");

    Command::new("bun")
        .args([
            "build",
            "--minify",
            "--outdir=build",
            "--entry-naming",
            "[name].[hash].[ext]",
            "--asset-naming",
            "[name].[hash].[ext]",
            "./assets/scripts/index.ts",
        ])
        .status()
        .expect("failed to run bun");

    

        Command::new("bun")
        .args([
            "build",
            "--minify",
            "--outdir=build",
            "--entry-naming",
            "[name].[ext]",
            "--asset-naming",
            "[name].[ext]",
            "./assets/scripts/lexical_editor.ts",
            "--external=lexical",
            "--external=@lexical/selection",
            "--external=@lexical/utils",
            "--external=@lexical/html",
            "--external=@lexical/clipboard",
        ])
        .status()
        .expect("failed to build lexical");

        Command::new("bun")
        .args([
            "build",
            "--minify",
            "--outdir=build",
            "--entry-naming",
            "[name].[hash].[ext]",
            "--asset-naming",
            "[name].[hash].[ext]",
            "./assets/scripts/quill.ts",
        ])
        .status()
        .expect("failed to run bun");



    std::fs::remove_file("build/index.css").unwrap_or_default();
    copy_files("public");
}

fn copy_files(dir: &str) {
    for entry in std::fs::read_dir(dir).expect("failed to read dir `public`") {
        let entry = entry.expect("failed to read entry");

        if entry.file_type().unwrap().is_dir() {
            copy_files(entry.path().to_str().unwrap());
        } else {
            let path = entry.path();
            let filename = path.file_name().unwrap().to_str().unwrap();
            let dest = format!("build/{}", filename);

            std::fs::copy(path, dest).expect("failed to copy file");
        }
    }
}
