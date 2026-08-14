# Fetches the prebuilt sherpa-onnx shared libraries into vendor/sherpa-onnx/.
# sherpa-onnx is Apache-2.0 and is not redistributed with this source tree.
#
# The version must match the `sherpa-onnx` crate in src-tauri/Cargo.toml: the crate links against
# these libraries, and a mismatched pair loads and then fails at the first call.

$ErrorActionPreference = 'Stop'

$version = '1.13.4'
$root = Split-Path -Parent $PSScriptRoot
$destination = Join-Path $root 'vendor\sherpa-onnx'
$bin = Join-Path $destination 'bin'
$library = Join-Path $bin 'sherpa-onnx-c-api.dll'

if (Test-Path $library) {
    Write-Host "sherpa-onnx already present: $library"
    exit 0
}

$name = "sherpa-onnx-v$version-win-x64-shared-MT-Release-lib.tar.bz2"
$url = "https://github.com/k2-fsa/sherpa-onnx/releases/download/v$version/$name"
$staging = Join-Path $destination 'staging'
New-Item -ItemType Directory -Force -Path $staging, $bin | Out-Null
$archive = Join-Path $destination $name

Write-Host "Downloading $name"
Invoke-WebRequest -Uri $url -OutFile $archive -UseBasicParsing
tar -xjf $archive -C $staging
Remove-Item $archive

Get-ChildItem -Path $staging -Recurse -Filter '*.dll' | Copy-Item -Destination $bin -Force
Remove-Item $staging -Recurse -Force

Write-Host "sherpa-onnx ready: $library"
