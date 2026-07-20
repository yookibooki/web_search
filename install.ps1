#!/usr/bin/env pwsh

$Version = if ($env:VERSION) { $env:VERSION } else { "v0.1.0" }
$HtmlVersion = if ($env:HTML2MARKDOWN_VERSION) { $env:HTML2MARKDOWN_VERSION } else { "v2.5.2" }
$HtmlVersionNoV = $HtmlVersion.TrimStart('v')
$Repo = "yookibooki/web_search"
$HtmlRepo = "JohannesKaufmann/html-to-markdown"

$Target = if ($env:PROCESSOR_ARCHITECTURE -eq 'ARM64') {
    "aarch64-pc-windows-msvc"; $HtmlTarget = "Windows_arm64"
} elseif ([Environment]::Is64BitOperatingSystem) {
    "x86_64-pc-windows-msvc"; $HtmlTarget = "Windows_x86_64"
} else {
    "i686-pc-windows-msvc"; $HtmlTarget = "Windows_i386"
}
$Bin = "web-${Target}.exe"
$Url = "https://github.com/${Repo}/releases/download/${Version}/${Bin}"

$InstallDir = Join-Path $env:LOCALAPPDATA "Microsoft" "WindowsApps"
$OutFile = Join-Path $InstallDir "web.exe"

New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null

Write-Host "downloading $Bin $Version..."
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
Invoke-WebRequest -Uri $Url -OutFile $OutFile -UseBasicParsing

Write-Host "installed to $OutFile"

$HtmlAsset = "html-to-markdown_${HtmlVersionNoV}_${HtmlTarget}.zip"
$HtmlUrl = "https://github.com/${HtmlRepo}/releases/download/${HtmlVersion}/${HtmlAsset}"
$HtmlZip = Join-Path $env:TMP "html2markdown.zip"
$HtmlOutFile = Join-Path $InstallDir "html2markdown.exe"

Write-Host "downloading html2markdown ${HtmlVersion}..."
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
Invoke-WebRequest -Uri $HtmlUrl -OutFile $HtmlZip -UseBasicParsing

$HtmlTmp = Join-Path $env:TMP "html2markdown_extract"
Expand-Archive -Path $HtmlZip -DestinationPath $HtmlTmp -Force
Move-Item -Path (Join-Path $HtmlTmp "html2markdown.exe") -Destination $HtmlOutFile -Force
Remove-Item -Path $HtmlTmp -Recurse -Force
Remove-Item -Path $HtmlZip -Force
Write-Host "installed to $HtmlOutFile"
