$ErrorActionPreference = "Stop"

$Repo = "AAEO04/ifa-lang"
$InstallDir = "$env:LOCALAPPDATA\Programs\ifa"
$BinaryName = "ifa-windows-x86_64.exe"
$DownloadUrl = "https://github.com/$Repo/releases/latest/download/$BinaryName"

Write-Host "Installing Ifa-Lang..." -ForegroundColor Cyan

# Create install directory
if (-not (Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
}

# Download binary
$OutputPath = "$InstallDir\ifa.exe"
Write-Host "Downloading from $DownloadUrl..."
try {
    Invoke-WebRequest -Uri $DownloadUrl -OutFile $OutputPath
} catch {
    Write-Error "Failed to download Ifa-Lang. Please check your internet connection."
    exit 1
}

# Add to PATH
$UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($UserPath -notlike "*$InstallDir*") {
    Write-Host "Adding to PATH..."
    [Environment]::SetEnvironmentVariable("Path", "$UserPath;$InstallDir", "User")
    $env:Path += ";$InstallDir"
    Write-Host "PATH updated. restart your terminal." -ForegroundColor Yellow
}

Write-Host ""
Write-Host "✅ Ifa-Lang installed successfully!" -ForegroundColor Green
Write-Host "Location: $OutputPath"
Write-Host "Run 'ifa --version' to get started."
