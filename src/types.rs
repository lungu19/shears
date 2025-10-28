use std::cmp::Ordering;
use std::path::PathBuf;

#[derive(Clone, Copy, Debug)]
pub struct ShearsUiState {
    pub page: ShearsPage,

    pub checkbox_textures: [bool; ForgeTextureQualityLevel::COUNT],
    pub checkbox_videos: bool,

    pub label_possible_space_saved: u64,

    pub modals: [bool; ShearsModals::COUNT],
}

impl Default for ShearsUiState {
    fn default() -> Self {
        Self {
            page: ShearsPage::SelectFolder,

            checkbox_textures: [true; ForgeTextureQualityLevel::COUNT],
            checkbox_videos: true,

            label_possible_space_saved: 0,
            modals: [false; ShearsModals::COUNT],
        }
    }
}

impl ShearsUiState {
    pub fn get_texture_checkbox(self, quality_level: usize) -> bool {
        *self
            .checkbox_textures
            .get(quality_level)
            .expect("Out of bounds error")
    }

    pub fn get_texture_checkbox_mut(&mut self, quality_level: usize) -> &mut bool {
        self.checkbox_textures
            .get_mut(quality_level)
            .expect("Out of bounds error")
    }

    pub fn get_modal(self, modal_index: usize) -> bool {
        *self.modals.get(modal_index).expect("Out of bounds error")
    }

    pub fn get_modal_mut(&mut self, modal_index: usize) -> &mut bool {
        self.modals
            .get_mut(modal_index)
            .expect("Out of bounds error")
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

    pub textures: [(bool, u64); ForgeTextureQualityLevel::COUNT],
    pub videos: (bool, u64),
}

impl ShearingFeaturesAvailability {
    pub fn get_texture(self, quality_level: usize) -> (bool, u64) {
        *self
            .textures
            .get(quality_level)
            .expect("Out of bounds error")
    }

    pub fn get_texture_mut(&mut self, quality_level: usize) -> &mut (bool, u64) {
        self.textures
            .get_mut(quality_level)
            .expect("Out of bounds error")
    }
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
    pub const COUNT: usize = 5;

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

#[derive(Clone, Copy, Debug)]
pub enum ShearsModals {
    About = 0,
}

impl ShearsModals {
    pub const START: Self = Self::About;
    pub const END: Self = Self::About;
    pub const COUNT: usize = 2;
}
