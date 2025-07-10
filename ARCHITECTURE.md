# My Launcher Architecture Documentation

## Overview

My Launcher is a Windows launcher application built with Rust, designed to be a customizable alternative to Alfred, Raycast, and Flow Launcher. The architecture emphasizes modularity, testability, and performance.

## Design Principles

1. **Separation of Concerns**: Business logic is separated from UI and platform-specific code
2. **Dependency Injection**: Core components use trait-based abstractions for testability
3. **Platform Abstraction**: Windows-specific code is isolated behind traits
4. **Performance First**: Uses native Windows APIs and efficient data structures

## Component Architecture

```
┌────────────────────────────────────────────────────────────┐
│                    UI Layer (egui)                         │
│  ┌──────────┐ ┌──────────────────┐ ┌──────────────────┐ │
│  │ main.rs  │ │ ui/alt_tab_grid  │ │ ui/browser_list  │ │
│  └──────────┘ └──────────────────┘ └──────────────────┘ │
└────────────────────────┬───────────────────────────────────┘
                         │
┌────────────────────────▼───────────────────────────────────┐
│                    Filter Layer                            │
│  ┌──────────────────┐ ┌──────────────┐ ┌──────────────┐  │
│  │ TaskbarFilter    │ │ SearchFilter │ │ Composable   │  │
│  └──────────────────┘ └──────────────┘ └──────────────┘  │
└────────────────────────┬───────────────────────────────────┘
                         │
┌────────────────────────▼───────────────────────────────────┐
│                    Data Layer                              │
│  ┌──────────────────┐ ┌────────────────────────────────┐  │
│  │ WindowProvider   │ │ WindowItem (Searchable)       │  │
│  └──────────────────┘ └────────────────────────────────┘  │
└────────────────────────┬───────────────────────────────────┘
                         │
┌────────────────────────▼───────────────────────────────────┐
│                    Core Layer                              │
│  ┌─────────────┐ ┌──────────────┐ ┌─────────────────────┐ │
│  │LauncherCore │ │SearchEngine  │ │WindowManager        │ │
│  └─────────────┘ └──────────────┘ └─────────────────────┘ │
└────────────────────────┬───────────────────────────────────┘
                         │
┌────────────────────────▼───────────────────────────────────┐
│              Platform Layer (Windows)                      │
│  ┌─────────────┐ ┌──────────────────┐ ┌────────────────┐  │
│  │windows_api  │ │window_thumbnail  │ │ Logger         │  │
│  └─────────────┘ └──────────────────┘ └────────────────┘  │
└────────────────────────────────────────────────────────────┘
```

## Core Components

### LauncherCore (`src/core/launcher.rs`)

The central orchestrator that combines search and window management functionality.

```rust
pub struct LauncherCore<S: SearchEngine, W: WindowManager> {
    search_engine: S,
    window_manager: Arc<W>,
    cached_windows: Vec<WindowInfo>,
}
```

**Responsibilities:**
- Manages the lifecycle of search operations
- Caches window information for performance
- Executes actions (URL opening, command execution, window switching)
- Provides a unified interface for the UI layer

**Key Methods:**
- `search()`: Delegates to SearchEngine with current mode and cached windows
- `execute_action()`: Handles all action types uniformly
- `refresh_windows()`: Updates the window cache

### SearchEngine (`src/core/search_engine.rs`)

Trait-based abstraction for search functionality.

```rust
pub trait SearchEngine {
    fn search(&self, query: &str, mode: SearchMode, windows: &[WindowInfo]) -> Vec<SearchResult>;
    fn is_window_search(&self, query: &str, mode: SearchMode) -> bool;
}
```

**Implementations:**
1. **DefaultSearchEngine**: Simple implementation (deprecated)
2. **BrowserSearchEngine**: Full-featured search with browser data integration

**Search Modes:**
1. **Browser Mode**: 
   - Integrates multiple search sources:
     - Google search (always first)
     - Chrome bookmarks (up to 5 results)
     - Chrome history (up to 5 results)
   - Supports Japanese and international characters
2. **Windows Mode**:
   - Searches through cached window information
   - Filters by window title, process name, or class name
   - Case-insensitive matching
   - Returns up to 10 results

