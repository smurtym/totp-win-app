fn main() {
    // Only compile Windows resources when targeting Windows
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() != "windows" {
        return;
    }

    let mut res = winres::WindowsResource::new();
    res.set_icon("assets/app.ico");

    // When cross-compiling on Linux, use the MinGW windres and ar
    #[cfg(not(target_os = "windows"))]
    {
        res.set_windres_path("x86_64-w64-mingw32-windres");
        res.set_ar_path("x86_64-w64-mingw32-ar");
    }

    res.compile().unwrap();

    // The MinGW linker requires a symbol index in the archive; run ranlib to add it
    #[cfg(not(target_os = "windows"))]
    {
        let out_dir = std::env::var("OUT_DIR").unwrap();
        std::process::Command::new("x86_64-w64-mingw32-ranlib")
            .arg(format!("{out_dir}/libresource.a"))
            .status()
            .expect("failed to run x86_64-w64-mingw32-ranlib on libresource.a");
    }
}
