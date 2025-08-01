# Simple build script for SWFL Extractor

Write-Host "Building BMW Virtual Reader portable package..."

# Build if not skipping
if ($args -notcontains "-SkipBuild") {
    Write-Host "Building release version..."
    cargo build --release
}

# Check files exist
$exe = "target\release\bmw_virtual_reader.exe"
$dll = "lib\libucl-1.dll"

if (-not (Test-Path $exe)) {
    Write-Host "ERROR: Executable not found" -ForegroundColor Red
    exit 1
}

if (-not (Test-Path $dll)) {
    Write-Host "ERROR: DLL not found" -ForegroundColor Red
    exit 1
}

# Create portable directory
$portable = "BMW_Virtual_Reader_Portable"
if (Test-Path $portable) {
    Remove-Item $portable -Recurse -Force
}
New-Item -ItemType Directory -Path $portable | Out-Null

# Copy files
Copy-Item $exe "$portable\"
Copy-Item $dll "$portable\"

# Create config for portable version
$config = @"
{
  "last_input_dir": null,
  "last_output_dir": null,
  "window_width": 600.0,
  "window_height": 400.0,
  "ucl_library_path": "libucl-1.dll"
}
"@
$config | Out-File "$portable\config.json" -Encoding UTF8

# Create README
$readme = @"
BMW Virtual Reader - Portable Version

To use:
1. Double-click bmw_virtual_reader.exe
2. Select your BTLD, SWFL1, and SWFL2 files
3. Choose output location and click "Extract All"

Files included:
- bmw_virtual_reader.exe (main application)
- libucl-1.dll (required decompression library)
- config.json (default configuration)

System Requirements: Windows 7 or later
"@
$readme | Out-File "$portable\README.txt" -Encoding UTF8

# Create zip if not skipping
if ($args -notcontains "-SkipZip") {
    $zip = "BMW_Virtual_Reader_Portable_v0.3.0.zip"
    if (Test-Path $zip) {
        Remove-Item $zip -Force
    }
    
    Add-Type -AssemblyName System.IO.Compression.FileSystem
    [System.IO.Compression.ZipFile]::CreateFromDirectory($portable, $zip)
    
    $size = (Get-Item $zip).Length
    $sizeMB = [math]::Round($size / 1MB, 2)
    Write-Host "Created $zip ($sizeMB MB)" -ForegroundColor Green
}

Write-Host "Portable package created in: $portable" -ForegroundColor Green 