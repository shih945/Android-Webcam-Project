#requires -Version 5.1
<#
.SYNOPSIS
    Build AWC in release mode and package a client folder + zip.

.DESCRIPTION
    The project itself is resolved relative to this script, so moving the
    whole repository does not break packaging. Only external toolchains need
    to be pointed at their new locations in the config block below.

.EXAMPLE
    .\package-awc.cmd

.EXAMPLE
    .\package-awc.cmd -SkipBuild
#>
param(
    [switch]$SkipBuild,
    [switch]$NoZip
)

$ErrorActionPreference = 'Stop'

# ============================================================================
# 配置目录（迁移后只改这一段）
# ============================================================================
$ProjectDir     = $PSScriptRoot
$AwcDir         = Join-Path $ProjectDir 'AWC'
$ToolchainBin   = 'D:\app\rust\rustup-home\toolchains\stable-x86_64-pc-windows-gnu\bin'
$MinGwBin       = 'D:\app\mingw64\bin'
$LibClangPath   = 'D:\app\Huawei\sdk\23\native\llvm\bin'
$FfmpegDir      = Join-Path $ProjectDir '.tools\ffmpeg81\ffmpeg-n8.1-latest-win64-gpl-shared-8.1'
$CargoHome      = Join-Path $ProjectDir '.tools\cargo-home'
$TargetDir      = Join-Path $ProjectDir '.tools\target-awc'
$OutputDir      = Join-Path $ProjectDir 'dist'
$CargoMirror    = 'sparse+http://127.0.0.1:8765/index/'
$OfflineFirst   = $true
# ============================================================================

$TargetTriple = 'x86_64-pc-windows-gnu'
$CargoExe = Join-Path $ToolchainBin 'cargo.exe'
$GccExe = Join-Path $MinGwBin 'gcc.exe'
$ArExe = Join-Path $MinGwBin 'ar.exe'
$CargoConfigDir = Join-Path $AwcDir '.cargo'
$CargoConfig = Join-Path $CargoConfigDir 'config.toml'
$ClangInclude = Join-Path $ProjectDir '.tools\clang-include'
$MinGwInclude = Join-Path (Split-Path -Parent $MinGwBin) 'x86_64-w64-mingw32\include'

function Require-File([string]$Path, [string]$Label) {
    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
        throw "$Label not found: $Path"
    }
}

function Require-Dir([string]$Path, [string]$Label) {
    if (-not (Test-Path -LiteralPath $Path -PathType Container)) {
        throw "$Label not found: $Path"
    }
}

function To-CargoPath([string]$Path) {
    return ($Path -replace '\\', '/')
}

Write-Host '=== AWC package ===' -ForegroundColor Cyan
Write-Host "project : $ProjectDir"
Write-Host "toolchain: $ToolchainBin"
Write-Host "mingw   : $MinGwBin"
Write-Host "ffmpeg  : $FfmpegDir"
Write-Host "target  : $TargetDir"
Write-Host "output  : $OutputDir"
Write-Host ''

Require-Dir $AwcDir 'AWC source directory'
Require-Dir $ToolchainBin 'Rust toolchain directory'
Require-Dir $MinGwBin 'MinGW directory'
Require-Dir $LibClangPath 'libclang directory'
Require-Dir $FfmpegDir 'FFmpeg directory'
Require-File $CargoExe 'cargo.exe'
Require-File $GccExe 'gcc.exe'
Require-File $ArExe 'ar.exe'

