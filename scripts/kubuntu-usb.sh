#!/usr/bin/env bash
# Safely verify and write an official Kubuntu live ISO for FreEco.ai USB use.
# This script never downloads an ISO, installs software, or writes a drive
# until the operator has inspected the target and typed its full device path.
#
# Usage:
#   scripts/kubuntu-usb.sh verify kubuntu.iso SHA256SUMS
#   sudo scripts/kubuntu-usb.sh write kubuntu.iso SHA256SUMS /dev/sdX

set -euo pipefail

usage() {
    echo "Usage: $0 verify <kubuntu.iso> <SHA256SUMS>"
    echo "       sudo $0 write <kubuntu.iso> <SHA256SUMS> </dev/device>"
    exit 2
}

[ "$#" -ge 3 ] || usage
MODE="$1"
ISO="$2"
SUMS="$3"
TARGET="${4:-}"

[ -f "$ISO" ] || { echo "ISO not found: $ISO" >&2; exit 1; }
[ -f "$SUMS" ] || { echo "Checksum file not found: $SUMS" >&2; exit 1; }
command -v sha256sum >/dev/null || { echo "sha256sum is required." >&2; exit 1; }

ISO_NAME="$(basename "$ISO")"
(cd "$(dirname "$ISO")" && grep -F " $ISO_NAME" "$SUMS" | sha256sum -c -)
echo "Verified official ISO checksum: $ISO_NAME"

if [ "$MODE" = "verify" ]; then
    exit 0
fi
[ "$MODE" = "write" ] && [ -n "$TARGET" ] || usage
[ "$(id -u)" -eq 0 ] || { echo "Writing a USB requires sudo." >&2; exit 1; }
[ -b "$TARGET" ] || { echo "Target is not a block device: $TARGET" >&2; exit 1; }
ROOT_SOURCE="$(findmnt -n -o SOURCE / 2>/dev/null || true)"
ROOT_PARENT="$(lsblk -no PKNAME "$ROOT_SOURCE" 2>/dev/null || true)"
if [ -n "$ROOT_PARENT" ] && [ "/dev/$ROOT_PARENT" = "$TARGET" ]; then
    echo "Refusing to overwrite the device that contains the running system." >&2
    exit 1
fi

echo
echo "WARNING: every file on $TARGET will be permanently erased."
lsblk -o NAME,SIZE,MODEL,TRAN,MOUNTPOINTS "$TARGET"
read -r -p "Type the complete target path ($TARGET) to continue: " CONFIRM
[ "$CONFIRM" = "$TARGET" ] || { echo "Cancelled." >&2; exit 1; }
read -r -p "Type WRITE KUBUNTU USB to confirm the destructive write: " CONFIRM
[ "$CONFIRM" = "WRITE KUBUNTU USB" ] || { echo "Cancelled." >&2; exit 1; }

umount "${TARGET}"?* 2>/dev/null || true
dd if="$ISO" of="$TARGET" bs=4M conv=fsync status=progress
sync
echo "Kubuntu USB written successfully. Boot it using your firmware's USB boot menu."
