fn main() {
    println!("cargo:rerun-if-changed=../anilist-iced/src/image/icon.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        winresource::WindowsResource::new()
            .set_icon("../anilist-iced/src/image/icon.ico")
            .compile()
            .expect("compile Anilist Windows icon");
    }
}
