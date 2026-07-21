<#
.SYNOPSIS
  Cross-toolchain-safe local build for FreEco.ai on Windows.

.DESCRIPTION
  On Windows there are two Rust toolchains:
    * MSVC (`x86_64-pc-windows-msvc`) — needs Visual Studio Build Tools + link.exe
    * GNU  (`x86_64-pc-windows-gnu`)  — needs MinGW-w64 (gcc, dlltool, as)

  `rust-toolchain.toml` pins `stable`, which resolves to the MSVC host by default.
  That is correct for CI (GitHub runners ship MSVC), but on a machine WITHOUT the
  MSVC Build Tools the link step fails in confusing ways:
    * `link: extra operand ...  Try 'link --help'`  <- Git's coreutils `link` shadows MSVC's link.exe
    * `error calling dlltool 'dlltool.exe': program not found`  <- MinGW not on PATH

  This script detects a usable toolchain and builds with it, adding MinGW to PATH
  when the GNU toolchain is used. It passes all extra args straight to cargo, so:
      ./scripts/dev-build.ps1 -p openfang-api --lib
      ./scripts/dev-build.ps1 --release -p openfang-cli

.NOTES
  Prefer PR CI for authoritative multi-OS verification. Use this for fast local
  `--lib` / single-crate checks. Full local release builds are slow.
#>
param(
  [Parameter(ValueFromRemainingArguments = $true)]
  [string[]] $CargoArgs
)

$ErrorActionPreference = 'Stop'
if (-not $CargoArgs -or $CargoArgs.Count -eq 0) {
  $CargoArgs = @('build', '--workspace', '--lib')
}

function Test-MsvcLinker {
  # A real MSVC link.exe reports "Microsoft (R) Incremental Linker".
  $link = Get-Command link.exe -ErrorAction SilentlyContinue
  if (-not $link) { return $false }
  try { return ((& $link.Source /? 2>&1 | Out-String) -match 'Microsoft') } catch { return $false }
}

function Find-MinGwBin {
  $candidates = @(
    (Join-Path $env:LOCALAPPDATA 'Programs\mingw64\bin'),
    'C:\mingw64\bin', 'C:\msys64\mingw64\bin', 'C:\tools\mingw64\bin'
  )
  foreach ($c in $candidates) {
    if ($c -and (Test-Path (Join-Path $c 'dlltool.exe'))) { return $c }
  }
  $onPath = Get-Command dlltool.exe -ErrorAction SilentlyContinue
  if ($onPath) { return (Split-Path $onPath.Source) }
  return $null
}

$gnuToolchain = 'stable-x86_64-pc-windows-gnu'
$hasGnu = (& rustup toolchain list 2>$null) -match [regex]::Escape($gnuToolchain)

if (Test-MsvcLinker) {
  Write-Host '==> MSVC linker found — building with the default (MSVC) toolchain.' -ForegroundColor Cyan
  & cargo @CargoArgs
  exit $LASTEXITCODE
}

Write-Host '==> No usable MSVC linker — switching to the GNU toolchain.' -ForegroundColor Yellow
if (-not $hasGnu) {
  Write-Host "    Install it once with:  rustup toolchain install $gnuToolchain" -ForegroundColor Yellow
  throw "GNU toolchain '$gnuToolchain' is not installed."
}

$mingw = Find-MinGwBin
if (-not $mingw) {
  throw "MinGW-w64 not found (need dlltool.exe). Install WinLibs MinGW-w64 and re-run, or add it to PATH."
}
Write-Host "    Using MinGW at: $mingw" -ForegroundColor DarkGray
$env:PATH = "$mingw;$env:PATH"

& cargo "+$gnuToolchain" @CargoArgs
exit $LASTEXITCODE
