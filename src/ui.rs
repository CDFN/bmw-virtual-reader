use eframe::egui;
use std::path::PathBuf;
use webbrowser;
use crate::types::{AvailableFile, FileType, UIMessage};

pub struct UIState {
    pub show_settings: bool,
    pub show_file_browser: bool,
    pub file_search_filter: String,
    pub selected_btld_index: Option<usize>,
    pub selected_swfl1_index: Option<usize>,
    pub selected_swfl2_index: Option<usize>,
    pub message_queue: Vec<UIMessage>,
}

impl Default for UIState {
    fn default() -> Self {
        Self {
            show_settings: false,
            show_file_browser: false,
            file_search_filter: String::new(),
            selected_btld_index: None,
            selected_swfl1_index: None,
            selected_swfl2_index: None,
            message_queue: Vec::new(),
        }
    }
}

pub fn render_header(ui: &mut egui::Ui, show_settings: &mut bool) {
    ui.horizontal(|ui| {
        ui.heading(egui::RichText::new("BMW Virtual Reader")
            .size(24.0)
            .color(egui::Color32::from_rgb(180, 160, 100)));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button(egui::RichText::new("Settings")
                .color(egui::Color32::from_rgb(220, 220, 220)))
                .clicked() {
                *show_settings = !*show_settings;
            }
            // put link to github below settings button
            if ui.link(egui::RichText::new("github.com/CDFN/bmw-virtual-reader")
                .color(egui::Color32::from_rgb(100, 150, 255))
                .size(12.0))
                .clicked() {
                let _ = webbrowser::open("https://github.com/CDFN/bmw-virtual-reader");
            }
        });
    });
}

pub fn render_psdz_section(
    ui: &mut egui::Ui,
    psdz_folder: &Option<PathBuf>,
    message_queue: &mut Vec<UIMessage>
) {
    ui.group(|ui| {
        ui.heading(egui::RichText::new("PSDZ Data Source")
            .size(18.0)
            .color(egui::Color32::from_rgb(120, 160, 200)));
        
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Folder:")
                .color(egui::Color32::from_rgb(180, 180, 180)));
            if let Some(ref path) = psdz_folder {
                ui.label(egui::RichText::new(path.to_string_lossy())
                    .color(egui::Color32::from_rgb(140, 200, 140)));
            } else {
                ui.label(egui::RichText::new("No folder selected")
                    .color(egui::Color32::from_rgb(200, 140, 140)));
            }
        });
        
        ui.horizontal(|ui| {
            if ui.button(egui::RichText::new("Browse Folder")
                .color(egui::Color32::from_rgb(220, 220, 220)))
                .clicked() {
                message_queue.push(UIMessage::SelectPSDZFolder);
            }
            if ui.button(egui::RichText::new("File Browser")
                .color(egui::Color32::from_rgb(220, 220, 220)))
                .clicked() {
                message_queue.push(UIMessage::ToggleFileBrowser);
            }
        });
    });
}

