use crate::scan::scan_recursive;

#[derive(Debug)]
pub struct ShearsScanFolderState {
    pub disks: sysinfo::Disks,
    pub thread_handle: Option<std::thread::JoinHandle<Vec<std::path::PathBuf>>>,
    pub stop_flag: std::sync::Arc<std::sync::atomic::AtomicBool>,
    pub scan_results: Option<Vec<std::path::PathBuf>>,
    timer_start: std::time::Instant,
}

impl Default for ShearsScanFolderState {
    fn default() -> Self {
        Self {
            disks: sysinfo::Disks::default(),
            timer_start: std::time::Instant::now(),
            thread_handle: None,
            stop_flag: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            scan_results: None,
        }
    }
}

impl ShearsScanFolderState {
    pub fn start_scan_thread(&mut self, drive: std::path::PathBuf) {
        self.stop_flag
            .store(false, std::sync::atomic::Ordering::Relaxed);

        let flag_clone = self.stop_flag.clone();

        self.restart_timer();
        self.thread_handle = Some(std::thread::spawn(move || {
            let mut found_paths = Vec::new();
            scan_recursive(&drive, &mut found_paths, &flag_clone);

            if flag_clone.load(std::sync::atomic::Ordering::Relaxed) {
                log::info!("Scan was cancelled early.");
            }

            found_paths
        }));
    }

    fn restart_timer(&mut self) {
        self.timer_start = std::time::Instant::now();
    }

    pub fn update_disks(&mut self) {
        self.disks = sysinfo::Disks::new_with_refreshed_list();
    }

    pub fn timer_elapsed(&self) -> String {
        let seconds = self.timer_start.elapsed().as_secs();

        let hours = seconds / 3600;
        let minutes = (seconds % 3600) / 60;
        let secs = seconds % 60;

        if hours > 0 {
            return format!("{hours:02}:{minutes:02}:{secs:02}");
        }

        format!("{minutes:02}:{secs:02}")
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ShearsUiState {
    page: ShearsPage,
    last_page: ShearsPage,

    pub checkbox_textures: [bool; ForgeTextureQualityLevel::COUNT],
    pub checkbox_videos: bool,
    pub checkbox_events: bool,

    pub label_possible_space_saved: u64,

    pub modals: [bool; ShearsModals::COUNT],
}

impl Default for ShearsUiState {
    fn default() -> Self {
        Self {
            page: ShearsPage::MainPage,
            last_page: ShearsPage::MainPage,

            checkbox_textures: [true; ForgeTextureQualityLevel::COUNT],
            checkbox_videos: true,
            checkbox_events: true,

            label_possible_space_saved: 0,
            modals: [false; ShearsModals::COUNT],
        }
    }
}

impl ShearsUiState {
    pub fn reset_pages(&mut self) {
        self.page = ShearsPage::MainPage;
        self.last_page = ShearsPage::MainPage;
    }

    pub fn change_page(&mut self, new_page: ShearsPage) {
        let current_page = self.page;
        self.page = new_page;
        self.last_page = current_page;
    }

    pub fn change_page_no_history(&mut self, new_page: ShearsPage) {
        self.page = new_page;
        self.last_page = ShearsPage::MainPage;
    }

    pub fn go_back(&mut self) {
        self.page = self.last_page;
        self.last_page = ShearsPage::MainPage;
    }

    pub fn get_page(&self) -> ShearsPage {
        self.page
    }

    pub fn get_last_page(&self) -> ShearsPage {
        self.last_page
    }

    pub fn get_texture_checkbox(self, quality_level: usize) -> bool {
        *self
            .checkbox_textures
            .get(quality_level)
            .expect("ShearsUiState.get_texture_checkbox: Out of bounds error")
    }

    pub fn get_texture_checkbox_mut(&mut self, quality_level: usize) -> &mut bool {
        self.checkbox_textures
            .get_mut(quality_level)
            .expect("ShearsUiState.get_texture_checkbox_mut: Out of bounds error")
    }

    pub fn get_modal(self, modal_index: usize) -> bool {
        *self
            .modals
            .get(modal_index)
            .expect("ShearsUiState.get_modal: Out of bounds error")
    }

    pub fn get_modal_mut(&mut self, modal_index: usize) -> &mut bool {
        self.modals
            .get_mut(modal_index)
            .expect("ShearsUiState.get_modal_mut: Out of bounds error")
    }
}

#[derive(Default, Clone, Debug)]
pub struct ShearsFolderState {
    pub siege_path: Option<std::path::PathBuf>,
    pub features_availability: ShearingFeaturesAvailability,
}

#[derive(Default, Clone, Copy, Debug)]
pub struct ShearingFeaturesAvailability {
    pub has_forge_files: bool,

    pub textures: [(bool, u64); ForgeTextureQualityLevel::COUNT],
    pub videos: (bool, u64),
    pub events: (bool, u64),
}

impl ShearingFeaturesAvailability {
    pub fn get_texture(self, quality_level: usize) -> (bool, u64) {
        *self
            .textures
            .get(quality_level)
            .expect("ShearingFeaturesAvailability.get_texture: Out of bounds error")
    }

    pub fn get_texture_mut(&mut self, quality_level: usize) -> &mut (bool, u64) {
        self.textures
            .get_mut(quality_level)
            .expect("ShearingFeaturesAvailability.get_texture_mut: Out of bounds error")
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
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
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
    MainPage = 0,
    FolderSelected,
    DiskScanSelect,
    DiskScanInProgress,
    DiskScanComplete,
}

#[derive(Clone, Copy, Debug)]
pub enum ShearsModals {
    About = 0,
    Settings,
}

impl ShearsModals {
    pub const START: Self = Self::About;
    pub const END: Self = Self::Settings;
    pub const COUNT: usize = 2;
}
