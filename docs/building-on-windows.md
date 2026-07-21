# Building on Windows (MSVC vs GNU toolchain)

Most contributors and CI build FreEco.ai with the default **MSVC** Rust toolchain and it
"just works." If you are on a Windows machine **without** the Visual Studio Build Tools (no
admin rights, no MSVC), you will hit confusing linker errors. This page explains why and
gives you a one-command fix.

## The two Windows toolchains

| Toolchain | Rust target | Needs |
|-----------|-------------|-------|
| **MSVC** (default) | `x86_64-pc-windows-msvc` | Visual Studio Build Tools (`link.exe`) |
| **GNU** | `x86_64-pc-windows-gnu` | MinGW-w64 (`gcc`, `dlltool`, `as`) |

`rust-toolchain.toml` pins `channel = "stable"`, which resolves to the **MSVC host** by
default. That is intentional — GitHub's CI runners ship MSVC, so release installers are built
with it. It only becomes a problem on a local machine that has no MSVC.

## Symptoms (and what they really mean)

- **`link: extra operand '…rcgu.o'  Try 'link --help'`**
  rustc is calling `link.exe`, but in Git Bash the GNU coreutils `link` command shadows
  MSVC's linker on `PATH`. → You have no working MSVC linker.

- **`error calling dlltool 'dlltool.exe': program not found`**
  You switched to the GNU toolchain, but MinGW-w64's binutils aren't on `PATH`.

## The fix

### One command (recommended)

Use the wrapper scripts — they detect a usable toolchain, and when they fall back to GNU they
add MinGW to `PATH` for you. All arguments pass straight through to `cargo`:

```bash
# Git Bash / MSYS2
./scripts/dev-build.sh -p openfang-api --lib
./scripts/dev-build.sh --release -p openfang-cli
```

```powershell
# PowerShell
./scripts/dev-build.ps1 -p openfang-api --lib
./scripts/dev-build.ps1 --release -p openfang-cli
```

### Manual setup (one time)

1. Install the GNU toolchain:
   ```bash
   rustup toolchain install stable-x86_64-pc-windows-gnu
   ```
2. Install **MinGW-w64** (e.g. [WinLibs](https://winlibs.com/)) — no admin needed; unzip it
   somewhere like `%LOCALAPPDATA%\Programs\mingw64`.
3. Build with the GNU toolchain, with MinGW's `bin` on `PATH`:
   ```bash
   export PATH="$LOCALAPPDATA/Programs/mingw64/bin:$PATH"
   cargo +stable-x86_64-pc-windows-gnu build -p openfang-api --lib
   ```

> Do **not** commit a `.cargo/config.toml` that forces the GNU linker globally — that would
> change how CI and MSVC contributors build. Keep the override local (the scripts above, or
> your shell), not in the repo.

## Gotchas

- **Never pipe `cargo` through `tail`/`head` to check success** — the pipeline's exit code is
  the pager's, not cargo's, so a failed build looks like it passed. Redirect to a file and
  check `$?`, or use `${PIPESTATUS[0]}` in bash.
- **`openfang.exe` is locked while the daemon runs.** A rebuild can't overwrite it. Stop the
  daemon first, or use `--lib` for a quick compile check.
- **Full local release builds are slow** (tens of minutes) and can starve the machine. Prefer
  PR CI (3-OS build/test/clippy) as the authoritative check; use local builds for fast
  `--lib`/single-crate iteration.
