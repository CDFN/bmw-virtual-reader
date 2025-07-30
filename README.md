# BMW Virtual Reader

A Rust-based GUI application for extracting SWFL/BTLD files using XML configuration and UCL decompression.

## Prerequisites

- Rust toolchain (install from https://rustup.rs/)
- Windows (for the provided UCL DLL)

## Building

1. Install Rust if you haven't already:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. Build the application:
   ```bash
   cargo build --release
   ```

3. The executable will be created at `target/release/bmw_virtual_reader.exe`

4. Create portable package:
   ```powershell
   powershell -ExecutionPolicy Bypass -File build.ps1
   ```

## Usage

### PSDZ Data Directory Selection (Recommended)

1. Run the application:
   ```bash
   cargo run --release
   ```

2. Click "Browse" next to "PSDZ Data Folder" to select your psdzdata directory
3. Click "File Browser" to open the file selection window
4. In the file browser:
   - Use the search filter to find specific files (case-insensitive, handles `-` and `_` interchangeably)
   - Select BTLD files by clicking "Select BTLD"
   - Select SWFL files by clicking "SWFL1" and/or "SWFL2"
5. Choose your output file location
6. (Optional) Check "Use Desired Size" and set the desired output file size in MB - if the combined files are smaller than this size, zero data will be appended to reach the target size. Required for some ECUs (e.g. EDC17C50 needs 4MB).
7. Click "Create binary" to process the selected files

## File Structure Support

The application automatically scans and supports the following PSDZ directory structure:
```
psdzdata/
├── swe/
│   ├── btld/          # Bootloader files (*.bin)
│   └── swfl/          # Software files (*.bin)
```