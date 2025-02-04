fn main() {
    #[cfg(target_os = "windows")]
    {
        let ico_path = ico_builder::IcoBuilder::default()
            .add_source_file("assets/icon.png")
            .build_file_cargo("icon.ico")
            .expect("无法生成图标");

        let mut res = winres::WindowsResource::new();

        res.set_manifest(
            r#"
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
    <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
        <security>
            <requestedPrivileges>
                <requestedExecutionLevel level="requireAdministrator" uiAccess="false"/>
            </requestedPrivileges>
        </security>
    </trustInfo>
</assembly>
    "#,
        );

        res.set_icon(ico_path.to_string_lossy().as_ref())
            .compile()
            .unwrap();
    }
}
