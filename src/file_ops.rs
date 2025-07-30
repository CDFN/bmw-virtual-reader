use std::fs;
use std::io::{Read, Seek};
use std::path::PathBuf;
use anyhow::{Result, Context};
use crate::types::{AvailableFile, FileType};
use crate::xml_parser::parse_xml;
use crate::ucl_bindings::UclLibrary;

pub fn scan_psdz_files(psdz_path: &PathBuf) -> Vec<AvailableFile> {
    let mut available_files = Vec::new();
    
    // Scan BTLD files
    let btld_path = psdz_path.join("swe").join("btld");
    if btld_path.exists() {
        if let Ok(entries) = fs::read_dir(btld_path) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if let Some(file_name) = path.file_name() {
                    let file_name_str = file_name.to_string_lossy();
                    // Check if filename contains ".bin" (for files like .bin.001_015_000)
                    if file_name_str.contains(".bin") {
                        if let Ok(metadata) = fs::metadata(&path) {
                            // Convert display name: replace .bin. with _ for better readability
                            let display_name = file_name_str.replace(".bin.", "_");
                            
                            available_files.push(AvailableFile {
                                path,
                                file_type: FileType::BTLD,
                                display_name,
                                size: metadata.len(),
                            });
                        }
                    }
                }
            }
        }
    }
    
    // Scan SWFL files
    let swfl_path = psdz_path.join("swe").join("swfl");
    if swfl_path.exists() {
        if let Ok(entries) = fs::read_dir(swfl_path) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if let Some(file_name) = path.file_name() {
                    let file_name_str = file_name.to_string_lossy();
                    // Check if filename contains ".bin" (for files like .bin.159_010_001)
                    if file_name_str.contains(".bin") {
                        if let Ok(metadata) = fs::metadata(&path) {
                            // Convert display name: replace .bin. with _ for better readability
                            let display_name = file_name_str.replace(".bin.", "_");
                            
                            available_files.push(AvailableFile {
                                path,
                                file_type: FileType::SWFL,
                                display_name,
                                size: metadata.len(),
                            });
                        }
                    }
                }
            }
        }
    }
    
    // Sort files by type and name
    available_files.sort_by(|a, b| {
        match (&a.file_type, &b.file_type) {
            (FileType::BTLD, FileType::SWFL) => std::cmp::Ordering::Less,
            (FileType::SWFL, FileType::BTLD) => std::cmp::Ordering::Greater,
            _ => a.display_name.cmp(&b.display_name),
        }
    });
    
    available_files
}

pub fn get_xml_path(bin_path: &PathBuf) -> PathBuf {
    let mut xml_path = bin_path.clone();
    if let Some(file_name) = xml_path.file_name() {
        let file_name_str = file_name.to_string_lossy();
        // Replace .bin with .xml in the filename (handles extended names like .bin.001_015_000)
        let xml_file_name = file_name_str.replace(".bin", ".xml");
        xml_path.set_file_name(xml_file_name);
    }
    xml_path
}

pub fn generate_output_filename(swfl1_path: &PathBuf) -> Option<String> {
    if let Some(file_name) = swfl1_path.file_name() {
        let file_name_str = file_name.to_string_lossy();
        
        // Extract the base name (remove .bin and any extensions)
        let base_name = if file_name_str.ends_with(".bin") {
            &file_name_str[..file_name_str.len() - 4]
        } else {
            &file_name_str
        };
        
        // Find the last underscore to get the version part
        if let Some(last_underscore_pos) = base_name.rfind('_') {
            let version_part = &base_name[last_underscore_pos + 1..];
            // Create the new filename: version_part.vr.bin
            return Some(format!("{}.vr.bin", version_part));
        }
    }
    None
}

pub fn get_program_directory() -> PathBuf {
    // Try to get the executable directory
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            return exe_dir.to_path_buf();
        }
    }
    
    // Fallback to current directory
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

pub fn decompress_ucl(ucl_library: &UclLibrary, data: &[u8]) -> Result<Vec<u8>> {
    ucl_library.decompress(data).map_err(|e| anyhow::anyhow!("UCL decompression failed: {}", e))
}

