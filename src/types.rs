use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct AvailableFile {
    pub path: PathBuf,
    pub file_type: FileType,
    pub display_name: String,
    pub size: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FileType {
    BTLD,
    SWFL,
}

#[derive(Debug)]
pub enum FileAction {
    Clear(String),
    SelectBTLD(usize),
    SelectSWFL1(usize),
    SelectSWFL2(usize),
}

#[derive(Debug)]
pub struct FlashSegment {
    pub source_start_addr: u32,
    pub source_end_addr: u32,
    pub target_start_addr: u32,
    pub target_end_addr: u32,
    pub is_compressed: bool,
}

#[derive(Debug)]
pub enum UIMessage {
    SelectPSDZFolder,
    ToggleFileBrowser,
    SelectFile(usize, String), // index, file_type
    ClearFile(String),
    SelectBTLDFile,
    SelectSWFL1File,
    SelectSWFL2File,
    SelectOutputFile,
    ExtractFiles,
    ReloadUCLLibrary,
    BrowseUCLLibrary,
} 