fn main() {
    let mut res = winres::WindowsResource::new();
    res.set_icon_with_id("icon.ico", "69");
    res.set_manifest(include_str!("manifest.xml"));
    res.compile().expect("Resource compiling should succeed");
}
