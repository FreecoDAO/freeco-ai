# Make the FreEco USB boot Kubuntu on any UEFI machine, without erasing it.
#
# The usual advice is to write the ISO over the whole disk with dd or Rufus.
# That works and it destroys everything already there - on this stick, the
# freeco workspace and the downloaded models. It is also unnecessary.
#
# Instead: the ISO file stays on the existing exFAT partition, and the 1 GB EFI
# System Partition gets a bootloader that loop-mounts it. UEFI firmware reads
# the ESP, GRUB reads the ISO from exFAT, and the live session starts. Nothing
# is overwritten and the stick still works as a normal drive in Windows.
#
# WHAT THIS TOUCHES
#   - Writes into the ESP (partition 1). It is currently empty.
#   - Copies one ~4.3 GB file onto the exFAT partition.
#   - Does NOT repartition, format, or delete anything.
#
# WHAT IT CANNOT DO
#   - Legacy BIOS-only machines (roughly pre-2012) need an MBR boot sector this
#     does not write. UEFI machines and Intel Macs are covered.
#   - Secure Boot must be off, or the firmware will refuse GRUB.
#
# Run from an elevated PowerShell:
#   powershell -ExecutionPolicy Bypass -File scripts\make-usb-bootable.ps1

[CmdletBinding()]
param(
    [string]$IsoPath   = "$env:USERPROFILE\.openfang\iso\kubuntu-24.04.3-desktop-amd64.iso",
    [int]$DiskNumber   = 1,
    [int]$EspPartition = 1,
    [int]$DataPartition = 2
)

$ErrorActionPreference = 'Stop'

function Fail($msg) { Write-Host "FAILED: $msg" -ForegroundColor Red; exit 1 }
function Step($msg) { Write-Host "==> $msg" -ForegroundColor Cyan }

# --- Preconditions, checked before anything is written ---------------------
if (-not ([Security.Principal.WindowsPrincipal] `
        [Security.Principal.WindowsIdentity]::GetCurrent()
    ).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
    Fail "Run this from an elevated PowerShell. Mounting the ESP needs admin."
}

if (-not (Test-Path $IsoPath)) { Fail "ISO not found at $IsoPath" }
$isoSize = (Get-Item $IsoPath).Length
Step ("ISO found: {0:N0} MB" -f ($isoSize / 1MB))

$disk = Get-Disk -Number $DiskNumber -ErrorAction SilentlyContinue
if (-not $disk) { Fail "Disk $DiskNumber not found." }
if ($disk.BusType -ne 'USB') {
    Fail "Disk $DiskNumber is $($disk.BusType), not USB. Refusing to touch an internal disk."
}
Step "Target: disk $DiskNumber - $($disk.FriendlyName)"

$data = Get-Partition -DiskNumber $DiskNumber -PartitionNumber $DataPartition
if (-not $data.DriveLetter) { Fail "Data partition $DataPartition has no drive letter." }
$dataRoot = "$($data.DriveLetter):"
$freeBytes = (Get-Volume -DriveLetter $data.DriveLetter).SizeRemaining
if ($freeBytes -lt ($isoSize * 1.05)) {
    Fail ("Not enough room on {0} - need {1:N0} MB, have {2:N0} MB." -f `
        $dataRoot, ($isoSize / 1MB), ($freeBytes / 1MB))
}

