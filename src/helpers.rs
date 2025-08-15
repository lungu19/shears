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

        texture_sizes[quality_level as usize] += get_file_size(&path).unwrap_or(0);
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

        if quality > *min_quality_level {
            if let Some(err) = std::fs::remove_file(&path).err() {
                let path_string = path.display().to_string();
                log::warn!("Unable to delete {path_string} because {err}.");
            }
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

pub fn windows_confirmation_dialog(title: &str, content: &str) -> bool {
    match win_msgbox::warning::<win_msgbox::OkayCancel>(content)
        .title(title)
        .show()
        .expect("Failed to show windows messagebox")
    {
        win_msgbox::OkayCancel::Okay => true,
        win_msgbox::OkayCancel::Cancel => false,
    }
}
