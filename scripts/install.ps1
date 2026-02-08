<# 
.SYNOPSIS
Installer for dugout — a local secrets manager for development teams.

.DESCRIPTION
Downloads the latest dugout release for Windows and installs it to ~/.dugout/bin.
Adds the install directory to your PATH if not already present.

.PARAMETER Version
Specific version to install (e.g. "0.1.2"). Defaults to latest.

.PARAMETER InstallDir
Custom install directory. Defaults to ~/.dugout/bin.

.PARAMETER NoModifyPath
Don't add the install directory to PATH.

.PARAMETER Help
Print help.
#>

param (
    [string]$Version,
    [string]$InstallDir,
    [switch]$NoModifyPath,
    [switch]$Help
)

$ErrorActionPreference = "Stop"

$repo = "usealtoal/dugout"

if ($Help) {
    Get-Help $PSCommandPath -Detailed
    exit 0
}

# Determine install directory
if (-not $InstallDir) {
    $InstallDir = Join-Path $env:USERPROFILE ".dugout\bin"
}

# Detect architecture
function Get-Arch {
    try {
        $a = [System.Reflection.Assembly]::LoadWithPartialName("System.Runtime.InteropServices.RuntimeInformation")
        $t = $a.GetType("System.Runtime.InteropServices.RuntimeInformation")
        $p = $t.GetProperty("OSArchitecture")
        switch ($p.GetValue($null).ToString()) {
            "X64" { return "x86_64" }
            "Arm64" { return "aarch64" }
            default { return $null }
        }
    } catch {
        if ([System.Environment]::Is64BitOperatingSystem) {
            return "x86_64"
        }
        return $null
    }
}

$arch = Get-Arch
if (-not $arch) {
    Write-Error "unsupported architecture"
    exit 1
}

$target = "$arch-pc-windows-msvc"

# Get version
if (-not $Version) {
    $release = Invoke-RestMethod -Uri "https://api.github.com/repos/$repo/releases/latest"
    $Version = $release.tag_name -replace '^v', ''
}

$url = "https://github.com/$repo/releases/download/v$Version/dugout-$target.zip"

Write-Host "downloading dugout v$Version for $target..."

# Download and extract
$tmp = New-TemporaryFile | Rename-Item -NewName { $_.Name + ".zip" } -PassThru
try {
    Invoke-WebRequest -Uri $url -OutFile $tmp -UseBasicParsing

    $extractDir = Join-Path ([System.IO.Path]::GetTempPath()) "dugout-install"
    if (Test-Path $extractDir) { Remove-Item -Recurse -Force $extractDir }
    Expand-Archive -Path $tmp -DestinationPath $extractDir

    # Create install directory
    if (-not (Test-Path $InstallDir)) {
        New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
    }

    # Find and move the binary
    $exe = Get-ChildItem -Path $extractDir -Filter "dugout.exe" -Recurse | Select-Object -First 1
    if (-not $exe) {
        Write-Error "dugout.exe not found in archive"
        exit 1
    }

    Copy-Item -Path $exe.FullName -Destination (Join-Path $InstallDir "dugout.exe") -Force

    Write-Host "installed dugout to $InstallDir\dugout.exe"
} finally {
    Remove-Item -Force $tmp -ErrorAction SilentlyContinue
    Remove-Item -Recurse -Force $extractDir -ErrorAction SilentlyContinue
}

# Add to PATH
if (-not $NoModifyPath) {
    $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if ($userPath -notlike "*$InstallDir*") {
        [Environment]::SetEnvironmentVariable("Path", "$userPath;$InstallDir", "User")
        $env:Path = "$env:Path;$InstallDir"
        Write-Host "added $InstallDir to your PATH"
    }
}

Write-Host "run 'dugout setup' to get started"
