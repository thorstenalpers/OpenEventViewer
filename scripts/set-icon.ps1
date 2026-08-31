# Regenerates every icon size from src-tauri/app-icon.svg.
#
# Usage:  scripts/set-icon.ps1
#
# `tauri icon` also writes Android and iOS assets. This app is Windows only, so they are removed
# again rather than left to be committed.

$ErrorActionPreference = 'Stop'

$root = Split-Path -Parent $PSScriptRoot
$source = Join-Path $root 'src-tauri\app-icon.svg'

if (-not (Test-Path $source)) { throw "no source icon at $source" }

Push-Location $root
try {
    # 'Continue' around the native call on purpose: npx writes progress to stderr, and under
    # 'Stop' Windows PowerShell turns that into a terminating error and skips everything below.
    $previous = $ErrorActionPreference
    $ErrorActionPreference = 'Continue'
    npx tauri icon src-tauri/app-icon.svg
    $code = $LASTEXITCODE
    $ErrorActionPreference = $previous
    if ($code -ne 0) { throw "tauri icon failed with exit code $code" }
} finally {
    Pop-Location
}

foreach ($mobile in 'android', 'ios') {
    $path = Join-Path $root "src-tauri\icons\$mobile"
    if (Test-Path $path) { Remove-Item $path -Recurse -Force }
}

Write-Host "done - rebuild to see it in the taskbar"
