# Fetches the prebuilt Pdfium shared library into vendor/pdfium/.
# Pdfium is BSD-3 and is not redistributed with this source tree.

$ErrorActionPreference = 'Stop'

$root = Split-Path -Parent $PSScriptRoot
$destination = Join-Path $root 'vendor\pdfium'
$library = Join-Path $destination 'bin\pdfium.dll'

if (Test-Path $library) {
    Write-Host "pdfium already present: $library"
    exit 0
}

$release = Invoke-RestMethod -Uri 'https://api.github.com/repos/bblanchon/pdfium-binaries/releases/latest' `
    -Headers @{ 'User-Agent' = 'OpenExamTrainer' } -UseBasicParsing

$asset = $release.assets | Where-Object { $_.name -eq 'pdfium-win-x64.tgz' } | Select-Object -First 1
if (-not $asset) {
    throw 'pdfium-win-x64.tgz is missing from the latest release'
}

New-Item -ItemType Directory -Force -Path $destination | Out-Null
$archive = Join-Path $destination $asset.name

Write-Host "Downloading $($asset.name) ($([math]::Round($asset.size / 1MB, 1)) MB) from $($release.tag_name)"
Invoke-WebRequest -Uri $asset.browser_download_url -OutFile $archive -UseBasicParsing
tar -xzf $archive -C $destination
Remove-Item $archive

Write-Host "pdfium ready: $library"