# Keep the Cargo config portable too. The .cargo directory is gitignored, so a
# fresh migration would otherwise silently use an absolute D:\app path.
New-Item -ItemType Directory -Force -Path $CargoConfigDir | Out-Null
$configLines = @()
if ($CargoMirror.Trim().Length -gt 0) {
    $configLines += '[source.crates-io]'
    $configLines += 'replace-with = "local-mirror"'
    $configLines += ''
    $configLines += '[source.local-mirror]'
    $configLines += "registry = `"$CargoMirror`""
    $configLines += ''
}
$configLines += "[target.$TargetTriple]"
$configLines += "linker = `"$(To-CargoPath (Join-Path $MinGwBin 'gcc.exe'))`""
$configLines += "ar = `"$(To-CargoPath (Join-Path $MinGwBin 'ar.exe'))`""
[System.IO.File]::WriteAllText(
    $CargoConfig,
    ($configLines -join "`r`n"),
    (New-Object System.Text.UTF8Encoding($false))
)

$env:CARGO_HOME = $CargoHome
$env:CARGO_TARGET_DIR = $TargetDir
$env:FFMPEG_DIR = $FfmpegDir
$env:LIBCLANG_PATH = $LibClangPath
$env:CC = $GccExe
$env:AR = $ArExe
$env:PATH = "$ToolchainBin;$MinGwBin;" + $env:PATH
$env:BINDGEN_EXTRA_CLANG_ARGS = "-I`"$(To-CargoPath $ClangInclude)`" -I`"$(To-CargoPath $MinGwInclude)`" --target=$TargetTriple"

New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null

if (-not $SkipBuild) {
    Write-Host '--- cargo build --release ---' -ForegroundColor Cyan
    Push-Location $AwcDir
    try {
        $cargoArgs = @('build', '--release', '--target', $TargetTriple, '--locked')
        if ($OfflineFirst) {
            & $CargoExe @cargoArgs --offline
            if ($LASTEXITCODE -ne 0) {
                Write-Warning 'offline build failed; retrying with the configured registry'
                & $CargoExe @cargoArgs
            }
        } else {
            & $CargoExe @cargoArgs
        }
        if ($LASTEXITCODE -ne 0) {
            throw "cargo build failed with exit code $LASTEXITCODE"
        }
    } finally {
        Pop-Location
    }
}

$releaseDir = Join-Path $TargetDir "$TargetTriple\release"
Require-Dir $releaseDir 'release output directory'
Require-File (Join-Path $releaseDir 'awc.exe') 'awc.exe'
Require-File (Join-Path $releaseDir 'softcam.dll') 'softcam.dll'

$versionLine = Select-String -LiteralPath (Join-Path $AwcDir 'Cargo.toml') `
    -Pattern '^version\s*=\s*"([^"]+)"' | Select-Object -First 1
if (-not $versionLine) { throw 'could not read version from AWC/Cargo.toml' }
$version = $versionLine.Matches[0].Groups[1].Value
$packageName = "AWC-Client-$version"
$stage = Join-Path $OutputDir $packageName

if (Test-Path -LiteralPath $stage) {
    try {
        Remove-Item -LiteralPath $stage -Recurse -Force -ErrorAction Stop
    } catch {
        throw "Cannot replace '$stage'. Close any AWC instance running from that folder, then retry. $($_.Exception.Message)"
    }
}
New-Item -ItemType Directory -Force -Path $stage | Out-Null

Write-Host '--- stage runtime files ---' -ForegroundColor Cyan
Copy-Item -LiteralPath (Join-Path $releaseDir 'awc.exe') -Destination $stage
$dlls = @(Get-ChildItem -LiteralPath $releaseDir -Filter '*.dll' -File)
if ($dlls.Count -eq 0) { throw 'no runtime DLLs found in release output' }
foreach ($dll in $dlls) {
    Copy-Item -LiteralPath $dll.FullName -Destination $stage
}

$adbDir = Join-Path $releaseDir 'adb'
if (Test-Path -LiteralPath $adbDir -PathType Container) {
    Copy-Item -LiteralPath $adbDir -Destination (Join-Path $stage 'adb') -Recurse
} else {
    Write-Warning "adb directory not found in release output; USB mode may not work"
}

$license = Join-Path $ProjectDir 'LICENSE'
if (Test-Path -LiteralPath $license) {
    Copy-Item -LiteralPath $license -Destination $stage
}

$readme = @"
AWC $version
================

Run:
  awc.exe

Useful launch options:
  awc.exe --test-pattern --start-cam
  awc.exe --wifi 192.168.2.4:8080 --start-cam
  awc.exe --wifi 192.168.2.4:8080 --start-cam --no-preview

The virtual camera is registered by awc.exe -> Setup (requires administrator).
Runtime files (softcam.dll, FFmpeg DLLs, adb\) must stay next to awc.exe.

Configuration:
  %APPDATA%\AWC\config.json

The first launch follows the Windows display language and system theme.
"@
Set-Content -LiteralPath (Join-Path $stage 'README.txt') -Value $readme -Encoding utf8

if (-not $NoZip) {
    $zip = Join-Path $OutputDir "$packageName.zip"
    if (Test-Path -LiteralPath $zip) { Remove-Item -LiteralPath $zip -Force }
    Write-Host '--- create zip ---' -ForegroundColor Cyan
    Compress-Archive -Path (Join-Path $stage '*') -DestinationPath $zip -CompressionLevel Optimal
    $hash = Get-FileHash -LiteralPath $zip -Algorithm SHA256
    $sizeMb = (Get-Item -LiteralPath $zip).Length / 1MB
    Write-Host ''
    Write-Host "OK: $zip" -ForegroundColor Green
    Write-Host ("    {0:N1} MB  SHA256 {1}" -f $sizeMb, $hash.Hash)
} else {
    Write-Host ''
    Write-Host "OK: $stage" -ForegroundColor Green
}

Write-Host "client folder: $stage"
