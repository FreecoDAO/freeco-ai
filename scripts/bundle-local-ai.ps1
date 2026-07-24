<#
.SYNOPSIS
    Pack Ollama + a local model onto a FreEco.ai portable/USB bundle so the
    private AI works fully offline — no download, no install on the host.

.DESCRIPTION
    Run this ONCE on a machine that already has Ollama and the model pulled
    (e.g. `ollama pull gemma4:e4b`). It copies the Ollama binary and the model
    blobs into the bundle, next to the FreEco.ai binaries. The portable
    launcher then starts Ollama from the drive with OLLAMA_MODELS pointed at
    the bundled models folder.

    Result: plug the stick into any Windows machine, run the launcher, and a
    private AI is running with zero setup and zero network.

.PARAMETER BundleDir
    The portable bundle root (the folder containing run-windows.bat).

.PARAMETER Model
    Model to verify is present, e.g. gemma4:e4b.

.EXAMPLE
    powershell -ExecutionPolicy Bypass -File scripts\bundle-local-ai.ps1 -BundleDir dist\portable-test -Model gemma4:e4b
#>
param(
    [Parameter(Mandatory = $true)][string]$BundleDir,
    [string]$Model = "gemma4:e4b"
)

$ErrorActionPreference = "Stop"

if (-not (Test-Path $BundleDir)) { throw "Bundle dir not found: $BundleDir" }

# 1. Locate Ollama on this machine.
$ollamaExe = @(
    "$env:LOCALAPPDATA\Programs\Ollama\ollama.exe",
    "$env:ProgramFiles\Ollama\ollama.exe"
) | Where-Object { Test-Path $_ } | Select-Object -First 1
if (-not $ollamaExe) { throw "ollama.exe not found. Install Ollama and pull the model first." }

# 2. Locate the model store.
$modelsSrc = if ($env:OLLAMA_MODELS) { $env:OLLAMA_MODELS } else { "$env:USERPROFILE\.ollama\models" }
if (-not (Test-Path "$modelsSrc\manifests")) {
    throw "No models found at $modelsSrc. Run: ollama pull $Model"
}

# 3. Verify the requested model is actually present (don't ship an empty brain).
$manifestHit = Get-ChildItem -Path "$modelsSrc\manifests" -Recurse -File -ErrorAction SilentlyContinue |
    Where-Object { $_.FullName -match [regex]::Escape(($Model -split ':')[0]) }
if (-not $manifestHit) {
    throw "Model '$Model' is not in $modelsSrc. Run: ollama pull $Model"
}

# 4. Copy Ollama binary.
$ollamaDest = Join-Path $BundleDir "ollama\windows"
New-Item -ItemType Directory -Force -Path $ollamaDest | Out-Null
Copy-Item $ollamaExe -Destination $ollamaDest -Force
Write-Host "Copied ollama.exe -> $ollamaDest"

# 5. Copy the model store (this is the big one — several GB).
$modelsDest = Join-Path $BundleDir "models"
New-Item -ItemType Directory -Force -Path $modelsDest | Out-Null
Write-Host "Copying model store (this takes a while — several GB)..."
Copy-Item -Path "$modelsSrc\*" -Destination $modelsDest -Recurse -Force
$sizeGb = [math]::Round(((Get-ChildItem $modelsDest -Recurse -File | Measure-Object Length -Sum).Sum / 1GB), 2)
Write-Host "Copied model store -> $modelsDest ($sizeGb GB)"

# 6. Point the bundle's config at the local model so it works on first run.
$cfg = Join-Path $BundleDir "data\config.toml"
if (Test-Path $cfg) {
    $content = Get-Content $cfg -Raw
    if ($content -notmatch '(?m)^\[default_model\]') {
        Add-Content -Path $cfg -Encoding utf8 -Value @"

[default_model]
provider = "ollama"
model = "$Model"
api_key_env = ""
base_url = "http://127.0.0.1:11434/v1"
"@
        Write-Host "Set [default_model] -> ollama/$Model in data\config.toml"
    } else {
        Write-Host "config.toml already has [default_model] — left unchanged"
    }
}

Write-Host ""
Write-Host "Done. This bundle now runs a private AI with no network and no install."
Write-Host "Copy '$BundleDir' to a USB stick and run run-windows.bat on any PC."
