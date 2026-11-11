fn main() {
    slint_build::compile("ui/renova.slint").expect("Compilation Slint réussie");

    #[cfg(windows)]
    {
        let kit_path = r"C:\Program Files (x86)\Windows Kits\10\bin\10.0.26100.0\x64";
        let mut res = winres::WindowsResource::new();
        if std::path::Path::new(kit_path).join("rc.exe").exists() {
            res.set_toolkit_path(kit_path);
        }
        res.set_icon("icons/icon.ico");
        res.set("ProductName", "RenovaPlan");
        res.set("FileDescription", "Logiciel natif 2D/3D de plans et rénovation de maison");
        res.set("LegalCopyright", "Copyright (c) 2026 Lib-LOCALE");
        if let Err(e) = res.compile() {
            eprintln!("cargo:warning=winres compilation failed: {e}");
        } else if let Ok(out_dir) = std::env::var("OUT_DIR") {
            let resource_o = std::path::PathBuf::from(out_dir).join("resource.o");
            if resource_o.exists() {
                // Ensure GNU ld does not drop resource section
                println!("cargo:rustc-link-arg={}", resource_o.display());
            }
        }
    }
}
