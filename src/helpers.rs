use crate::types::{ForgeTextureQualityLevel, ShearingFeaturesAvailability};

use std::io::{Result, Write as _};
use std::path::Path;

fn get_file_size(path: &std::path::Path) -> Result<u64> {
    std::fs::metadata(path).map(|m| m.len())
}

fn get_folder_size(path: &std::path::Path) -> Result<u64> {
    let entries = std::fs::read_dir(path)?;

    let mut total_size: u64 = 0;
    for entry in entries.flatten() {
        let path = entry.path();

        // handle subfolders (mostly for newer versions of siege)
        if path.is_dir() {
            total_size += get_folder_size(&path)?;
            continue;
        }

        let metadata = std::fs::metadata(&path)?;
        total_size += metadata.len();
    }

    Ok(total_size)
}

fn get_videos_subfolder_size(folder: &Path) -> Result<u64> {
    let video_sub_folder = folder.join("videos");

    if !video_sub_folder.exists() || !video_sub_folder.is_dir() {
        return Ok(0);
    }

    get_folder_size(&video_sub_folder)
}

fn get_events_size(folder: &Path) -> u64 {
    let Ok(entries) = std::fs::read_dir(folder) else {
        return 0;
    };

    let mut size = 0;
    for entry in entries.flatten() {
        let path = entry.path();

        let Some(ext) = path.extension().and_then(|s| s.to_str()) else {
            continue;
        };

        let Some(filename) = path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };

        if !ext.eq_ignore_ascii_case("forge") && !ext.eq_ignore_ascii_case("depgraphbin") {
            continue;
        }

        if !filename.contains("events") {
            continue;
        }

        size += get_file_size(&path).unwrap_or(0);
    }

    size
}

pub fn get_shearing_features_availability(folder: &Path) -> ShearingFeaturesAvailability {
    let mut features = ShearingFeaturesAvailability::default();

    let Ok(entries) = std::fs::read_dir(folder) else {
        return features;
    };

    let mut has_forges = false;
    let mut texture_sizes: [u64; ForgeTextureQualityLevel::COUNT] =
        [0; ForgeTextureQualityLevel::COUNT];

    for entry in entries.flatten() {
        let path = entry.path();

        let Some(ext) = path.extension().and_then(|s| s.to_str()) else {
            continue;
        };

        let Some(filename) = path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };

        if !ext.eq_ignore_ascii_case("forge") {
            continue;
        }

        has_forges = true;

        let Some((_, suffix)) = filename.split_once("textures") else {
            continue;
        };

        let Some(first_char) = suffix.chars().next() else {
            continue;
        };

        let Some(digit) = first_char.to_digit(10) else {
            continue;
        };

        let level = digit as i32;
        if !(0..=4).contains(&level) {
            continue;
        }

        let Some(quality_level) = ForgeTextureQualityLevel::convert_from_i32(level) else {
            continue;
        };

        *texture_sizes
            .get_mut(quality_level as usize)
            .expect("get_shearing_features_availability.*texture_sizes: Out of bounds error") +=
            get_file_size(&path).unwrap_or(0);
    }

    features.has_forge_files = has_forges;

    let low = ForgeTextureQualityLevel::Low as usize;
    for (dst, &size) in features
        .textures
        .iter_mut()
        .skip(low)
        .take(ForgeTextureQualityLevel::COUNT)
        .zip(texture_sizes.iter().skip(low))
    {
        *dst = (size > 0, size);
    }

    features.videos = {
        match get_videos_subfolder_size(folder) {
            Err(_) => (false, 0),
            Ok(o) => (o > 0, o),
        }
    };

    features.events = {
        let res = get_events_size(folder);
        (res > 0, res)
    };

    features
}

pub fn delete_texture_files(folder: &Path, min_quality_level: &ForgeTextureQualityLevel) {
    // unless they add a texture quality level above ultra in the future, early exit to avoid useless code
    if *min_quality_level == ForgeTextureQualityLevel::Ultra {
        return;
    }

    let Ok(entries) = std::fs::read_dir(folder) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();

        let Some(ext) = path.extension().and_then(|s| s.to_str()) else {
            continue;
        };
        if !ext.eq_ignore_ascii_case("forge") {
            continue;
        }

        let Some(filename) = path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        let Some((_, suffix)) = filename.split_once("textures") else {
            continue;
        };
        let Some(level) = suffix.chars().next().and_then(|c| c.to_digit(10)) else {
            continue;
        };
        let Some(quality) = ForgeTextureQualityLevel::convert_from_i32(level as i32) else {
            continue;
        };

        if quality > *min_quality_level
            && let Some(err) = std::fs::remove_file(&path).err()
        {
            let path_string = path.display().to_string();
            log::warn!("Unable to delete {path_string} because {err}.");
        }
    }
}

pub fn delete_videos_folder(folder: &Path) {
    let video_sub_folder = folder.join("videos");

    if let Some(err) = std::fs::remove_dir_all(&video_sub_folder).err() {
        let path_string = video_sub_folder.display().to_string();
        log::warn!("Unable to delete {path_string} because {err}.");
    }
}