pub fn render_file_browser(
    ctx: &egui::Context,
    show_file_browser: &mut bool,
    available_files: &[AvailableFile],
    file_search_filter: &mut String,
    selected_btld_index: &Option<usize>,
    selected_swfl1_index: &Option<usize>,
    selected_swfl2_index: &Option<usize>,
    message_queue: &mut Vec<UIMessage>
) {
    if *show_file_browser && !available_files.is_empty() {
        egui::Window::new("PSDZ File Browser")
            .open(show_file_browser)
            .default_size([700.0, 500.0])
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Search:")
                        .color(egui::Color32::from_rgb(180, 180, 180)));
                    ui.text_edit_singleline(file_search_filter);
                });
                
                ui.add_space(10.0);
                
                // File list
                egui::ScrollArea::vertical().show(ui, |ui| {
                    let filter_text = file_search_filter.to_lowercase();
                    
                    for (index, file) in available_files.iter().enumerate() {
                        // Since display names now have _ instead of .bin., we can simplify the search
                        let display_name_normalized = file.display_name.to_lowercase();
                        
                        // Create search patterns for different formats
                        let search_patterns = vec![
                            filter_text.clone(), // Exact match
                            filter_text.replace("-", "_"), // Replace hyphens with underscores
                            filter_text.replace("_", "-"), // Replace underscores with hyphens
                        ];
                        
                        // Check if any pattern matches against the display name
                        let matches = if !filter_text.is_empty() {
                            search_patterns.iter().any(|pattern| {
                                display_name_normalized.contains(pattern)
                            })
                        } else {
                            true
                        };
                        
                        if !matches {
                            continue;
                        }
                        
                        let is_selected_btld = *selected_btld_index == Some(index);
                        let is_selected_swfl1 = *selected_swfl1_index == Some(index);
                        let is_selected_swfl2 = *selected_swfl2_index == Some(index);
                        
                        let file_type_str = match file.file_type {
                            FileType::BTLD => "BTLD",
                            FileType::SWFL => "SWFL",
                        };
                        
                        let size_mb = file.size as f64 / (1024.0 * 1024.0);
                        
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                ui.vertical(|ui| {
                                    ui.label(egui::RichText::new(&file.display_name)
                                        .size(16.0)
                                        .color(egui::Color32::from_rgb(220, 220, 180)));
                                    ui.label(egui::RichText::new(format!("Type: {} | Size: {:.1} MB", file_type_str, size_mb))
                                        .color(egui::Color32::from_rgb(160, 160, 160))
                                        .size(12.0));
                                });
                            
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if file.file_type == FileType::BTLD {
                                        if is_selected_btld {
                                            if ui.button(egui::RichText::new("[SELECTED] BTLD")
                                                .color(egui::Color32::from_rgb(120, 200, 120)))
                                                .clicked() {
                                                message_queue.push(UIMessage::ClearFile("btld".to_string()));
                                            }
                                        } else {
                                            if ui.button(egui::RichText::new("Select BTLD")
                                                .color(egui::Color32::from_rgb(220, 220, 220)))
                                                .clicked() {
                                                message_queue.push(UIMessage::SelectFile(index, "btld".to_string()));
                                            }
                                        }
                                    } else if file.file_type == FileType::SWFL {
                                        ui.horizontal(|ui| {
                                            if is_selected_swfl1 {
                                                if ui.button(egui::RichText::new("[SELECTED] SWFL1")
                                                    .color(egui::Color32::from_rgb(120, 200, 120)))
                                                    .clicked() {
                                                    message_queue.push(UIMessage::ClearFile("swfl1".to_string()));
                                                }
                                            } else {
                                                if ui.button(egui::RichText::new("SWFL1")
                                                    .color(egui::Color32::from_rgb(220, 220, 220)))
                                                    .clicked() {
                                                    message_queue.push(UIMessage::SelectFile(index, "swfl1".to_string()));
                                                }
                                            }
                                            
                                            if is_selected_swfl2 {
                                                if ui.button(egui::RichText::new("[SELECTED] SWFL2")
                                                    .color(egui::Color32::from_rgb(120, 200, 120)))
                                                    .clicked() {
                                                    message_queue.push(UIMessage::ClearFile("swfl2".to_string()));
                                                }
                                            } else {
                                                if ui.button(egui::RichText::new("SWFL2")
                                                    .color(egui::Color32::from_rgb(220, 220, 220)))
                                                    .clicked() {
                                                    message_queue.push(UIMessage::SelectFile(index, "swfl2".to_string()));
                                                }
                                            }
                                        });
                                    }
                                });
                            });
                        });
                        
                        ui.add_space(8.0);
                    }
                });
            });
    }
}

