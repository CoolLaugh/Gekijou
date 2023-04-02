pub const ANIME_PER_PAGE: usize = 50;
pub const SECONDS_IN_MINUTES: i32 = 60;
pub const DEFAULT_HIGHLIGHT_COLOR: &str = "rgb(96, 217, 236)";
pub const ANIME_UPDATE_DELAY: u64 = 5;
pub const SIMILARITY_SCORE_THRESHOLD: f64 = 0.8;
pub const STARTUP_SCAN_DELAY: u64 = 30;
pub const ONE_HOUR: u64 = 60 * 60;
pub const BROWSE_PAGE_LIMIT: i32 = 4;


#[cfg(debug_assertions)]
pub const DEBUG: bool = true;
#[cfg(not(debug_assertions))]
pub const DEBUG: bool = false;