**Search Algorithm:**
```
Browser Mode:
1. If empty query: return empty
2. Otherwise:
   a. Add Google search result first
   b. Search bookmarks (title, URL) - max 5 results
   c. Search history (title, URL) - max 5 results

Windows Mode:
1. If empty query: return all windows
2. Otherwise: filter windows by title, process name, or class name
   - Case-insensitive matching
   - Limit to 10 results
```

**Action Types:**
```rust
pub enum Action {
    SwitchWindow(isize),     // Switch to window by handle
    GoogleSearch(String),    // Open Google search with query
    OpenBookmark(String),    // Open bookmark URL
    OpenHistory(String),     // Open history URL
}
```

**Result Types:**
```rust
pub enum ResultType {
    GoogleSearch,
    Bookmark,
    History,
    Window,
}
```

### BrowserSearchEngine (`src/core/browser_search_engine.rs`)

Advanced search engine implementation with browser data integration.

```rust
pub struct BrowserSearchEngine {
    browser_provider: Arc<Mutex<CachedBrowserProvider>>,
}
```

**Features:**
- Integrates Chrome bookmarks and history
- Caches browser data for performance
- Graceful fallback when Chrome data is unavailable
- Thread-safe with Arc<Mutex<>>

### WindowManager (`src/core/window_manager.rs`)

Platform abstraction for window operations.

```rust
pub trait WindowManager: Send + Sync {
    fn enumerate_windows(&self) -> Vec<WindowInfo>;
    fn switch_to_window(&self, hwnd: isize);
}
```

**WindowInfo Structure:**
```rust
pub struct WindowInfo {
    pub hwnd: isize,           // Window handle
    pub title: String,         // Window title
    pub class_name: String,    // Window class (e.g., "Chrome_WidgetWin_1")
    pub process_name: String,  // Process name (e.g., "firefox.exe")
    pub is_visible: bool,      // Visibility state
    pub is_minimized: bool,    // Minimized state
    pub rect: (i32, i32, i32, i32), // Position and size
}
```

## Data Layer Components

### WindowProvider (`src/data/window_provider.rs`)

Abstraction for window data sources, separating data fetching from UI.

```rust
pub trait WindowProvider {
    fn get_windows(&self) -> Vec<WindowItem>;
    fn refresh(&mut self);
    fn focus_window(&self, hwnd: isize) -> Result<(), String>;
}
```

**WindowsApiProvider Implementation:**
- Caches window information for performance
- Converts between `WindowInfo` and `WindowItem` types
- Provides a consistent interface for UI components

### WindowItem (`src/data/window_item.rs`)

Data model for windows with UI and search support.

```rust
pub struct WindowItem {
    pub hwnd: isize,
    pub title: String,
    pub process_name: String,
    pub class_name: String,
    pub is_visible: bool,
    pub is_minimized: bool,
    pub rect: (i32, i32, i32, i32),
}
```

**Trait Implementations:**
- `GridItem`: For UI display in grid layouts
- `Searchable`: For text-based filtering

### BrowserProvider (`src/data/browser_provider.rs`)

Abstraction for browser data sources (bookmarks, history).

```rust
pub trait BrowserDataProvider: Send + Sync {
    fn get_bookmarks(&self) -> Result<Vec<BookmarkItem>, Box<dyn Error>>;
    fn get_history(&self) -> Result<Vec<HistoryItem>, Box<dyn Error>>;
}
```

**ChromeBrowserProvider Implementation:**
- Reads Chrome bookmarks from JSON file
- Reads Chrome history from SQLite database (optional feature)
- Auto-detects Chrome profile location on Windows
- Uses SQLite immutable mode to read locked Chrome databases directly
- No file copying needed for history access

**CachedBrowserProvider:**
- Wraps any BrowserDataProvider with caching
- Uses Mutex-based thread-safe caching
- Improves performance for repeated searches
- Refreshable cache for dynamic updates

### BrowserItem (`src/data/browser_item.rs`)

Data models for browser-related items.

