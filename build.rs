fn main() {
    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("app_icon.ico");
        let _ = res.compile();
    }
    slint_build::compile("ui/appwindow.slint").unwrap();
}
