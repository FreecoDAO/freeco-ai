#!/usr/bin/env bash
# Fetch a GGUF model exactly once, with real resume and no duplicate downloaders.
#
# Why this exists: a slow link plus two concurrent downloaders wasted ~25 GB of
# traffic and three days. Ollama's blob CDN ignores HTTP Range (it answers 200,
# not 206), so an interrupted pull restarts from zero forever. HuggingFace does
# support Range, so `curl -C -` genuinely resumes — but only if exactly ONE
# process is writing the file.
#
# Guarantees:
#   1. single-flight  — refuses to start if a downloader already holds the lock
#   2. true resume    — `curl -C -` continues from the exact byte
#   3. verification   — checks the final size against the server's Content-Length
#
# Usage: scripts/fetch-model.sh <url> <dest-file>
set -euo pipefail

URL="${1:?usage: fetch-model.sh <url> <dest>}"
DEST="${2:?usage: fetch-model.sh <url> <dest>}"
LOCK="${DEST}.lock"

mkdir -p "$(dirname "$DEST")"

# 1. Single-flight: never run two downloaders against the same file.
if [ -f "$LOCK" ]; then
  pid=$(cat "$LOCK" 2>/dev/null || echo "")
  if [ -n "$pid" ] && kill -0 "$pid" 2>/dev/null; then
    echo "already downloading (pid $pid) — not starting a second one." >&2
    exit 0
  fi
  echo "stale lock (pid ${pid:-none} gone), taking over."
  rm -f "$LOCK"
fi
echo $$ > "$LOCK"
trap 'rm -f "$LOCK"' EXIT INT TERM

# 2. Expected size, so we can verify completion rather than guess.
expected=$(curl -sIL "$URL" | tr -d '\r' | awk 'tolower($1)=="content-length:"{n=$2} END{print n+0}')
have=$(stat -c %s "$DEST" 2>/dev/null || echo 0)
echo "have ${have} bytes; expected ${expected:-unknown}"

if [ "${expected:-0}" -gt 0 ] && [ "$have" -ge "$expected" ]; then
  echo "already complete."
  exit 0
fi

# 3. Resume. --retry-all-errors keeps a flaky link going; -C - resumes exactly.
curl -L -C - --retry 999 --retry-delay 5 --retry-all-errors \
     --speed-time 120 --speed-limit 1024 \
     -o "$DEST" "$URL"

# 4. Verify — a truncated model fails to load with a confusing error later.
final=$(stat -c %s "$DEST" 2>/dev/null || echo 0)
if [ "${expected:-0}" -gt 0 ] && [ "$final" -lt "$expected" ]; then
  echo "INCOMPLETE: ${final}/${expected} bytes. Re-run to resume." >&2
  exit 1
fi
echo "complete: ${final} bytes -> $DEST"
