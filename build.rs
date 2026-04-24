fn main() {
    #[cfg(target_os = "windows")]
    {
        // Embed the application icon and version info into the Windows .exe
        // so that Explorer, taskbar and Start menu display it in release builds.
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/icon.ico");
        if let Err(e) = res.compile() {
            eprintln!("warning: failed to embed Windows resources: {e}");
        }
    }

    println!("cargo:rerun-if-changed=assets/icon.ico");
    println!("cargo:rerun-if-changed=build.rs");
}