pub fn delete_events_folder(folder: &Path) {
    let Ok(entries) = std::fs::read_dir(folder) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();

        let Some(ext) = path.extension().and_then(|s| s.to_str()) else {
            continue;
        };
        if !ext.eq_ignore_ascii_case("forge") && !ext.eq_ignore_ascii_case("depgraphbin") {
            continue;
        }

        let Some(filename) = path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };

        if !filename.contains("events") {
            continue;
        }

        if let Some(err) = std::fs::remove_file(&path).err() {
            let path_string = path.display().to_string();
            log::warn!("Unable to delete event file {path_string} because {err}.");
        }
    }
}

pub fn write_streaminginstall(siege_folder: &std::path::Path) -> std::io::Result<()> {
    let streaming_install_path = siege_folder.join("streaminginstall.ini");

    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(streaming_install_path)?;

    file.write_all(b"[MissionToChunk]\n[FileToChunk]\n")?;
    file.flush()?;
    Ok(())
}

pub fn is_siege_running(sys: &mut sysinfo::System) -> bool {
    let possible_process_names = [
        "rainbowsixgame.exe",
        "rainbowsix.exe",
        "rainbowsix_vulkan.exe",
        "rainbowsix_dx11.exe",
    ];

    sys.refresh_processes_specifics(
        sysinfo::ProcessesToUpdate::All,
        true,
        sysinfo::ProcessRefreshKind::nothing(), // only retrieve basic info like name and PID
    );

    // checks all possible siege process names to see if it's open
    sys.processes().values().any(|process| {
        possible_process_names
            .iter()
            .any(|&siege_name| process.name().eq_ignore_ascii_case(siege_name))
    })
}

#[derive(Debug, PartialEq)]
enum ShearsVersionStatus {
    UpToDate,
    NotUpToDate,
    Error,
}

#[cfg_attr(debug_assertions, expect(dead_code))]
fn get_shears_version_status() -> ShearsVersionStatus {
    let response = match minreq::get("http://api.github.com/repos/lungu19/shears/releases/latest")
        .with_header("User-Agent", "shears-update-check")
        .send()
    {
        Ok(res) => res,
        Err(e) => {
            log::error!("Network request failed: {e}");
            return ShearsVersionStatus::Error;
        }
    };

    let raw_json = match response.as_str() {
        Ok(s) => s,
        Err(e) => {
            log::error!("Failed to read response body: {e}");
            return ShearsVersionStatus::Error;
        }
    };

    let json = microjson::JSONValue::load(raw_json);

    if let Ok(version_value) = json.get_key_value("tag_name")
        && let Ok(version_string) = version_value.read_string()
    {
        let current_version = env!("CARGO_PKG_VERSION");

        log::info!("current_version: {current_version}");
        log::info!("version_string: {version_string}");

        if current_version == version_string {
            log::info!("Shears is up-to-date");
            return ShearsVersionStatus::UpToDate;
        } else {
            log::info!("Shears is not up-to-date");
            return ShearsVersionStatus::NotUpToDate;
        }
    }

    log::error!("Failed to parse JSON or find 'tag_name'");
    ShearsVersionStatus::Error
}

#[cfg(not(debug_assertions))]
pub fn run_shears_version_background_check(invoked_automatically: bool) {
    std::thread::spawn(move || {
        let shears_version_status = get_shears_version_status();

        match shears_version_status {
            ShearsVersionStatus::Error => {
                if !invoked_automatically {
                    native_dialog::DialogBuilder::message()
                        .set_level(native_dialog::MessageLevel::Error)
                        .set_title("Something went wrong")
                        .set_text("Failed to check Shears version status. Check log for more information.")
                        .alert()
                        .show()
                        .expect("Failed to show dialog")
                }
            }
            ShearsVersionStatus::UpToDate => {
                if !invoked_automatically {
                    native_dialog::DialogBuilder::message()
                        .set_level(native_dialog::MessageLevel::Info)
                        .set_title("Shears is up-to-date")
                        .set_text("You are currently using the newest version of Shears available!")
                        .alert()
                        .show()
                        .expect("Failed to show dialog")
                }
            }
            ShearsVersionStatus::NotUpToDate => {
                if native_dialog::DialogBuilder::message()
                    .set_level(native_dialog::MessageLevel::Warning)
                    .set_title("Shears is not up-to-date")
                    .set_text("A newer version of Shears is available. Do you want to update now?")
                    .confirm()
                    .show()
                    .expect("Failed to show dialog")
                {
                    open::that("https://github.com/lungu19/shears/releases/latest")
                        .expect("Failed to open link in browser");
                }
            }
        }
    });
}

#[cfg(debug_assertions)]
pub fn run_shears_version_background_check(invoked_automatically: bool) {
    if invoked_automatically {
        return;
    }

    std::thread::spawn(|| {
        native_dialog::DialogBuilder::message()
            .set_level(native_dialog::MessageLevel::Info)
            .set_title("Updated Check")
            .set_text("Debug Update Check")
            .alert()
            .show()
            .expect("Failed to show dialog");
    });
}
