use eframe::egui;
use crate::app::BMWVirtualReaderApp;
use crate::ui::*;
use crate::types::UIMessage;

mod config;
mod ucl_bindings;
mod types;
mod xml_parser;
mod file_ops;
mod ui;
mod app;

impl eframe::App for BMWVirtualReaderApp {
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        if let Err(e) = self.config.save() {
            eprintln!("Failed to save config: {}", e);
        }
    }
    
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Header
            render_header(ui, &mut self.ui_state.show_settings);
            
            ui.add_space(5.0);
            ui.separator();
            ui.add_space(15.0);
            
            // PSDZ Section
            render_psdz_section(
                ui,
                &self.psdz_folder,
                &mut self.ui_state.message_queue
            );
            
            ui.add_space(10.0);
            
            // File Browser
            render_file_browser(
                ctx,
                &mut self.ui_state.show_file_browser,
                &self.available_files,
                &mut self.ui_state.file_search_filter,
                &self.ui_state.selected_btld_index,
                &self.ui_state.selected_swfl1_index,
                &self.ui_state.selected_swfl2_index,
                &mut self.ui_state.message_queue
            );
            
            // Selected Files
            render_selected_files(
                ui,
                &self.btld_file,
                &self.swfl1_file,
                &self.swfl2_file,
                &mut self.ui_state.message_queue
            );
            
            ui.add_space(10.0);
            
            // Manual File Selection
            render_manual_file_selection(
                ui,
                &self.btld_file,
                &self.swfl1_file,
                &self.swfl2_file,
                &mut self.ui_state.message_queue
            );
            
            ui.add_space(10.0);
            
            // Output Configuration
            render_output_configuration(
                ui,
                &self.output_file,
                &mut self.ui_state.desired_size_mb,
                &mut self.ui_state.use_desired_size,
                &mut self.ui_state.message_queue
            );
            
            ui.add_space(20.0);
            
            // Extract Button
            render_extract_button(
                ui,
                self.is_processing,
                &mut self.ui_state.message_queue
            );
            
            ui.add_space(10.0);
            
            // Status
            render_status(ui, &self.status_message);
            
            // Settings Window
            render_settings_window(
                ctx,
                &mut self.ui_state.show_settings,
                &mut self.config.ucl_library_path,
                &mut self.ui_state.message_queue
            );
        });
        
        // Handle UI messages after rendering
        self.handle_ui_messages();
    }
}

impl BMWVirtualReaderApp {
    fn handle_ui_messages(&mut self) {
        let messages: Vec<UIMessage> = self.ui_state.message_queue.drain(..).collect();
        
        for message in messages {
            match message {
                UIMessage::SelectPSDZFolder => {
                    self.select_psdz_folder();
                }
                UIMessage::ToggleFileBrowser => {
                    self.ui_state.show_file_browser = !self.ui_state.show_file_browser;
                }
                UIMessage::SelectFile(index, file_type) => {
                    self.select_file_by_index(index, &file_type);
                }
                UIMessage::ClearFile(file_type) => {
                    self.clear_file_selection(&file_type);
                }
                UIMessage::SelectBTLDFile => {
                    self.select_btld_file();
                }
                UIMessage::SelectSWFL1File => {
                    self.select_swfl1_file();
                }
                UIMessage::SelectSWFL2File => {
                    self.select_swfl2_file();
                }
                UIMessage::SelectOutputFile => {
                    self.select_output_file();
                }
                UIMessage::ExtractFiles => {
                    if let Err(e) = self.process_files() {
                        self.status_message = format!("Error: {}", e);
                    }
                }
                UIMessage::ReloadUCLLibrary => {
                    self.reload_ucl_library();
                }
                UIMessage::BrowseUCLLibrary => {
                    if let Some(new_path) = rfd::FileDialog::new()
                        .add_filter("DLL files", &["dll"])
                        .add_filter("All files", &["*"])
                        .pick_file() 
                    {
                        self.config.ucl_library_path = new_path.to_string_lossy().to_string();
                        self.reload_ucl_library();
                    }
                }
                UIMessage::SetDesiredSizeMB(size) => {
                    self.ui_state.desired_size_mb = size;
                }
                UIMessage::ToggleUseDesiredSize => {
                    self.ui_state.use_desired_size = !self.ui_state.use_desired_size;
                }
            }
        }
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        default_theme: eframe::Theme::Dark,
        ..Default::default()
    };
    
    eframe::run_native(
        "BMW Virtual Reader",
        options,
        Box::new(|cc| {
            let app = BMWVirtualReaderApp::new(cc);
            // Set dark theme colors
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Box::new(app)
        }),
    )
} 
