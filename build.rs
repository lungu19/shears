fn main() {
    // if any of those steps fail, abort
    let mut build_res = true;
    build_res &= set_rustc_env_variable();
    build_res &= set_executable_icon();

    if !build_res {
        std::process::exit(1);
    }
}

#[cfg(target_os = "windows")]
fn set_executable_icon() -> bool {
    if cfg!(target_os = "windows") {
        let mut res = winres::WindowsResource::new();

        res.set_icon("assets\\favicon.ico")
            .set_manifest(include_str!("assets\\app.manifest"));

        if let Err(error) = res.compile() {
            println!("{error}");
            return false;
        }
    }

    true
}

#[cfg(not(target_os = "windows"))]
fn set_executable_icon() -> bool {
    true
}

fn set_rustc_env_variable() -> bool {
    let output = std::process::Command::new("rustc")
        .arg("--version")
        .output()
        .expect("Failed to run rustc");

    match String::from_utf8(output.stdout) {
        Err(error) => {
            println!("{error}");
            false
        }
        Ok(version) => {
            println!("cargo:rustc-env=RUSTC_VERSION={}", version.trim());
            true
        }
    }
}
