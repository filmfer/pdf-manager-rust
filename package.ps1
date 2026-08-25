param(
    [switch]$SkipBuild,
    [string]$OutputDir = "D:\scripts\pdf-manager-rust\dist"
)

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSCommandPath
Set-Location $root

# 1. Build release if needed
if (-not $SkipBuild) {
    Write-Host "=== Building release ===" -ForegroundColor Cyan
    $env:Path += ";$env:USERPROFILE\.cargo\bin"
    cargo build --release
    if ($LASTEXITCODE -ne 0) { throw "Build failed." }
}

# 2. Prepare output directory
if (Test-Path $OutputDir) { Remove-Item $OutputDir -Recurse -Force }
New-Item -ItemType Directory -Path $OutputDir -Force | Out-Null
New-Item -ItemType Directory -Path "$OutputDir\poppler" -Force | Out-Null

# 3. Copy the executable
Write-Host "=== Copying executable ===" -ForegroundColor Cyan
Copy-Item "target\release\pdf-manager-rust.exe" "$OutputDir\"

# 4. Copy bundled Poppler binaries
Write-Host "=== Copying bundled Poppler ===" -ForegroundColor Cyan
Get-ChildItem "assets\poppler" | Copy-Item -Destination "$OutputDir\poppler\"

# 5. Copy the icon
if (Test-Path "assets\simple_pdf_manager.ico") {
    Copy-Item "assets\simple_pdf_manager.ico" "$OutputDir\"
}

# 6. Create README
$readme = @"
# Simple PDF Manager (Rust)

A fast, lightweight PDF manager built in Rust.

## Installation
Extract this ZIP to any folder and run `pdf-manager-rust.exe`. No installation needed.

## Features
- Merge multiple PDFs into one
- Split a PDF into individual pages
- Extract a range of pages
- Remove specific pages
- Convert images to PDF
- Convert PDF to images (PNG, JPG, BMP, TIFF)

## Standalone
All dependencies are bundled. The `poppler/` folder contains the binaries
required for the "PDF to Images" feature.

## Author
Filipe Fernandes <filmfer@gmail.com>
"@
Set-Content -Path "$OutputDir\README.txt" -Value $readme -Encoding UTF8

# 7. Create the distributable ZIP
$zipPath = Join-Path $root "pdf-manager-rust-windows.zip"
if (Test-Path $zipPath) { Remove-Item $zipPath -Force }
Write-Host "=== Creating ZIP ===" -ForegroundColor Cyan
Compress-Archive -Path "$OutputDir\*" -DestinationPath $zipPath -CompressionLevel Optimal

Write-Host "=== Done ===" -ForegroundColor Green
Write-Host "Distribution folder: $OutputDir"
Write-Host "ZIP file:           $zipPath"
$size = (Get-Item $zipPath).Length / 1MB
Write-Host ("ZIP size:           {0:N2} MB" -f $size)
