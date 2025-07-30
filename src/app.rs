use std::path::PathBuf;
use rfd::FileDialog;
use anyhow::Result;
use crate::types::{AvailableFile, FileType, FileAction};
use crate::config::AppConfig;
use crate::ucl_bindings::UclLibrary;
use crate::file_ops::{scan_psdz_files, generate_output_filename, get_program_directory, process_files};
use crate::ui::UIState;

pub struct BMWVirtualReaderApp {
    pub btld_file: Option<PathBuf>,
    pub swfl1_file: Option<PathBuf>,
    pub swfl2_file: Option<PathBuf>,
    pub output_file: Option<PathBuf>,
    pub status_message: String,
    pub is_processing: bool,
    pub ucl_library: Option<UclLibrary>,
    pub config: AppConfig,
    pub psdz_folder: Option<PathBuf>,
    pub available_files: Vec<AvailableFile>,
    pub ui_state: UIState,
}

impl Default for BMWVirtualReaderApp {
    fn default() -> Self {
        Self {
            btld_file: None,
            swfl1_file: None,
            swfl2_file: None,
            output_file: None,
            status_message: "Ready".to_string(),
            is_processing: false,
            ucl_library: None,
            config: AppConfig::load(),
            psdz_folder: None,
            available_files: Vec::new(),
            ui_state: UIState::default(),
        }
    }
}

