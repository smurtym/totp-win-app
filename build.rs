fn main() {
    // Only embed resources when targeting Windows
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() != "windows" {
        return;
    }

    #[cfg(target_os = "windows")]
    {
        // Native Windows build: let winres handle everything normally
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/app.ico");
        res.compile().unwrap();
    }

    #[cfg(not(target_os = "windows"))]
    {
        // Cross-compile from Linux.
        //
        // We cannot use winres's normal compile() path here because Rust's
        // release profile passes -Wl,--gc-sections to the GNU linker, which
        // silently garbage-collects the .rsrc section from libresource.a since
        // no code references it. Linking the .o directly (via rustc-link-arg)
        // bypasses --gc-sections and keeps the icon in the final .exe.
        use std::process::Command;

        let out_dir = std::env::var("OUT_DIR").unwrap();
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();

        // Write a minimal .rc that embeds the icon
        let rc_path = format!("{out_dir}/icon.rc");
        let obj_path = format!("{out_dir}/icon.o");
        std::fs::write(
            &rc_path,
            format!(
                "#pragma code_page(65001)\n1 ICON \"{}/assets/app.ico\"\n",
                manifest_dir.replace('\\', "/")
            ),
        )
        .expect("failed to write icon.rc");

        // Compile .rc → COFF .o with the MinGW windres
        let status = Command::new("x86_64-w64-mingw32-windres")
            .args(["--input", &rc_path, "--output", &obj_path, "--output-format=coff"])
            .status()
            .expect("x86_64-w64-mingw32-windres not found — install binutils-mingw-w64-x86-64");
        assert!(status.success(), "windres failed");

        // Pass the object directly to the linker — not as a static lib — so
        // --gc-sections cannot drop the .rsrc section
        println!("cargo:rustc-link-arg={obj_path}");
    }
}

