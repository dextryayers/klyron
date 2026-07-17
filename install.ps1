#!/usr/bin/env pwsh
# Install Klyron on Windows (PowerShell 5.1+ / pwsh)
param(
  [string]$Version = "latest",
  [string]$InstallDir = "$env:LOCALAPPDATA\klyron"
)

$Repo = "dextryayers/klyron"
$BinName = "klyron"

# ── Platform detection ──────────────────────────────────────────

$Arch = switch ($env:PROCESSOR_ARCHITECTURE) {
  "AMD64"  { "x86_64"; break }
  "ARM64"  { "aarch64"; break }
  default  { Write-Error "Unsupported architecture: $env:PROCESSOR_ARCHITECTURE"; exit 1 }
}

# ── Download & install ──────────────────────────────────────────

$ArchiveExt = ".zip"
if ($Version -eq "latest") {
  $Url = "https://github.com/$Repo/releases/latest/download/${BinName}-windows-${Arch}${ArchiveExt}"
} else {
  $Url = "https://github.com/$Repo/releases/download/${Version}/${BinName}-windows-${Arch}${ArchiveExt}"
}

$TmpDir = "$env:TEMP\klyron-install"
$ZipPath = "$TmpDir\klyron.zip"

Write-Host "Downloading $Url ..."
New-Item -ItemType Directory -Force -Path $TmpDir | Out-Null
Invoke-WebRequest -Uri $Url -OutFile $ZipPath

Write-Host "Extracting ..."
Expand-Archive -Path $ZipPath -DestinationPath $TmpDir -Force

# Ensure install dir exists
New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null

# Move binary
$SrcExe = "$TmpDir\${BinName}.exe"
$DstExe = "$InstallDir\${BinName}.exe"
Move-Item -Force -Path $SrcExe -Destination $DstExe

# Clean up
Remove-Item -Recurse -Force $TmpDir -ErrorAction SilentlyContinue

Write-Host "Installed $BinName to $DstExe"

# Add to PATH if not already
$UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($UserPath -notlike "*$InstallDir*") {
  [Environment]::SetEnvironmentVariable("Path", "$UserPath;$InstallDir", "User")
  Write-Host "Added $InstallDir to user PATH. Restart your terminal or run:"
  Write-Host "  `$env:Path += ';$InstallDir'"
}

Write-Host ""
Write-Host "Klyron installed successfully!"
Write-Host "Run 'klyron --version' to verify."
