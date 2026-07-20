#!/usr/bin/env pwsh

$Version = if ($env:VERSION) { $env:VERSION } else { "v0.1.0" }
$Repo = "yookibooki/web_search"

$Arch = if ([Environment]::Is64BitOperatingSystem) { "x86_64" } else { "i686" }
$Target = "x86_64-pc-windows-msvc"
$Bin = "web_search-${Target}.exe"
$Url = "https://github.com/${Repo}/releases/download/${Version}/${Bin}"

$InstallDir = Join-Path $HOME ".local" "bin"
$OutFile = Join-Path $InstallDir "web_search.exe"

New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null

Write-Host "downloading $Bin $Version..."
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
Invoke-WebRequest -Uri $Url -OutFile $OutFile -UseBasicParsing

Write-Host "installed to $OutFile"

$PathValue = [Environment]::GetEnvironmentVariable("PATH", "User")
if ($PathValue -notlike "*$InstallDir*") {
    Write-Host "note: add $InstallDir to PATH"
}
