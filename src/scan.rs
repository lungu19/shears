fn is_restricted_or_junk(name: &std::ffi::OsStr) -> bool {
    let Some(name_str) = name.to_str() else {
        return false;
    };

    // list of stuff that is either restricted without admin access, not useful for us, probably doesnt contain a old siege instance etc...
    matches!(
        name_str,
        "$RECYCLE.BIN"
            | "$Recycle.Bin"
            | ".Trash-1000"
            | "Config.Msi"
            | "$Windows.~BT"
            | "$Windows.~WS"
            | "System Volume Information"
            | "WindowsApps"
            | "Recovery"
            | "MSOCache"
            | "PerfLogs"
            | "Microsoft"
            | "Windows"
            | "ProgramData"
            | "Temp"
            | "NVIDIA"
            | "Program Files"
            | "Program Files (x86)"
    )
}

pub fn scan_recursive(
    dir: &std::path::Path,
    results: &mut Vec<std::path::PathBuf>,
    stop_flag: &std::sync::atomic::AtomicBool,
) {
    if stop_flag.load(std::sync::atomic::Ordering::Relaxed) {
        return;
    }

    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(e) => {
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                log::warn!("Access Denied (Skipping): {}", dir.display());
            }
            return;
        }
    };

    let mut found_forge = false;
    let mut found_exe = false;
    let mut subdirs = Vec::new();

    for entry in entries.filter_map(|e| e.ok()) {
        if stop_flag.load(std::sync::atomic::Ordering::Relaxed) {
            return;
        }

        let file_name = entry.file_name();

        if file_name == "datapc64.forge" {
            found_forge = true;
        } else if file_name == "RainbowSix.exe" {
            found_exe = true;
        }

        if let Ok(ft) = entry.file_type()
            && ft.is_dir()
            && !is_restricted_or_junk(&file_name)
        {
            subdirs.push(entry.path());
        }
    }

    if found_forge && found_exe {
        log::info!("FOUND IN: {}", dir.display());
        results.push(dir.to_path_buf());
    }

    for sub in subdirs {
        if stop_flag.load(std::sync::atomic::Ordering::Relaxed) {
            return;
        }
        scan_recursive(&sub, results, stop_flag);
    }
}
