$ErrorActionPreference = "Stop"

$Repo = "AAEO04/ifa-lang"
$InstallDir = "$env:LOCALAPPDATA\Programs\ifa"
$BinaryName = "ifa-windows-x86_64.exe"
$BaseUrl = "https://github.com/$Repo/releases/latest/download"
$DownloadUrl = "$BaseUrl/$BinaryName"
$ChecksumUrl = "$BaseUrl/SHA256SUMS"

Write-Host "Installing Ifa-Lang..." -ForegroundColor Cyan
Write-Host ""

# Create install directory
if (-not (Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
}

# Create temporary directory for secure download
$TempDir = Join-Path ([System.IO.Path]::GetTempPath()) ([System.Guid]::NewGuid().ToString())
New-Item -ItemType Directory -Force -Path $TempDir | Out-Null

try {
    $TempBinary = Join-Path $TempDir "ifa.exe"
    $TempChecksums = Join-Path $TempDir "SHA256SUMS"
    
    # Download binary
    Write-Host "Downloading binary from $DownloadUrl..."
    try {
        Invoke-WebRequest -Uri $DownloadUrl -OutFile $TempBinary -UseBasicParsing
    } catch {
        Write-Error "Failed to download Ifa-Lang. Please check your internet connection."
        exit 1
    }
    
    # Download checksums
    Write-Host "Downloading checksums..."
    try {
        Invoke-WebRequest -Uri $ChecksumUrl -OutFile $TempChecksums -UseBasicParsing
    } catch {
        Write-Error "Failed to download checksums. Cannot verify binary integrity."
        exit 1
    }
    
    # Verify checksum
    Write-Host "Verifying binary integrity..." -ForegroundColor Yellow
    
    $ChecksumContent = Get-Content $TempChecksums
    $ExpectedLine = $ChecksumContent | Where-Object { $_ -match $BinaryName }
    
    if (-not $ExpectedLine) {
        Write-Error "Could not find checksum for $BinaryName in SHA256SUMS"
        exit 1
    }
    
    $ExpectedHash = ($ExpectedLine -split '\s+')[0].ToLower()
    $ActualHash = (Get-FileHash -Path $TempBinary -Algorithm SHA256).Hash.ToLower()
    
    if ($ActualHash -ne $ExpectedHash) {
        Write-Host ""
        Write-Host "SECURITY ALERT: Checksum verification failed!" -ForegroundColor Red
        Write-Host "  Expected: $ExpectedHash" -ForegroundColor Red
        Write-Host "  Got:      $ActualHash" -ForegroundColor Red
        Write-Host ""
        Write-Host "  The downloaded binary may have been tampered with." -ForegroundColor Red
        Write-Host "  Installation aborted for your security." -ForegroundColor Red
        exit 1
    }
    
    Write-Host "Checksum verified successfully." -ForegroundColor Green
    
    # Move verified binary to install directory
    $OutputPath = Join-Path $InstallDir "ifa.exe"
    Move-Item -Path $TempBinary -Destination $OutputPath -Force
    
} finally {
    # Cleanup temp directory
    if (Test-Path $TempDir) {
        Remove-Item -Path $TempDir -Recurse -Force -ErrorAction SilentlyContinue
    }
}

# Add to PATH
$UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($UserPath -notlike "*$InstallDir*") {
    Write-Host "Adding to PATH..."
    [Environment]::SetEnvironmentVariable("Path", "$UserPath;$InstallDir", "User")
    $env:Path += ";$InstallDir"
    Write-Host "PATH updated. Restart your terminal to use 'ifa' command." -ForegroundColor Yellow
}

Write-Host ""
Write-Host "Ifa-Lang installed successfully!" -ForegroundColor Green
Write-Host "  Location: $OutputPath"
Write-Host "Run 'ifa --version' to get started."
