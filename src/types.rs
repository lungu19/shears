use std::cmp::Ordering;
use std::path::PathBuf;

#[derive(Clone, Copy, Debug)]
pub struct ShearsUiState {
    pub page: ShearsPage,

    pub checkbox_textures_low: bool,
    pub checkbox_textures_medium: bool,
    pub checkbox_textures_high: bool,
    pub checkbox_textures_very_high: bool,
    pub checkbox_textures_ultra: bool,

    pub checkbox_videos: bool,

    pub modal_about: bool,

    pub label_possible_space_saved: u64,
}

impl Default for ShearsUiState {
    fn default() -> Self {
        Self {
            page: ShearsPage::SelectFolder,

            checkbox_textures_low: true,
            checkbox_textures_medium: true,
            checkbox_textures_high: true,
            checkbox_textures_very_high: true,
            checkbox_textures_ultra: true,

            checkbox_videos: true,

            modal_about: false,

            label_possible_space_saved: 0,
        }
    }
}

#[derive(Default, Clone, Debug)]
pub struct ShearsFolderState {
    pub siege_path: Option<PathBuf>,
    pub features_availability: ShearingFeaturesAvailability,
}

#[derive(Default, Clone, Copy, Debug)]
pub struct ShearingFeaturesAvailability {
    pub has_forge_files: bool,

    pub textures_low: (bool, u64),
    pub textures_medium: (bool, u64),
    pub textures_high: (bool, u64),
    pub textures_very_high: (bool, u64),
    pub textures_ultra: (bool, u64),
    pub videos: (bool, u64),
}

#[derive(Debug, Clone, Copy)]
pub enum ForgeTextureQualityLevel {
    Low = 0,
    Medium = 1,
    High = 2,
    VeryHigh = 3,
    Ultra = 4,
}

impl PartialOrd for ForgeTextureQualityLevel {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.convert_to_i32().cmp(&other.convert_to_i32()))
    }
}

impl PartialEq for ForgeTextureQualityLevel {
    fn eq(&self, other: &Self) -> bool {
        self.convert_to_i32() == other.convert_to_i32()
    }
}

impl Eq for ForgeTextureQualityLevel {}

impl std::fmt::Display for ForgeTextureQualityLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Low => write!(f, "Low"),
            Self::Medium => write!(f, "Medium"),
            Self::High => write!(f, "High"),
            Self::VeryHigh => write!(f, "Very High"),
            Self::Ultra => write!(f, "Ultra"),
        }
    }
}

impl ForgeTextureQualityLevel {
    pub fn convert_from_i32(texture_level: i32) -> Option<Self> {
        match texture_level {
            0 => Some(Self::Low),
            1 => Some(Self::Medium),
            2 => Some(Self::High),
            3 => Some(Self::VeryHigh),
            4 => Some(Self::Ultra),
            _ => None,
        }
    }

    pub fn convert_to_i32(&self) -> i32 {
        match self {
            Self::Low => 0,
            Self::Medium => 1,
            Self::High => 2,
            Self::VeryHigh => 3,
            Self::Ultra => 4,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum ShearsPage {
    SelectFolder = 0,
    FolderSelected,
}