impl BMWVirtualReaderApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut app = Self::default();
        
        // Try to load the UCL library
        if let Ok(lib) = UclLibrary::new(&app.config.ucl_library_path) {
            app.ucl_library = Some(lib);
            app.status_message = "UCL library loaded successfully".to_string();
        } else {
            app.status_message = format!("Warning: Could not load UCL library from {}", app.config.ucl_library_path);
        }
        
        app
    }

    pub fn select_psdz_folder(&mut self) {
        let mut dialog = FileDialog::new()
            .add_filter("Directories", &["*"]);
        
        if let Some(ref last_dir) = self.config.last_input_dir {
            dialog = dialog.set_directory(last_dir);
        }
        
        if let Some(path) = dialog.pick_folder() {
            self.psdz_folder = Some(path.clone());
            self.scan_psdz_files(&path);
            
            // Update config
            self.config.last_input_dir = Some(path.to_string_lossy().to_string());
        }
    }

    pub fn scan_psdz_files(&mut self, psdz_path: &PathBuf) {
        self.available_files.clear();
        self.status_message = "Scanning PSDZ files...".to_string();
        
        self.available_files = scan_psdz_files(psdz_path);
        
        self.status_message = format!("Found {} files ({} BTLD, {} SWFL)", 
            self.available_files.len(),
            self.available_files.iter().filter(|f| f.file_type == FileType::BTLD).count(),
            self.available_files.iter().filter(|f| f.file_type == FileType::SWFL).count());
    }

    pub fn select_file_by_index(&mut self, index: usize, file_type: &str) {
        if index < self.available_files.len() {
            let file = &self.available_files[index];
            match file_type {
                "btld" => {
                    self.btld_file = Some(file.path.clone());
                    self.ui_state.selected_btld_index = Some(index);
                    
                    // Auto-generate output file path if not set
                    if self.output_file.is_none() {
                        if let Some(file_name) = file.path.file_name() {
                            let file_name_str = file_name.to_string_lossy();
                            let output_file_name = file_name_str.replace(".bin", ".extracted");
                            let mut output_path = file.path.clone();
                            output_path.set_file_name(output_file_name);
                            self.output_file = Some(output_path);
                        }
                    }
                }
                "swfl1" => {
                    self.swfl1_file = Some(file.path.clone());
                    self.ui_state.selected_swfl1_index = Some(index);
                    
                    // Auto-generate output file path based on SWFL1
                    if let Some(output_filename) = generate_output_filename(&file.path) {
                        let mut output_path = get_program_directory();
                        output_path.push(output_filename);
                        self.output_file = Some(output_path);
                    }
                }
                "swfl2" => {
                    self.swfl2_file = Some(file.path.clone());
                    self.ui_state.selected_swfl2_index = Some(index);
                }
                _ => {}
            }
        }
    }

    pub fn clear_file_selection(&mut self, file_type: &str) {
        match file_type {
            "btld" => {
                self.btld_file = None;
                self.ui_state.selected_btld_index = None;
            }
            "swfl1" => {
                self.swfl1_file = None;
                self.ui_state.selected_swfl1_index = None;
            }
            "swfl2" => {
                self.swfl2_file = None;
                self.ui_state.selected_swfl2_index = None;
            }
            _ => {}
        }
    }

    pub fn select_btld_file(&mut self) {
        let mut dialog = FileDialog::new()
            .add_filter("All files", &["*"]);
        
        if let Some(ref last_dir) = self.config.last_input_dir {
            dialog = dialog.set_directory(last_dir);
        }
        
        if let Some(path) = dialog.pick_file() {
            self.btld_file = Some(path.clone());
            
            // Auto-generate output file path if not set and no SWFL1 selected
            if self.output_file.is_none() && self.swfl1_file.is_none() {
                if let Some(file_name) = path.file_name() {
                    let file_name_str = file_name.to_string_lossy();
                    // Replace .bin with .extracted in the filename
                    let output_file_name = file_name_str.replace(".bin", ".extracted");
                    let mut output_path = path.clone();
                    output_path.set_file_name(output_file_name);
                    self.output_file = Some(output_path);
                }
            }
            
            // Update config
            if let Some(ref output_path) = self.output_file {
                self.config.update_directories(&path, output_path);
            }
        }
    }

    pub fn select_swfl1_file(&mut self) {
        let mut dialog = FileDialog::new()
            .add_filter("All files", &["*"]);
        
        if let Some(ref last_dir) = self.config.last_input_dir {
            dialog = dialog.set_directory(last_dir);
        }
        
        if let Some(path) = dialog.pick_file() {
            self.swfl1_file = Some(path.clone());
            
            // Auto-generate output file path based on SWFL1
            if let Some(output_filename) = generate_output_filename(&path) {
                let mut output_path = get_program_directory();
                output_path.push(output_filename);
                self.output_file = Some(output_path);
            }
            
            // Update config
            self.config.last_input_dir = path.parent().map(|p| p.to_string_lossy().to_string());
        }
    }

    pub fn select_swfl2_file(&mut self) {
        let mut dialog = FileDialog::new()
            .add_filter("All files", &["*"]);
        
        if let Some(ref last_dir) = self.config.last_input_dir {
            dialog = dialog.set_directory(last_dir);
        }
        
        if let Some(path) = dialog.pick_file() {
            self.swfl2_file = Some(path.clone());
            
            // Update config
            self.config.last_input_dir = path.parent().map(|p| p.to_string_lossy().to_string());
        }
    }

    pub fn select_output_file(&mut self) {
        let mut dialog = FileDialog::new()
            .add_filter("All files", &["*"]);
        
        if let Some(ref last_dir) = self.config.last_output_dir {
            dialog = dialog.set_directory(last_dir);
        }
        
        if let Some(path) = dialog.save_file() {
            self.output_file = Some(path.clone());
            
            // Update config
            if let Some(ref btld_path) = self.btld_file {
                self.config.update_directories(btld_path, &path);
            }
        }
    }

    pub fn process_files(&mut self) -> Result<()> {
        self.is_processing = true;
        self.status_message = "Processing...".to_string();
        
        let output_path = self.output_file.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No output file selected"))?
            .clone();
        
        if let Some(ref ucl_lib) = self.ucl_library {
            process_files(
                self.btld_file.as_ref(),
                self.swfl1_file.as_ref(),
                self.swfl2_file.as_ref(),
                &output_path,
                ucl_lib,
                &mut |status| self.status_message = status.to_string()
            )?;
        } else {
            return Err(anyhow::anyhow!("UCL library not loaded"));
        }
        
        self.is_processing = false;
        Ok(())
    }

    pub fn reload_ucl_library(&mut self) {
        self.ucl_library = None;
        
        if let Ok(lib) = UclLibrary::new(&self.config.ucl_library_path) {
            self.ucl_library = Some(lib);
            self.status_message = "UCL library reloaded successfully".to_string();
        } else {
            self.status_message = format!("Failed to load UCL library from {}", self.config.ucl_library_path);
        }
    }

    pub fn handle_file_action(&mut self, action: FileAction) {
        match action {
            FileAction::Clear(file_type) => self.clear_file_selection(&file_type),
            FileAction::SelectBTLD(index) => self.select_file_by_index(index, "btld"),
            FileAction::SelectSWFL1(index) => self.select_file_by_index(index, "swfl1"),
            FileAction::SelectSWFL2(index) => self.select_file_by_index(index, "swfl2"),
        }
    }
} 