pub fn render_selected_files(
    ui: &mut egui::Ui,
    btld_file: &Option<PathBuf>,
    swfl1_file: &Option<PathBuf>,
    swfl2_file: &Option<PathBuf>,
    message_queue: &mut Vec<UIMessage>
) {
    if btld_file.is_some() || swfl1_file.is_some() || swfl2_file.is_some() {
        ui.add_space(10.0);
        ui.group(|ui| {
            ui.heading(egui::RichText::new("Selected Files")
                .size(16.0)
                .color(egui::Color32::from_rgb(160, 200, 160)));
            
            if let Some(ref path) = btld_file {
                let file_name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("BTLD:")
                        .color(egui::Color32::from_rgb(200, 180, 120)));
                    ui.label(egui::RichText::new(&file_name)
                        .color(egui::Color32::from_rgb(160, 200, 160)));
                    if ui.button(egui::RichText::new("Clear")
                        .color(egui::Color32::from_rgb(200, 140, 140)))
                        .clicked() {
                        message_queue.push(UIMessage::ClearFile("btld".to_string()));
                    }
                });
            }
            
            if let Some(ref path) = swfl1_file {
                let file_name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("SWFL1:")
                        .color(egui::Color32::from_rgb(200, 180, 120)));
                    ui.label(egui::RichText::new(&file_name)
                        .color(egui::Color32::from_rgb(160, 200, 160)));
                    if ui.button(egui::RichText::new("Clear")
                        .color(egui::Color32::from_rgb(200, 140, 140)))
                        .clicked() {
                        message_queue.push(UIMessage::ClearFile("swfl1".to_string()));
                    }
                });
            }
            
            if let Some(ref path) = swfl2_file {
                let file_name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("SWFL2:")
                        .color(egui::Color32::from_rgb(200, 180, 120)));
                    ui.label(egui::RichText::new(&file_name)
                        .color(egui::Color32::from_rgb(160, 200, 160)));
                    if ui.button(egui::RichText::new("Clear")
                        .color(egui::Color32::from_rgb(200, 140, 140)))
                        .clicked() {
                        message_queue.push(UIMessage::ClearFile("swfl2".to_string()));
                    }
                });
            }
        });
    }
}

pub fn render_manual_file_selection(
    ui: &mut egui::Ui,
    btld_file: &Option<PathBuf>,
    swfl1_file: &Option<PathBuf>,
    swfl2_file: &Option<PathBuf>,
    message_queue: &mut Vec<UIMessage>
) {
    ui.collapsing("Manual File Selection", |ui| {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("BTLD (bootloader) File:")
                .color(egui::Color32::from_rgb(180, 180, 180)));
            if let Some(ref path) = btld_file {
                ui.label(egui::RichText::new(path.to_string_lossy())
                    .color(egui::Color32::from_rgb(140, 200, 140)));
            } else {
                ui.label(egui::RichText::new("No file selected")
                    .color(egui::Color32::from_rgb(200, 140, 140)));
            }
            if ui.button(egui::RichText::new("Browse")
                .color(egui::Color32::from_rgb(220, 220, 220)))
                .clicked() {
                message_queue.push(UIMessage::SelectBTLDFile);
            }
        });
        
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("SWFL1 (program) File:")
                .color(egui::Color32::from_rgb(180, 180, 180)));
            if let Some(ref path) = swfl1_file {
                ui.label(egui::RichText::new(path.to_string_lossy())
                    .color(egui::Color32::from_rgb(140, 200, 140)));
            } else {
                ui.label(egui::RichText::new("No file selected")
                    .color(egui::Color32::from_rgb(200, 140, 140)));
            }
            if ui.button(egui::RichText::new("Browse")
                .color(egui::Color32::from_rgb(220, 220, 220)))
                .clicked() {
                message_queue.push(UIMessage::SelectSWFL1File);
            }
        });
        
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("SWFL2 (tune) File:")
                .color(egui::Color32::from_rgb(180, 180, 180)));
            if let Some(ref path) = swfl2_file {
                ui.label(egui::RichText::new(path.to_string_lossy())
                    .color(egui::Color32::from_rgb(140, 200, 140)));
            } else {
                ui.label(egui::RichText::new("No file selected")
                    .color(egui::Color32::from_rgb(200, 140, 140)));
            }
            if ui.button(egui::RichText::new("Browse")
                .color(egui::Color32::from_rgb(220, 220, 220)))
                .clicked() {
                message_queue.push(UIMessage::SelectSWFL2File);
            }
        });
    });
}

