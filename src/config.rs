use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub last_input_dir: Option<String>,
    pub last_output_dir: Option<String>,
    pub window_width: f32,
    pub window_height: f32,
    pub ucl_library_path: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            last_input_dir: None,
            last_output_dir: None,
            window_width: 600.0,
            window_height: 400.0,
            ucl_library_path: Self::get_default_dll_path(),
        }
    }
}

impl AppConfig {
    pub fn load() -> Self {
        if let Ok(config_str) = fs::read_to_string("config.json") {
            if let Ok(config) = serde_json::from_str(&config_str) {
                return config;
            }
        }
        Self::default()
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_str = serde_json::to_string_pretty(self)?;
        fs::write("config.json", config_str)?;
        Ok(())
    }

    pub fn update_directories(&mut self, input_path: &PathBuf, output_path: &PathBuf) {
        if let Some(parent) = input_path.parent() {
            self.last_input_dir = Some(parent.to_string_lossy().to_string());
        }
        if let Some(parent) = output_path.parent() {
            self.last_output_dir = Some(parent.to_string_lossy().to_string());
        }
    }

    /// Get the default DLL path based on the current executable location
    fn get_default_dll_path() -> String {
        // Try to get the executable directory
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                // Check if we're in portable mode (DLL is in the same directory as exe)
                let portable_dll = exe_dir.join("libucl-1.dll");
                if portable_dll.exists() {
                    return portable_dll.to_string_lossy().to_string();
                }
                
                // Check if we're in development mode (DLL is in lib subdirectory)
                let dev_dll = exe_dir.join("lib").join("libucl-1.dll");
                if dev_dll.exists() {
                    return dev_dll.to_string_lossy().to_string();
                }
            }
        }
        
        // Fallback to relative path for development
        "lib/libucl-1.dll".to_string()
    }
} 