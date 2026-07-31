#!/usr/bin/env bash
# Single-flight, resumable ISO fetcher.
#
# Two lessons are baked in here, both learned the expensive way:
#   1. A partially downloaded 4 GB file must live on durable storage. Temp
#      directories get cleaned and take the whole download with them.
#   2. Never let a second downloader touch the same file. Two curls on one
#      target do not go twice as fast; they halve each other's throughput and
#      burn double the bytes. The pid lock below makes that impossible.
#
# Usage: fetch-iso.sh <url> <dest>
set -u

URL="${1:?usage: fetch-iso.sh <url> <dest>}"
DEST="${2:?usage: fetch-iso.sh <url> <dest>}"
LOCK="${DEST}.lock"

mkdir -p "$(dirname "$DEST")" || exit 1

# Single-flight guard. A stale lock from a killed process is reclaimed;
# a live one means we exit rather than compete.
if [ -f "$LOCK" ]; then
    OLD="$(cat "$LOCK" 2>/dev/null || true)"
    if [ -n "$OLD" ] && kill -0 "$OLD" 2>/dev/null; then
        echo "already downloading under pid $OLD - not starting a second one"
        exit 0
    fi
    echo "clearing stale lock from pid ${OLD:-unknown}"
    rm -f "$LOCK"
fi
echo "$$" > "$LOCK"
trap 'rm -f "$LOCK"' EXIT INT TERM

EXPECTED="$(curl -sIL --max-time 60 "$URL" \
    | grep -i '^content-length' | tail -1 | tr -dc '0-9')"
if [ -z "$EXPECTED" ]; then
    echo "could not determine size of $URL"
    exit 1
fi
echo "expecting $EXPECTED bytes"

# --continue-at resumes; the retry flags survive the drop-outs that make a
# multi-hour download on a home connection fail somewhere in the middle.
for attempt in 1 2 3 4 5 6 7 8 9 10; do
    HAVE=$(stat -c %s "$DEST" 2>/dev/null || echo 0)
    if [ "$HAVE" -ge "$EXPECTED" ]; then break; fi
    echo "attempt $attempt: have $HAVE / $EXPECTED"
    curl -L --continue-at - --retry 10 --retry-delay 15 --retry-all-errors \
        --connect-timeout 30 --speed-limit 1024 --speed-time 120 \
        -o "$DEST" "$URL"
    sleep 5
done

FINAL=$(stat -c %s "$DEST" 2>/dev/null || echo 0)
if [ "$FINAL" -eq "$EXPECTED" ]; then
    echo "OK: $DEST is complete ($FINAL bytes)"
    exit 0
fi
# A truncated ISO looks like a real file and fails only at boot, so say so
# loudly rather than let it be mistaken for a finished download.
echo "INCOMPLETE: $FINAL of $EXPECTED bytes. Re-run this script to resume."
exit 1
