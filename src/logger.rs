use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;
use chrono::Local;
use env_logger::Builder;

lazy_static::lazy_static! {
    static ref LOG_FILE: Mutex<Option<fs::File>> = Mutex::new(None);
}

pub fn init_logger() -> Result<(), Box<dyn std::error::Error>> {
    // Get executable directory
    let exe_path = std::env::current_exe()?;
    let exe_dir = exe_path.parent().ok_or("Failed to get executable directory")?;
    
    // Create logs directory
    let logs_dir = exe_dir.join("logs");
    fs::create_dir_all(&logs_dir)?;
    
    // Create log file with timestamp
    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
    let log_filename = format!("my_launcher_{}.log", timestamp);
    let log_path = logs_dir.join(log_filename);
    
    // Open log file
    let log_file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(&log_path)?;
    
    // Store file handle
    *LOG_FILE.lock().unwrap() = Some(log_file);
    
    // Initialize env_logger with custom format
    let mut builder = Builder::new();
    
    // Set log level from environment or default to debug
    let log_level = std::env::var("RUST_LOG")
        .unwrap_or_else(|_| "debug".to_string());
    builder.parse_filters(&log_level);
    
    // Custom format function
    builder.format(move |buf, record| {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        let level = record.level();
        let target = record.target();
        let args = record.args();
        
        // Write to stdout/stderr
        let _ = writeln!(buf, "[{}] {} {} - {}", timestamp, level, target, args);
        
        // Also write to file
        if let Ok(mut file_opt) = LOG_FILE.lock() {
            if let Some(ref mut file) = *file_opt {
                let _ = writeln!(file, "[{}] {} {} - {}", timestamp, level, target, args);
                let _ = file.flush();
            }
        }
        
        Ok(())
    });
    
    builder.init();
    
    // Log initialization info
    log::info!("Logger initialized. Log file: {}", log_path.display());
    log::info!("Executable directory: {}", exe_dir.display());
    
    // Also print to stderr for debugging
    eprintln!("Log file created at: {}", log_path.display());
    
    Ok(())
}

pub fn get_log_directory() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let exe_path = std::env::current_exe()?;
    let exe_dir = exe_path.parent().ok_or("Failed to get executable directory")?;
    Ok(exe_dir.join("logs"))
}