param(
    [string]$CodexHome = $(if ($env:CODEX_HOME) { $env:CODEX_HOME } else { Join-Path $env:USERPROFILE ".codex" }),
    [string]$BinDir = $(Join-Path $env:USERPROFILE ".local\bin")
)

$ErrorActionPreference = "Stop"

$Root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$SkillName = "nanda-structural-gate"
$RuntimeSkillDir = Join-Path (Join-Path $CodexHome "skills") $SkillName
$RuntimeBinDir = Join-Path $RuntimeSkillDir "bin"
$ReleaseExe = Join-Path $Root "target\release\nanda.exe"

$Commands = @(
    "check",
    "gate",
    "init-task",
    "pack-from-md",
    "init-md",
    "gate-md",
    "self-check",
    "comb",
    "doctor",
    "extract",
    "eval",
    "waw",
    "feedback",
    "index",
    "search",
    "probe",
    "dataset-doctor",
    "budget",
    "pack6m",
    "serve",
    "dogfood",
    "report",
    "split",
    "split-md",
    "map",
    "hgate"
)

New-Item -ItemType Directory -Force -Path (Join-Path $CodexHome "skills") | Out-Null
New-Item -ItemType Directory -Force -Path $BinDir | Out-Null

cargo build --release --manifest-path (Join-Path $Root "Cargo.toml")

if (Test-Path $RuntimeSkillDir) {
    Remove-Item -Recurse -Force $RuntimeSkillDir
}
Copy-Item -Recurse -Path (Join-Path $Root $SkillName) -Destination $RuntimeSkillDir
New-Item -ItemType Directory -Force -Path $RuntimeBinDir | Out-Null
Copy-Item -Force -Path $ReleaseExe -Destination (Join-Path $RuntimeBinDir "nanda.exe")

function Write-CmdWrapper {
    param(
        [string]$Path,
        [string]$ExePath,
        [string]$Subcommand
    )
    $Content = @"
@echo off
setlocal
"$ExePath" $Subcommand %*
exit /b %ERRORLEVEL%
"@
    Set-Content -Path $Path -Value $Content -Encoding ASCII
}

$NandaExe = Join-Path $RuntimeBinDir "nanda.exe"

$MainWrapper = @"
@echo off
setlocal
"$NandaExe" %*
exit /b %ERRORLEVEL%
"@
Set-Content -Path (Join-Path $BinDir "nanda.cmd") -Value $MainWrapper -Encoding ASCII

foreach ($Command in $Commands) {
    $WrapperName = "nanda-$Command.cmd"
    Write-CmdWrapper -Path (Join-Path $RuntimeSkillDir "scripts\$WrapperName") -ExePath $NandaExe -Subcommand $Command
    Write-CmdWrapper -Path (Join-Path $BinDir $WrapperName) -ExePath $NandaExe -Subcommand $Command
}

Write-Host "Installed skill: $RuntimeSkillDir"
Write-Host "Installed CLI:   $(Join-Path $BinDir "nanda.cmd")"
Write-Host "Installed tools: $BinDir\nanda-*.cmd"
Write-Host ""
Write-Host "If commands are not found, add this directory to PATH:"
Write-Host "  $BinDir"
Write-Host ""
Write-Host "Run:"
Write-Host "  nanda-doctor.cmd"