# --- Mount the ESP. It has no drive letter by default. ---------------------
$espLetter = 'S'
Step "Mounting the EFI partition as ${espLetter}:"
try {
    Set-Partition -DiskNumber $DiskNumber -PartitionNumber $EspPartition `
        -NewDriveLetter $espLetter -ErrorAction Stop
} catch {
    if (-not (Test-Path "${espLetter}:\")) { Fail "Could not mount the ESP: $_" }
    Write-Host "    (already mounted)" -ForegroundColor DarkGray
}

try {
    # --- Copy the ISO onto the data partition -----------------------------
    $isoName = Split-Path $IsoPath -Leaf
    $isoDest = Join-Path $dataRoot $isoName
    if ((Test-Path $isoDest) -and ((Get-Item $isoDest).Length -eq $isoSize)) {
        Step "ISO already on $dataRoot, same size - skipping the copy"
    } else {
        Step "Copying the ISO to $dataRoot (this is the slow part)"
        Copy-Item $IsoPath $isoDest -Force
    }

    # --- Take the EFI bootloader out of the ISO itself --------------------
    # Ubuntu images ship a signed bootloader at /EFI/BOOT. Using the one from
    # the ISO guarantees GRUB and the kernel are the same build; a bootloader
    # from elsewhere is the usual cause of "kernel not found" at boot.
    Step "Mounting the ISO to extract its EFI bootloader"
    $mount = Mount-DiskImage -ImagePath $IsoPath -PassThru
    $isoDrive = ($mount | Get-Volume).DriveLetter
    if (-not $isoDrive) { Fail "Could not mount the ISO." }

    $srcEfi = "${isoDrive}:\EFI\BOOT"
    if (-not (Test-Path $srcEfi)) {
        Dismount-DiskImage -ImagePath $IsoPath | Out-Null
        Fail "No \EFI\BOOT in the ISO - is this really a UEFI Ubuntu image?"
    }

    Step "Copying the bootloader into the ESP"
    New-Item -ItemType Directory -Force -Path "${espLetter}:\EFI\BOOT" | Out-Null
    Copy-Item "$srcEfi\*" "${espLetter}:\EFI\BOOT\" -Recurse -Force

    # A 2010 MacBook looks for a different filename than PC firmware does.
    # Providing both means one stick boots both, which is the whole point.
    New-Item -ItemType Directory -Force -Path "${espLetter}:\EFI\BOOT\BOOTX64" | Out-Null
    if (Test-Path "${espLetter}:\EFI\BOOT\BOOTX64.EFI") {
        Copy-Item "${espLetter}:\EFI\BOOT\BOOTX64.EFI" `
                  "${espLetter}:\EFI\BOOT\BOOTIA32.EFI" -Force -ErrorAction SilentlyContinue
    }

    Dismount-DiskImage -ImagePath $IsoPath | Out-Null

    # --- GRUB config that loop-mounts the ISO -----------------------------
    # `nomodeset` is there for the 2010 MacBook's NVIDIA GT 330M, which shows a
    # black screen without it. It costs nothing on machines that do not need it.
    Step "Writing the GRUB menu"
    $grubDir = "${espLetter}:\boot\grub"
    New-Item -ItemType Directory -Force -Path $grubDir | Out-Null

    $cfg = @"
set timeout=10
set default=0

menuentry "Kubuntu live (recommended)" {
    insmod part_gpt
    insmod exfat
    insmod loopback
    insmod iso9660
    search --no-floppy --set=root --file /$isoName
    loopback loop /$isoName
    linux (loop)/casper/vmlinuz boot=casper iso-scan/filename=/$isoName quiet splash ---
    initrd (loop)/casper/initrd
}

menuentry "Kubuntu live - safe graphics (older Macs, NVIDIA)" {
    insmod part_gpt
    insmod exfat
    insmod loopback
    insmod iso9660
    search --no-floppy --set=root --file /$isoName
    loopback loop /$isoName
    linux (loop)/casper/vmlinuz boot=casper iso-scan/filename=/$isoName nomodeset quiet splash ---
    initrd (loop)/casper/initrd
}

menuentry "Check the disc for defects" {
    insmod part_gpt
    insmod exfat
    insmod loopback
    insmod iso9660
    search --no-floppy --set=root --file /$isoName
    loopback loop /$isoName
    linux (loop)/casper/vmlinuz boot=casper integrity-check quiet splash ---
    initrd (loop)/casper/initrd
}
"@
    Set-Content -Path "$grubDir\grub.cfg" -Value $cfg -Encoding ASCII
    Copy-Item "$grubDir\grub.cfg" "${espLetter}:\EFI\BOOT\grub.cfg" -Force

    Write-Host ""
    Write-Host "Done. The stick now boots Kubuntu and still holds your files." -ForegroundColor Green
    Write-Host ""
    Write-Host "  This laptop : F12 (or Esc) at power-on, choose the USB device"
    Write-Host "  MacBook     : hold Option/Alt at the chime, choose 'EFI Boot'"
    Write-Host ""
    Write-Host "  On the 2010 MacBook pick the 'safe graphics' entry - the"
    Write-Host "  GT 330M shows a black screen without nomodeset."
    Write-Host ""
    Write-Host "  Secure Boot must be OFF, or the firmware will reject GRUB."
    Write-Host "  52 GB of this stick is unallocated, so you can install"
    Write-Host "  Kubuntu onto it from the live session and keep the rest."
}
finally {
    # Always give the ESP its letter back, even if something above failed.
    Step "Unmounting the ESP"
    Remove-PartitionAccessPath -DiskNumber $DiskNumber -PartitionNumber $EspPartition `
        -AccessPath "${espLetter}:\" -ErrorAction SilentlyContinue
}
