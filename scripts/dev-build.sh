#!/usr/bin/env bash
# Cross-toolchain-safe local build for FreEco.ai.
#
# On Windows there are two Rust toolchains:
#   * MSVC (x86_64-pc-windows-msvc) — needs Visual Studio Build Tools + link.exe
#   * GNU  (x86_64-pc-windows-gnu)  — needs MinGW-w64 (gcc, dlltool, as)
#
# rust-toolchain.toml pins `stable`, which resolves to the MSVC host by default.
# That's correct for CI (GitHub runners have MSVC), but on a machine WITHOUT the
# MSVC Build Tools the link step fails confusingly:
#   * "link: extra operand ...  Try 'link --help'"      <- coreutils `link` shadows MSVC link.exe
#   * "error calling dlltool 'dlltool.exe': program not found"  <- MinGW not on PATH
#
# This script picks a working toolchain and adds MinGW to PATH for the GNU case.
# All arguments pass straight through to cargo:
#   ./scripts/dev-build.sh -p openfang-api --lib
#   ./scripts/dev-build.sh --release -p openfang-cli
#
# NOTE: exit code is reported via ${PIPESTATUS[0]} conventions — this script does
# NOT pipe cargo through tail, so `echo $?` after it is reliable.
#
# Prefer PR CI for authoritative multi-OS verification; use this for fast local
# --lib / single-crate checks.
set -euo pipefail

# Default target if none given.
if [ "$#" -eq 0 ]; then
  set -- build --workspace --lib
fi

is_windows() { case "$(uname -s)" in MINGW*|MSYS*|CYGWIN*) return 0;; *) return 1;; esac; }

# On non-Windows just run cargo as-is.
if ! is_windows; then
  exec cargo "$@"
fi

msvc_linker_ok() {
  # A real MSVC link.exe prints "Microsoft" in its help/banner; coreutils `link` does not.
  command -v link.exe >/dev/null 2>&1 || return 1
  link.exe /? 2>&1 | grep -qi microsoft
}

find_mingw_bin() {
  local c
  for c in \
    "$LOCALAPPDATA/Programs/mingw64/bin" \
    "/c/Users/$USER/AppData/Local/Programs/mingw64/bin" \
    "/c/mingw64/bin" "/c/msys64/mingw64/bin" "/c/tools/mingw64/bin"; do
    if [ -n "${c:-}" ] && [ -f "$c/dlltool.exe" ]; then echo "$c"; return 0; fi
  done
  if command -v dlltool.exe >/dev/null 2>&1; then dirname "$(command -v dlltool.exe)"; return 0; fi
  return 1
}

GNU_TOOLCHAIN="stable-x86_64-pc-windows-gnu"

if msvc_linker_ok; then
  echo "==> MSVC linker found — building with the default (MSVC) toolchain."
  exec cargo "$@"
fi

echo "==> No usable MSVC linker — switching to the GNU toolchain."
if ! rustup toolchain list 2>/dev/null | grep -q "$GNU_TOOLCHAIN"; then
  echo "    Install it once with:  rustup toolchain install $GNU_TOOLCHAIN" >&2
  exit 1
fi

if ! MINGW_BIN="$(find_mingw_bin)"; then
  echo "    MinGW-w64 not found (need dlltool.exe). Install WinLibs MinGW-w64 or add it to PATH." >&2
  exit 1
fi
echo "    Using MinGW at: $MINGW_BIN"
export PATH="$MINGW_BIN:$PATH"

exec cargo "+$GNU_TOOLCHAIN" "$@"
