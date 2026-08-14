# Applies one of the candidate icons in src-tauri/icon-variants/ and regenerates every size.
#
# Usage:  scripts/set-icon.ps1 check
#         scripts/set-icon.ps1            # lists what there is
#
# `tauri icon` also writes Android and iOS assets. This app is Windows only, so they are removed
# again rather than left to be committed.

param([string]$Name)

$ErrorActionPreference = 'Stop'

$root = Split-Path -Parent $PSScriptRoot
$variants = Join-Path $root 'src-tauri\icon-variants'
$target = Join-Path $root 'src-tauri\app-icon.svg'

$available = Get-ChildItem $variants -Filter *.svg | ForEach-Object { $_.BaseName }

if (-not $Name) {
    Write-Host "Candidates in src-tauri/icon-variants:"
    $available | ForEach-Object { Write-Host "  $_" }
    exit 0
}

if ($available -notcontains $Name) {
    throw "no candidate named '$Name'. Available: $($available -join ', ')"
}

Copy-Item (Join-Path $variants "$Name.svg") $target -Force
Write-Host "app-icon.svg <- $Name.svg"

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
