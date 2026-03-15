fn main() {
    slint_build::compile("ui/main.slint").unwrap();

    // Embed icon into the Windows .exe so it shows in File Explorer,
    // taskbar, Start Menu, and Desktop shortcuts.
    #[cfg(target_os = "windows")]
    {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/icon.ico");
        res.compile().unwrap();
    }
}