```rust
pub struct BookmarkItem {
    pub title: String,
    pub url: String,
    pub folder: Option<String>,
}

pub struct HistoryItem {
    pub title: String,
    pub url: String,
    pub visit_count: i32,
    pub last_visit_time: i64,
}
```

**Features:**
- Both implement `Searchable` trait
- Supports hierarchical bookmark folders
- Includes visit statistics for history items

## Filter Layer Components

### WindowFilter (`src/filter/window_filter.rs`)

Window-specific filtering abstraction.

```rust
pub trait WindowFilter {
    fn matches(&self, window: &WindowItem) -> bool;
    fn name(&self) -> &str;
}
```

**TaskbarWindowFilter:**
- Uses Windows API to determine taskbar visibility
- Checks `WS_EX_APPWINDOW` and `WS_EX_TOOLWINDOW` flags
- Filters based on owner window relationships

**CompositeFilter:**
- Combines multiple filters with AND/OR logic
- Enables complex filtering scenarios

### SearchFilter (`src/filter/search_filter.rs`)

Generic text-based search functionality.

```rust
pub trait Searchable {
    fn search_fields(&self) -> Vec<(&str, &str)>;
}

pub struct SearchFilter {
    query: String,
    target_fields: Vec<String>,
}
```

**Features:**
- Field-specific or all-field search
- Case-insensitive matching
- Extensible to any `Searchable` type

## Platform-Specific Implementation

### Windows API Integration (`src/windows_api.rs`)

Direct Windows API calls using the `winapi` crate.

**Key Functions:**
1. **enumerate_windows()**: 
   - Uses `EnumWindows` to iterate all top-level windows
   - Filters out invisible windows and system windows
   - Extracts process information using `GetWindowThreadProcessId`
   - Gets window bounds using DWM API

2. **switch_to_window()**:
   - Restores minimized windows with `ShowWindow`
   - Brings window to foreground with `SetForegroundWindow`

**Filtering Logic:**
- Uses `is_taskbar_window()` function to filter only taskbar-visible windows
- Checks for `WS_VISIBLE` style
- Excludes windows with `WS_EX_TOOLWINDOW` style
- Requires either `WS_EX_APPWINDOW` style or no owner window
- Excludes windows with empty titles

### Window Thumbnail Capture (`src/window_thumbnail.rs`)

Captures and caches window thumbnails for visual preview.

**Implementation Details:**
1. **Capture Process**:
   - Get window device context with `GetWindowDC`
   - Create compatible bitmap and device context
   - Copy window contents with `BitBlt`
   - Convert from BGRA to RGBA format
   - Scale to target size (80x60 by default)

2. **Caching Strategy**:
   - Thumbnails cached by window handle
   - Cache cleared on demand
   - Lazy loading on first access

## UI Layer

### Main Launcher (`src/main.rs`)

Main launcher with dual search modes and browser integration.

```rust
struct LauncherApp {
    input_text: String,
    mode: SearchMode,
    core: LauncherCore<BrowserSearchEngine, WindowsApiManager>,
    search_results: Vec<SearchResult>,
    grid: AltTabGrid,
    browser_list: BrowserList,
    thumbnail_cache: ThumbnailCache,
    first_frame: bool,
}
```

**Features:**
- Mode switching (Browser/Windows)
- Browser integration (Google search, bookmarks, history)
- Window search and switching with thumbnails
- Adaptive UI: Browser mode uses BrowserList component, Windows mode uses grid
- Japanese font support
- Auto-focus on search input
- Icons for different result types (🔍, ⭐, 🕒, 🪟)
- Color-coded results in Browser mode for better visibility
- URL trimming for long query parameters in history


### UI Components (`src/ui/`)

**AltTabGrid:**
- Grid-based window display
- Keyboard navigation (arrows, Home/End)
- Mouse interaction
- Thumbnail integration
- Configurable layout (columns, spacing)

