# Plug-and-play USB — FreEco.ai with a private AI onboard

The safest and simplest way to run FreEco.ai: a USB stick that carries the app,
the AI runtime, **and the model**. Plug it into any machine, double-click the
launcher, and a private assistant is running — **no install, no account, no
network**. Nothing is written to the host machine; everything (config, data,
model) lives on the stick.

## Why this is the safest option
- **Nothing is installed** on the host — no admin rights, no registry, no leftovers.
- **No network needed** — the model is already on the stick, so nothing is downloaded and nothing about your work leaves the drive.
- **Your data travels with you** — config, agents, and the encrypted vault are in `\data` on the stick.
- **Unplug = gone.** Ideal for a nonprofit handling sensitive donor material on shared machines.

## Build the stick (once, on a machine that has the model)

```bash
# 1. Assemble the portable bundle (binaries + launchers)
bash scripts/build-portable.sh

# 2. Make sure the model exists on this machine
ollama pull gemma4:e4b
```

```powershell
# 3. Pack Ollama + the model onto the bundle
powershell -ExecutionPolicy Bypass -File scripts\bundle-local-ai.ps1 `
    -BundleDir dist\portable-test -Model gemma4:e4b
```

Then copy the bundle folder to the USB stick. Final layout:

```
FreEco.ai/
  run-windows.bat          <- double-click this
  run-linux.sh  run-macos.command
  bin/<os>/<arch>/openfang(.exe)
  ollama/windows/ollama.exe    <- AI runtime, from the stick
  models/                      <- the model itself (~3 GB)
  data/                        <- your config, agents, vault
```

## Use it
Double-click **`run-windows.bat`**. The launcher:
1. points `OLLAMA_MODELS` at `\models` on the stick,
2. starts the bundled Ollama (or reuses one already running),
3. starts FreEco.ai with `OPENFANG_HOME` set to `\data`,
4. opens the dashboard.

Sizing: the stick needs roughly **model size + 500 MB**. `gemma4:e4b` ≈ 3 GB, so
an 8 GB stick is comfortable; use 16 GB+ if you want a larger model.

> Tip: pick the model to match the weakest machine you'll plug into.
> `gemma4:e4b` needs ~8 GB RAM; `gemma3n:e2b` runs on ~6 GB; `llama3.2:1b` on ~4 GB.
