# My Launcher Development Guide

## Environment Variables

### Browser Selection
- `LAUNCHER_ENABLE_CHROME=true` - Enable Chrome browser data (default: false)
- `LAUNCHER_ENABLE_WAVEBOX=true` - Enable Wavebox browser data (default: true)

Example:
```bash
# Enable both browsers
LAUNCHER_ENABLE_CHROME=true LAUNCHER_ENABLE_WAVEBOX=true cargo run

# Only Wavebox (default)
cargo run

# Only Chrome
LAUNCHER_ENABLE_CHROME=true LAUNCHER_ENABLE_WAVEBOX=false cargo run
```

## Environment Setup

### Prerequisites
- Rust toolchain (install via [rustup.rs](https://rustup.rs/))
- Windows: Visual Studio Build Tools or MinGW
- Linux/WSL: MinGW for cross-compilation (`sudo dnf install mingw64-gcc mingw64-gcc-c++`)

### Initial Setup
```bash
# Load Rust environment (if needed)
source ~/.cargo/env

# Add Windows targets
rustup target add x86_64-pc-windows-msvc
rustup target add x86_64-pc-windows-gnu
```

## Build Commands

### Cross-platform builds
```bash
# Windows (using MinGW on Linux/WSL)
cargo build --target x86_64-pc-windows-gnu

# Windows with SQLite support (for Chrome history)
cargo build --target x86_64-pc-windows-gnu --features sqlite

# Windows (using MSVC - requires Windows or proper setup)
cargo build --target x86_64-pc-windows-msvc

# Release builds (optimized for performance)
cargo build --release --target x86_64-pc-windows-gnu --features sqlite

# Running release build with only Wavebox (best performance)
cargo run --release --target x86_64-pc-windows-gnu --features sqlite

# Using build scripts
./build.ps1   # PowerShell (Windows)
./build.bat   # Command Prompt (Windows)
```

## Testing Commands

### Run all tests
```bash
cargo test
```

### Run unit tests only
```bash
cargo test --lib
```

### Run integration tests with feature flag
```bash
cargo test --features test-support
```

### Run specific test
```bash
cargo test test_window_search
```

### Run tests with output
```bash
cargo test -- --nocapture
RUST_BACKTRACE=1 cargo test
```

### Test scripts
```powershell
.\test.ps1  # PowerShell
test.bat    # Command Prompt
```

## Code Quality

### Format code
```bash
cargo fmt
```

### Run linter
```bash
cargo clippy -- -D warnings
```

### Fix warnings automatically
```bash
cargo fix --lib -p my-launcher
```

## Architecture Overview

### Core Modules (`src/core/`)

1. **search_engine.rs**
   - Trait: `SearchEngine` - Abstraction for search logic
   - Implementations: 
     - `DefaultSearchEngine` (deprecated)
     - `BrowserSearchEngine` - Full-featured with browser data
   - Search modes:
     - Browser: Google search + Chrome bookmarks + history
     - Windows: Filter cached windows by text
   - Action types: SwitchWindow, GoogleSearch, OpenBookmark, OpenHistory
   - Result types: GoogleSearch, Bookmark, History, Window

2. **browser_search_engine.rs**
   - Advanced search implementation
   - Integrates with Chrome browser data
   - Uses SQL-side search methods for better performance
   - Thread-safe with Arc<Mutex<>>

3. **window_manager.rs**
   - Trait: `WindowManager` - Abstraction for window operations
   - Implementation: `WindowsApiManager` (platform-specific)
   - Mock: `MockWindowManager` (for testing)
   - Data structure: `WindowInfo`

4. **launcher.rs**
   - `LauncherCore<S: SearchEngine, W: WindowManager>`
   - Combines search and window management
   - Handles action execution (window switch, URL open)

### Data Layer (`src/data/`)

1. **window_item.rs**
   - `WindowItem` struct - Window information holder
   - Implements `GridItem` trait for UI display
   - Implements `Searchable` trait for filtering

2. **window_provider.rs**
   - Trait: `WindowProvider` - Abstraction for window data source
   - Implementation: `WindowsApiProvider`
   - Handles window enumeration and caching

3. **browser_item.rs**
   - `BookmarkItem` - Chrome bookmark data
   - `HistoryItem` - Chrome history data
   - Both implement `Searchable` trait
   - Supports folder hierarchy for bookmarks

4. **browser_provider.rs**
   - Trait: `BrowserDataProvider` - Browser data abstraction
   - `ChromeBrowserProvider` - Reads Chrome data
   - `CachedBrowserProvider` - Performance optimization with Mutex-based caching
   - Auto-detects Chrome profile on Windows
   - SQLite support for history (optional feature)
   - Direct Chrome history reading using SQLite immutable mode (no file copy needed)
   - Supports multiple Chrome profiles and Wavebox browser
   - Implements search_bookmarks() and search_history() for SQL-side filtering

### Filter Layer (`src/filter/`)

1. **window_filter.rs**
   - Trait: `WindowFilter` - Window-specific filtering
   - `TaskbarWindowFilter` - Filters windows shown in taskbar
   - `CompositeFilter` - Combines multiple filters

2. **search_filter.rs**
   - Trait: `Searchable` - Generic search interface
   - `SearchFilter` - Text-based search implementation
   - Supports multiple search fields

### UI Layer (`src/ui/`)

1. **alt_tab_grid.rs**
   - `AltTabGrid` - Alt+Tab style grid UI component
   - Trait: `GridItem` - Interface for displayable items
   - Keyboard navigation support
   - Thumbnail display integration

2. **browser_list.rs**
   - `BrowserList` - Dedicated UI component for Browser mode
   - Color-coded results by type:
     - Google Search: Blue background (#28323C)
     - Bookmarks: Yellow-tinted background (#3C322B)
     - History: Purple-tinted background (#322832)
   - URL trimming for long query parameters (50 char limit)
   - Keyboard navigation (Up/Down/Home/End)
   - Scroll-to-selected functionality

3. **window_grid.rs** (Legacy)
   - Original grid implementation for main launcher

### Platform-Specific (`src/`)

1. **windows_api.rs**
   - Direct Windows API calls using winapi crate
   - Window enumeration and switching
   - Process name extraction
   - Taskbar window detection

2. **window_thumbnail.rs**
   - Thumbnail capture using Windows API
   - `ThumbnailCache` for performance
   - High-resolution capture support

### Applications

1. **main.rs**
   - Main launcher with dual search modes
   - Browser mode: Google + bookmarks + history search
   - Windows mode: Window switching with thumbnails
   - Adaptive UI: List view (Browser) vs Grid view (Windows)
   - Mode switching with Tab key
   - Japanese font support
   - Auto-focus on search input
   - Result type icons (🔍, ⭐, 🕒, 🪟)

## Key Features

### Search Modes
- **Browser Mode**: Integrated web search with browser data and Chrome tabs
- **Windows Mode**: Window switching by title/process name

### Search Behavior
- **Browser Mode**: 
  - Any text → Shows:
    1. Google search option (always first)
    2. Matching Chrome bookmarks (unlimited)
    3. Matching Chrome history (unlimited)
    4. Matching Chrome tabs (requires Chrome extension)
  - Empty query → No results
  - Searches in title and URL fields
  - Supports Japanese/international characters
  - Color-coded results for better visibility
  - History URLs with long query parameters are trimmed
  - **Debounced search**: 500ms delay after typing stops to reduce query load
  - **SQL-side filtering**: Searches happen at database level for better performance
  - **Progressive loading**: Shows 20 items initially, loads 10 more as you scroll
  - **Performance optimizations**:
    - Null/empty title filtering at SQL level
    - Configurable browser selection to reduce search scope
    - History limited to last 2 weeks (100 items max)
    - Sorted by visit count (DESC), then by last visit time (DESC)
    - Shows last visit time as relative time (e.g., "2 hours ago", "3 days ago")
- **Windows Mode**:
  - Empty query → Shows all windows
  - Text query → Filters windows by title, process, or class name
  - Case-insensitive matching
  - Limited to 10 results
  - **Instant search**: No debounce delay for responsive window switching

### Keyboard Shortcuts
- `Tab` - Switch between modes
- `↑/↓` - Navigate results
- `Enter` - Execute action
- `Esc` - Exit application

## Common Development Tasks

### Adding a new search type
1. Update `Action` enum in `search_engine.rs`
2. Add search logic to `DefaultSearchEngine::search()`
3. Add execution logic to `LauncherCore::execute_action()`
4. Update UI if needed in `main.rs`
5. Add tests

### Adding a new platform
1. Create platform-specific implementation of `WindowManager`
2. Add conditional compilation with `#[cfg(target_os = "...")]`
3. Update `Cargo.toml` with platform dependencies

### Debugging tips
- Use `env_logger` for runtime logging
- Set `RUST_LOG=debug` environment variable
- Use `cargo test -- --nocapture` for test output
- Check Windows API errors with `GetLastError()`

## Chrome Tab Search Setup

To enable Chrome tab search functionality:

1. **Build the launcher**: `cargo build --release --target x86_64-pc-windows-gnu --features sqlite`
2. **Load Chrome extension**: 
   - Open `chrome://extensions/`
   - Enable Developer mode
   - Load unpacked from `chrome-extension/` directory
3. **Run launcher**: The WebSocket server starts automatically on port 9999
4. **Test**: Start My Launcher and search for open Chrome tabs in Browser mode

### Tab Switching Features
- **Instant Response**: WebSocket provides <10ms latency (previously 500ms)
- **Visual Feedback**: Shows "Switching to tab: [title]" briefly
- **Auto-reconnect**: Extension automatically reconnects if connection drops
- **No Installation**: No Native Host registration required!

### Troubleshooting Tab Switching
1. **Check Chrome DevTools Console**: 
   - Look for "=== WEBSOCKET CONNECTED ===" message
   - Check for "=== TAB SWITCH EVENT RECEIVED ===" when switching
2. **Verify WebSocket Connection**: Should see "WebSocket: Attempting to connect to ws://localhost:9999"
3. **Check Launcher Logs**: Run with `RUST_LOG=debug` for detailed logs
4. **Common Issues**:
   - Port 9999 already in use
   - Extension not reloaded after changes
   - Firewall blocking localhost connections (rare)

### WebSocket Architecture Benefits
- **Performance**: 50x faster tab switching
- **Simplicity**: No complex Native Host setup
- **Reliability**: Automatic reconnection
- **Future-proof**: Easy to add more real-time features

## Known Issues & Workarounds

1. **Linux testing**: Some tests require `--features test-support`
2. **Cross-compilation**: Use MinGW target (`x86_64-pc-windows-gnu`) on Linux
3. **API compatibility**: eframe 0.24 doesn't have `frame.close()`, use `std::process::exit(0)`
4. **Chrome history**: Requires `--features sqlite` for history search
5. **Chrome profile detection**: Only Windows is currently supported
6. **Chrome database access**: Uses SQLite immutable mode to read Chrome's locked database files directly
7. **Chrome tab search**: Requires Chrome extension installation (no Native Messaging setup needed!)

## Project Structure
```
my-launcher/
├── src/
│   ├── core/
│   │   ├── mod.rs
│   │   ├── search_engine.rs
│   │   ├── browser_search_engine.rs
│   │   ├── window_manager.rs
│   │   └── launcher.rs
│   ├── data/
│   │   ├── mod.rs
│   │   ├── window_item.rs
│   │   ├── window_provider.rs
│   │   ├── browser_item.rs
│   │   └── browser_provider.rs
│   ├── filter/
│   │   ├── mod.rs
│   │   ├── window_filter.rs
│   │   └── search_filter.rs
│   ├── ui/
│   │   ├── mod.rs
│   │   ├── alt_tab_grid.rs
│   │   ├── browser_list.rs
│   │   └── window_grid.rs
│   ├── main.rs
│   ├── lib.rs
│   ├── windows_api.rs
│   ├── window_thumbnail.rs
│   ├── logger.rs
│   └── test_helpers.rs
├── tests/
│   └── integration_test.rs
├── Cargo.toml
├── .cargo/
│   └── config.toml
├── build.ps1
├── build.bat
├── test.ps1
├── test.bat
├── CLAUDE.md
└── README.md
```