**BrowserList:**
- List-based display for browser search results
- Color-coded results by type:
  - Google Search: Blue background (#28323C, selected: #325A5A)
  - Bookmarks: Yellow-tinted (#3C322B, selected: #5A4632)
  - History: Purple-tinted (#322832, selected: #463246)
  - Windows: Gray (#282828, selected: #3C3C3C)
- URL trimming for long query parameters (max 50 chars)
- Keyboard navigation (Up/Down/Home/End)
- Scroll-to-selected functionality
- Separates UI concerns from main application logic

**GridItem Trait:**
```rust
pub trait GridItem {
    fn title(&self) -> &str;
    fn description(&self) -> &str;
    fn hwnd(&self) -> isize;
    fn id(&self) -> String;
}
```

## Testing Strategy

### Unit Tests
Located alongside implementation in `src/core/*.rs`:
- Test search logic with various input patterns
- Test window filtering and matching
- Test action execution (mocked)

### Integration Tests
Located in `tests/integration_test.rs`:
- End-to-end workflows
- Mock window manager for predictable testing
- Requires `test-support` feature flag

### Mock Implementation
```rust
pub struct MockWindowManager {
    windows: Arc<Mutex<Vec<WindowInfo>>>,
    switched_to: Arc<Mutex<Option<isize>>>,
}
```

Allows testing without Windows API dependencies.

## Performance Considerations

1. **Window Caching**: Windows enumerated once and cached
2. **Thumbnail Caching**: Generated once per window
3. **Search Limiting**: Results capped at 10 for performance
4. **Immediate Mode GUI**: Efficient rendering with egui

## Recent Additions

### Multi-Layer Architecture
- **Filter Layer**: Separates filtering logic from data and UI
- **Data Layer**: Abstracts data sources from business logic
- **Improved UI Components**: Modular grid system for different display styles

### Browser Integration
- Chrome bookmarks and history search
- SQLite immutable mode for safe database access
- Cached browser data for performance
- Color-coded search results by type

### Extensible Filtering System
- `WindowFilter` trait for window-specific filters
- `Searchable` trait for generic search
- Composable filters with AND/OR logic
- Easy addition of new filter types

## Future Extension Points

1. **Search Providers**: Add new search engines by implementing `SearchEngine` trait
2. **Platforms**: Add macOS/Linux support with new `WindowManager` implementations
3. **Actions**: Extend `Action` enum for new capabilities
4. **UI Themes**: Leverage egui's theming system
5. **Plugins**: Could add plugin system using dynamic loading
6. **Additional Filters**: Process-based, time-based, or usage-based filtering
7. **Search Targets**: Extend beyond windows to files, applications, settings
8. **Customizable Layouts**: User-defined grid sizes and thumbnail resolutions

## Configuration and State

Currently, the application has no persistent configuration. Future versions could add:
- Search history
- Favorite windows
- Custom shortcuts
- Theme preferences

## Security Considerations

1. **Process Enumeration**: Requires appropriate Windows permissions
2. **Command Execution**: Commands run with user privileges
3. **URL Opening**: Delegated to system default handler
4. **No Network Access**: Except for opening URLs in browser
5. **Chrome Data Access**: Read-only access to user's Chrome data (bookmarks, history)
6. **Database Access**: Uses SQLite immutable mode for safe concurrent access

## Dependencies

### Core Dependencies
- `egui`/`eframe`: GUI framework
- `winapi`: Windows API bindings
- `open`: Cross-platform file/URL opening
- `urlencoding`: URL encoding for searches
- `serde`/`serde_json`: JSON parsing for Chrome bookmarks
- `url`: URL parsing utilities
- `rusqlite`: SQLite for Chrome history (optional)
- `chrono`: Timestamp handling
- `lazy_static`: Static initialization

### Dev Dependencies
- `mockall`: Mocking framework
- `tempfile`: Temporary file handling in tests

## Build Configuration

### Cargo Features
- `test-support`: Exposes mock implementations for integration tests
- `sqlite`: Enables Chrome history search via SQLite

### Target Configuration
- Default: Native platform
- Cross-compilation: `x86_64-pc-windows-gnu` for Linux→Windows

## Common Patterns

### Error Handling
- Most operations use `Result` types
- Windows API errors logged but not surfaced to user
- Graceful degradation (e.g., missing thumbnails)

### Resource Management
- RAII pattern for Windows handles
- Automatic cleanup via Drop implementations
- No manual memory management needed