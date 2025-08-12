// used for executable icon

fn main() {
    if cfg!(target_os = "windows") {
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets\\favicon.ico").set_manifest(
            r#"
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
<trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
    <security>
        <requestedPrivileges>
            <requestedExecutionLevel level="asInvoker" uiAccess="false" />
        </requestedPrivileges>
    </security>
</trustInfo>
</assembly>
"#,
        );
        if let Err(error) = res.compile() {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}
