use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    // Compile GResource
    glib_build_tools::compile_resources(
        &["resources"],
        "resources/gresource.xml",
        "zongflow.gresource",
    );

    // Set environment variables for config.rs
    println!("cargo:rustc-env=APP_ID=com.github.zongflow");
    println!("cargo:rustc-env=GETTEXT_PACKAGE=zongflow");
    println!("cargo:rustc-env=LOCALEDIR=locales");

    // Compile .po files to .mo files
    compile_translations();

    // Re-run if translation files change
    println!("cargo:rerun-if-changed=po/");
}

/// Compile .po files to .mo binary files
fn compile_translations() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let po_dir = Path::new(&manifest_dir).join("po");

    if !po_dir.exists() {
        println!("cargo:warning=po/ directory not found, skipping translation compilation");
        return;
    }

    // Create locales directory structure in the project root
    let locales_dir = Path::new(&manifest_dir).join("locales");
    fs::create_dir_all(&locales_dir).expect("Failed to create locales directory");

    // Read LINGUAS file to get list of languages
    let linguas_file = po_dir.join("LINGUAS");
    let languages = if linguas_file.exists() {
        let content = fs::read_to_string(&linguas_file).expect("Failed to read LINGUAS");
        content
            .lines()
            .filter(|line| !line.starts_with('#') && !line.trim().is_empty())
            .map(|line| line.trim().to_string())
            .collect::<Vec<_>>()
    } else {
        // Fallback to all .po files
        fs::read_dir(&po_dir)
            .expect("Failed to read po directory")
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();
                if path.extension().and_then(|ext| ext.to_str()) == Some("po") {
                    path.file_stem()
                        .and_then(|stem| stem.to_str())
                        .map(|s| s.to_string())
                } else {
                    None
                }
            })
            .collect()
    };

    for lang in &languages {
        let po_file = po_dir.join(format!("{}.po", lang));
        if !po_file.exists() {
            println!("cargo:warning=Translation file {:?} not found", po_file);
            continue;
        }

        // Create locale directory structure
        let locale_dir = locales_dir.join(lang).join("LC_MESSAGES");
        fs::create_dir_all(&locale_dir).expect("Failed to create locale directory");

        let mo_file = locale_dir.join("zongflow.mo");

        // Use msgfmt to compile .po to .mo
        let status = Command::new("msgfmt")
            .arg(&po_file)
            .arg("-o")
            .arg(&mo_file)
            .status();

        match status {
            Ok(exit_status) if exit_status.success() => {
                println!(
                    "cargo:warning=Compiled {} to {:?}",
                    po_file.display(),
                    mo_file
                );
            }
            Ok(exit_status) => {
                println!(
                    "cargo:warning=msgfmt failed for {}: exit code {:?}",
                    po_file.display(),
                    exit_status.code()
                );
            }
            Err(e) => {
                println!(
                    "cargo:warning=Failed to run msgfmt for {}: {}. \
                    Make sure gettext tools are installed.",
                    po_file.display(),
                    e
                );
            }
        }
    }
}