pub fn render_output_configuration(
    ui: &mut egui::Ui,
    output_file: &Option<PathBuf>,
    message_queue: &mut Vec<UIMessage>
) {
    ui.group(|ui| {
        ui.heading(egui::RichText::new("Output Configuration")
            .size(16.0)
            .color(egui::Color32::from_rgb(120, 160, 200)));
        
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Output File:")
                .color(egui::Color32::from_rgb(180, 180, 180)));
            if let Some(ref path) = output_file {
                ui.label(egui::RichText::new(path.to_string_lossy())
                    .color(egui::Color32::from_rgb(140, 200, 140)));
            } else {
                ui.label(egui::RichText::new("No file selected")
                    .color(egui::Color32::from_rgb(200, 140, 140)));
            }
            if ui.button(egui::RichText::new("Browse")
                .color(egui::Color32::from_rgb(220, 220, 220)))
                .clicked() {
                message_queue.push(UIMessage::SelectOutputFile);
            }
        });
    });
}

pub fn render_extract_button(
    ui: &mut egui::Ui,
    is_processing: bool,
    message_queue: &mut Vec<UIMessage>
) {
    ui.horizontal(|ui| {
        if ui.button(egui::RichText::new("Extract All")
            .size(18.0)
            .color(egui::Color32::from_rgb(220, 220, 220)))
            .clicked() && !is_processing {
            message_queue.push(UIMessage::ExtractFiles);
        }
        
        if is_processing {
            ui.add(egui::widgets::Spinner::new());
        }
    });
}

pub fn render_status(ui: &mut egui::Ui, status_message: &str) {
    ui.group(|ui| {
        ui.heading(egui::RichText::new("Status")
            .size(14.0)
            .color(egui::Color32::from_rgb(180, 180, 180)));
        ui.label(egui::RichText::new(status_message)
            .color(if status_message.contains("Error") {
                egui::Color32::from_rgb(200, 140, 140)
            } else if status_message.contains("complete") {
                egui::Color32::from_rgb(140, 200, 140)
            } else {
                egui::Color32::from_rgb(180, 180, 180)
            }));
    });
}

pub fn render_settings_window(
    ctx: &egui::Context,
    show_settings: &mut bool,
    ucl_library_path: &mut String,
    message_queue: &mut Vec<UIMessage>
) {
    if *show_settings {
        egui::Window::new("Settings")
            .open(show_settings)
            .show(ctx, |ui| {
                ui.heading(egui::RichText::new("UCL Library Configuration")
                    .size(18.0)
                    .color(egui::Color32::from_rgb(120, 160, 200)));
                
                ui.label(egui::RichText::new("UCL Library Path:")
                    .color(egui::Color32::from_rgb(180, 180, 180)));
                ui.text_edit_singleline(ucl_library_path);
                
                ui.horizontal(|ui| {
                    if ui.button(egui::RichText::new("Browse")
                        .color(egui::Color32::from_rgb(220, 220, 220)))
                        .clicked() {
                        message_queue.push(UIMessage::BrowseUCLLibrary);
                    }
                    if ui.button(egui::RichText::new("Reload Library")
                        .color(egui::Color32::from_rgb(220, 220, 220)))
                        .clicked() {
                        message_queue.push(UIMessage::ReloadUCLLibrary);
                    }
                });
                
                ui.add_space(10.0);
                ui.label(egui::RichText::new("Note: Changes will be saved when you close the application.")
                    .color(egui::Color32::from_rgb(160, 160, 160))
                    .size(12.0));
            });
    }
}