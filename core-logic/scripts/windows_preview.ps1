# urGlance Windows Explorer Integration
# Add to Windows Registry to trigger on file hover
#
# Usage:
#   powershell -ExecutionPolicy Bypass -File windows_preview.ps1 -FilePath "C:\path\to\file"
#
# To register as a shell extension, run:
#   reg add "HKCU\Software\Classes\*\shell\urGlancePreview" /ve /t REG_SZ /d "Preview with urGlance" /f
#   reg add "HKCU\Software\Classes\*\shell\urGlancePreview\command" /ve /t REG_SZ /d "powershell -ExecutionPolicy Bypass -File \"%~dp0windows_preview.ps1\" -FilePath \"%1\"" /f

param(
    [string]$FilePath = ""
)

if ([string]::IsNullOrEmpty($FilePath)) {
    Write-Host "Usage: .\windows_preview.ps1 -FilePath <file-path>"
    exit 1
}

if (-not (Test-Path $FilePath)) {
    Write-Host "Error: File not found: $FilePath"
    exit 1
}

$uri = [System.Uri]::new("http://127.0.0.1:8080/api/preview?path=$([System.Uri]::EscapeDataString($FilePath))")

try {
    $response = Invoke-WebRequest -Uri $uri -UseBasicParsing -TimeoutSec 2
    Write-Host "Preview sent to urGlance daemon"
} catch {
    Write-Host "Error: urGlance daemon not running at http://127.0.0.1:8080"
    Write-Host "Start the daemon first: urglance.exe"
    exit 1
}
