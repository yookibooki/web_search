#!/usr/bin/env pwsh

$Version = if ($env:VERSION) { $env:VERSION } else { "v0.1.0" }
$Repo = "yookibooki/web_search"

$Target = if ($env:PROCESSOR_ARCHITECTURE -eq 'ARM64') {
    "aarch64-pc-windows-msvc"
} elseif ([Environment]::Is64BitOperatingSystem) {
    "x86_64-pc-windows-msvc"
} else {
    Write-Host "unsupported architecture (32-bit)" >&2
    exit 1
}
$Bin = "web_search-${Target}.exe"
$Url = "https://github.com/${Repo}/releases/download/${Version}/${Bin}"

$InstallDir = Join-Path $env:LOCALAPPDATA "Microsoft" "WindowsApps"
$OutFile = Join-Path $InstallDir "web_search.exe"

New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null

Write-Host "downloading $Bin $Version..."
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
Invoke-WebRequest -Uri $Url -OutFile $OutFile -UseBasicParsing

Write-Host "installed to $OutFile"
