#!/usr/bin/env pwsh

$Repo = "yookibooki/webhands"
$HtmlRepo = "JohannesKaufmann/html-to-markdown"

$Target = if ($env:PROCESSOR_ARCHITECTURE -eq 'ARM64') {
    "aarch64-pc-windows-msvc"; $HtmlTarget = "Windows_arm64"
} elseif ([Environment]::Is64BitOperatingSystem) {
    "x86_64-pc-windows-msvc"; $HtmlTarget = "Windows_x86_64"
} else {
    "i686-pc-windows-msvc"; $HtmlTarget = "Windows_i386"
}

$Bin = "webhands-${Target}.exe"
$Url = "https://github.com/${Repo}/releases/latest/download/${Bin}"

$HtmlAsset = "html-to-markdown_${HtmlTarget}.zip"
$HtmlUrl = "https://github.com/${HtmlRepo}/releases/latest/download/${HtmlAsset}"

$InstallDir = Join-Path $env:LOCALAPPDATA "Microsoft" "WindowsApps"
$OutFile = Join-Path $InstallDir "webhands.exe"

New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null

Write-Host "downloading webhands..."
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
Invoke-WebRequest -Uri $Url -OutFile $OutFile -UseBasicParsing

Write-Host "installed to $OutFile"

$HtmlZip = Join-Path $env:TMP "html2markdown.zip"
$HtmlOutFile = Join-Path $InstallDir "html2markdown.exe"

Write-Host "downloading html2markdown..."
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
Invoke-WebRequest -Uri $HtmlUrl -OutFile $HtmlZip -UseBasicParsing

$HtmlTmp = Join-Path $env:TMP "html2markdown_extract"
Expand-Archive -Path $HtmlZip -DestinationPath $HtmlTmp -Force
Move-Item -Path (Join-Path $HtmlTmp "html2markdown.exe") -Destination $HtmlOutFile -Force
Remove-Item -Path $HtmlTmp -Recurse -Force
Remove-Item -Path $HtmlZip -Force
Write-Host "installed to $HtmlOutFile"