pub fn process_single_file(
    bin_path: &PathBuf, 
    xml_path: &PathBuf, 
    ucl_library: &UclLibrary
) -> Result<Vec<(u32, Vec<u8>)>> {
    // Parse XML
    let segments = parse_xml(xml_path)?;
    
    // Read and process binary file
    let mut input_file = fs::File::open(bin_path)
        .context("Failed to open input file")?;
    
    let mut buff_list = Vec::new();
    
    for segment in segments {
        let source_size = segment.source_end_addr - segment.source_start_addr + 1;
        let target_size = segment.target_end_addr - segment.target_start_addr + 1;
        
        let mut buffer = vec![0u8; source_size as usize];
        input_file.seek(std::io::SeekFrom::Start(segment.source_start_addr as u64))?;
        input_file.read_exact(&mut buffer)?;
        
        let output_buffer = if segment.is_compressed {
            decompress_ucl(ucl_library, &buffer)?
        } else {
            buffer
        };
        
        if output_buffer.len() != target_size as usize {
            eprintln!("Warning: Size mismatch for segment - expected {} bytes, got {}", 
                target_size, output_buffer.len());
        }
        
        buff_list.push((segment.target_start_addr, output_buffer));
    }
    
    Ok(buff_list)
}

pub fn process_files(
    btld_file: Option<&PathBuf>,
    swfl1_file: Option<&PathBuf>,
    swfl2_file: Option<&PathBuf>,
    output_file: &PathBuf,
    ucl_library: &UclLibrary,
    status_callback: &mut dyn FnMut(&str)
) -> Result<()> {
    let mut all_segments = Vec::new();
    
    // Process BTLD file
    if let Some(btld_path) = btld_file {
        let xml_path = get_xml_path(btld_path);
        status_callback(&format!("Processing BTLD file: {}", btld_path.file_name().unwrap_or_default().to_string_lossy()));
        
        match process_single_file(btld_path, &xml_path, ucl_library) {
            Ok(segments) => {
                let segment_count = segments.len();
                all_segments.extend(segments);
                status_callback(&format!("BTLD: Found {} segments", segment_count));
            }
            Err(e) => {
                status_callback(&format!("Warning: Failed to process BTLD file: {}", e));
            }
        }
    }
    
    // Process SWFL1 file
    if let Some(swfl1_path) = swfl1_file {
        let xml_path = get_xml_path(swfl1_path);
        status_callback(&format!("Processing SWFL1 file: {}", swfl1_path.file_name().unwrap_or_default().to_string_lossy()));
        
        match process_single_file(swfl1_path, &xml_path, ucl_library) {
            Ok(segments) => {
                let segment_count = segments.len();
                all_segments.extend(segments);
                status_callback(&format!("SWFL1: Found {} segments", segment_count));
            }
            Err(e) => {
                status_callback(&format!("Warning: Failed to process SWFL1 file: {}", e));
            }
        }
    }
    
    // Process SWFL2 file
    if let Some(swfl2_path) = swfl2_file {
        let xml_path = get_xml_path(swfl2_path);
        status_callback(&format!("Processing SWFL2 file: {}", swfl2_path.file_name().unwrap_or_default().to_string_lossy()));
        
        match process_single_file(swfl2_path, &xml_path, ucl_library) {
            Ok(segments) => {
                let segment_count = segments.len();
                all_segments.extend(segments);
                status_callback(&format!("SWFL2: Found {} segments", segment_count));
            }
            Err(e) => {
                status_callback(&format!("Warning: Failed to process SWFL2 file: {}", e));
            }
        }
    }
    
    if all_segments.is_empty() {
        return Err(anyhow::anyhow!("No valid files to process"));
    }
    
    // Write combined aligned output
    if let Some((base_addr, _)) = all_segments.first() {
        let base_addr = *base_addr;
        let end_addr = all_segments.iter()
            .map(|(addr, data)| addr + data.len() as u32 - 1)
            .max()
            .unwrap_or(base_addr);
        let total_size = end_addr - base_addr + 1;
        
        let mut full_buffer = vec![0xFFu8; total_size as usize];
        
        for (target_addr, data) in all_segments {
            let offset = (target_addr - base_addr) as usize;
            if offset + data.len() <= full_buffer.len() {
                full_buffer[offset..offset + data.len()].copy_from_slice(&data);
            }
        }
        
        fs::write(output_file, &full_buffer)
            .context("Failed to write output file")?;
        
        status_callback(&format!("Combined extraction complete: {} bytes, range: 0x{:08X} to 0x{:08X}", 
            full_buffer.len(), base_addr, end_addr));
    }
    
    Ok(())
} 