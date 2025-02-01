fn main() {
    #[cfg(target_os = "windows")]
    {
        let ico_path = ico_builder::IcoBuilder::default()
            .add_source_file("assets/icon.png")
            .build_file_cargo("icon.ico")
            .expect("无法生成图标");

        winres::WindowsResource::new()
            .set_icon(ico_path.to_string_lossy().as_ref())
            .compile()
            .unwrap();
    }
}
