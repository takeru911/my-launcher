pub mod windows_api;
pub mod window_thumbnail;
pub mod core;
pub mod ui;
pub mod logger;
pub mod data;
pub mod filter;

#[cfg(test)]
pub mod test_helpers;

pub use windows_api::{enumerate_windows, switch_to_window};
pub use window_thumbnail::ThumbnailCache;
pub use core::{LauncherCore, SearchEngine, SearchResult, SearchMode, Action, WindowManager, WindowInfo};