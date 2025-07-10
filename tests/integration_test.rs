#![cfg(feature = "test-support")]

use my_launcher::core::{
    LauncherCore, SearchEngine, SearchMode, WindowInfo,
    search_engine::DefaultSearchEngine,
    window_manager::mock::MockWindowManager,
};
use std::sync::Arc;

fn setup_test_launcher() -> (
    LauncherCore<DefaultSearchEngine, MockWindowManager>,
    Arc<MockWindowManager>,
) {
    let test_windows = vec![
        WindowInfo {
            hwnd: 100,
            title: "Visual Studio Code - my_project".to_string(),
            class_name: "Chrome_WidgetWin_1".to_string(),
            process_name: "Code.exe".to_string(),
            is_visible: true,
            is_minimized: false,
            rect: (0, 0, 1920, 1080),
        },
        WindowInfo {
            hwnd: 200,
            title: "Mozilla Firefox".to_string(),
            class_name: "MozillaWindowClass".to_string(),
            process_name: "firefox.exe".to_string(),
            is_visible: true,
            is_minimized: false,
            rect: (100, 100, 1600, 900),
        },
        WindowInfo {
            hwnd: 300,
            title: "Command Prompt".to_string(),
            class_name: "ConsoleWindowClass".to_string(),
            process_name: "cmd.exe".to_string(),
            is_visible: true,
            is_minimized: true,
            rect: (200, 200, 800, 600),
        },
    ];

    let window_manager = Arc::new(MockWindowManager::new(test_windows));
    let search_engine = DefaultSearchEngine::new();
    let launcher = LauncherCore::new(search_engine, window_manager.clone());

    (launcher, window_manager)
}

#[test]
fn test_window_switching_workflow() {
    let (launcher, window_manager) = setup_test_launcher();
    
    // Search for Firefox
    let results = launcher.search("firefox", SearchMode::Windows);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title, "Mozilla Firefox");
    
    // Execute switch action
    launcher.execute_action(&results[0].action);
    
    // Verify window was switched
    assert_eq!(window_manager.get_switched_window(), Some(200));
}

#[test]
fn test_mixed_search_workflow() {
    let (launcher, _) = setup_test_launcher();
    
    // Test URL search
    let url_results = launcher.search("https://github.com", SearchMode::General);
    assert!(url_results.iter().any(|r| matches!(&r.action, 
        my_launcher::core::Action::OpenUrl(url) if url == "https://github.com"
    )));
    
    // Test command search
    let cmd_results = launcher.search(">dir", SearchMode::General);
    assert!(cmd_results.iter().any(|r| matches!(&r.action,
        my_launcher::core::Action::RunCommand(cmd) if cmd == "dir"
    )));
    
    // Test window search with prefix
    let win_results = launcher.search("w code", SearchMode::General);
    assert_eq!(win_results.len(), 1);
    assert!(win_results[0].title.contains("Visual Studio Code"));
}

#[test]
fn test_case_insensitive_window_search() {
    let (launcher, _) = setup_test_launcher();
    
    // Should find "Command Prompt" with various cases
    let test_cases = vec!["COMMAND", "command", "CoMmAnD", "prompt", "PROMPT"];
    
    for query in test_cases {
        let results = launcher.search(query, SearchMode::Windows);
        assert!(
            results.iter().any(|r| r.title == "Command Prompt"),
            "Failed to find 'Command Prompt' with query '{}'",
            query
        );
    }
}

#[test]
fn test_search_by_process_name() {
    let (launcher, _) = setup_test_launcher();
    
    // Search by process name
    let results = launcher.search("Code.exe", SearchMode::Windows);
    assert_eq!(results.len(), 1);
    assert!(results[0].title.contains("Visual Studio Code"));
    
    // Search by partial process name
    let results = launcher.search(".exe", SearchMode::Windows);
    assert_eq!(results.len(), 3); // All windows have .exe processes
}

#[test]
fn test_empty_search_behavior() {
    let (launcher, _) = setup_test_launcher();
    
    // Empty search in General mode should return nothing
    let general_results = launcher.search("", SearchMode::General);
    assert!(general_results.is_empty());
    
    // Empty search in Windows mode should return all windows
    let window_results = launcher.search("", SearchMode::Windows);
    assert_eq!(window_results.len(), 3);
}

#[test]
fn test_window_refresh() {
    let (mut launcher, window_manager) = setup_test_launcher();
    
    // Initial window count
    assert_eq!(launcher.get_cached_windows().len(), 3);
    
    // Update windows in mock
    window_manager.set_windows(vec![
        WindowInfo {
            hwnd: 400,
            title: "New Window".to_string(),
            class_name: "NewClass".to_string(),
            process_name: "new.exe".to_string(),
            is_visible: true,
            is_minimized: false,
            rect: (0, 0, 800, 600),
        },
    ]);
    
    // Refresh launcher
    launcher.refresh_windows();
    
    // Verify new window list
    assert_eq!(launcher.get_cached_windows().len(), 1);
    assert_eq!(launcher.get_cached_windows()[0].title, "New Window");
}

#[test]
fn test_google_search_fallback() {
    let (launcher, _) = setup_test_launcher();
    
    // Any non-special query should include Google search
    let results = launcher.search("rust programming language", SearchMode::General);
    
    let google_result = results.iter().find(|r| matches!(&r.action,
        my_launcher::core::Action::OpenUrl(url) if url.contains("google.com/search")
    ));
    
    assert!(google_result.is_some());
    if let my_launcher::core::Action::OpenUrl(url) = &google_result.unwrap().action {
        assert!(url.contains("rust+programming+language"));
    }
}