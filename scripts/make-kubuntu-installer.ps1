# Build a Kubuntu live installer inside the USB's unallocated space.
#
# Why this works without wiping anything: the Kubuntu ISO is UEFI-bootable by
# plain file copy. Firmware boots \EFI\BOOT\BOOTX64.EFI from any FAT32
# partition, so there is no need to dd the ISO over the whole disk. The
# largest file in the ISO (casper/filesystem.squashfs, 3720 MB) fits under
# FAT32's 4 GB per-file limit, which is the only thing that could have
# stopped this.
#
# It writes ONLY into unallocated space. The ESP and the exFAT data partition
# are never opened, formatted, or resized. If anything goes wrong, delete the
# new partition and the disk is exactly as it was.
#
# Run from an elevated PowerShell:
#   powershell -ExecutionPolicy Bypass -File scripts\make-kubuntu-installer.ps1

[CmdletBinding()]
param(
    [string] $IsoPath      = "$env:USERPROFILE\.freeco-ai\iso\kubuntu-24.04.3-desktop-amd64.iso",
    [int]    $InstallerGb  = 6,
    # Guard: refuse to touch anything that is not this removable disk.
    [string] $ExpectedBus  = 'USB'
)

$ErrorActionPreference = 'Stop'

function Fail($msg) { Write-Host "FAILED: $msg" -ForegroundColor Red; exit 1 }
function Step($msg) { Write-Host "==> $msg" -ForegroundColor Cyan }

if (-not ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()
        ).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
    Fail 'Run this from an elevated PowerShell (Run as administrator).'
}

if (-not (Test-Path $IsoPath)) { Fail "ISO not found at $IsoPath" }

# ---- Pick the disk, and refuse anything that is not clearly the USB --------
# Selecting the wrong disk here would be catastrophic, so every assumption is
# checked rather than trusted: removable bus, has free space, and is not the
# system disk.
Step 'Locating the USB disk'
$candidates = Get-Disk | Where-Object {
    $_.BusType -eq $ExpectedBus -and -not $_.IsSystem -and -not $_.IsBoot
}
if ($candidates.Count -eq 0) { Fail "No $ExpectedBus disk found." }
if ($candidates.Count -gt 1) {
    $candidates | Select-Object Number, FriendlyName, @{n='SizeGB';e={[math]::Round($_.Size/1GB,1)}} | Format-Table
    Fail 'More than one USB disk is attached. Detach the others so the target is unambiguous.'
}
$disk = $candidates[0]
if ($disk.IsBoot -or $disk.IsSystem) { Fail 'Refusing: target looks like the system disk.' }

$freeGb = [math]::Round($disk.LargestFreeExtent / 1GB, 1)
Write-Host ("    disk {0}: {1}  {2} GB total, {3} GB unallocated" -f `
    $disk.Number, $disk.FriendlyName, [math]::Round($disk.Size/1GB,1), $freeGb)

if ($disk.LargestFreeExtent -lt ($InstallerGb + 0.3) * 1GB) {
    Fail "Only $freeGb GB unallocated; need about $InstallerGb GB."
}

# Record what exists now, so we can prove afterwards that it is untouched.
$before = Get-Partition -DiskNumber $disk.Number |
    Select-Object PartitionNumber, DriveLetter, Size, Type
Write-Host '    existing partitions (these will not be modified):'
$before | ForEach-Object {
    Write-Host ("      #{0} {1}: {2} GB {3}" -f $_.PartitionNumber, $_.DriveLetter,
        [math]::Round($_.Size/1GB,1), $_.Type)
}

# ---- Create and format the installer partition ----------------------------
Step "Creating a ${InstallerGb} GB FAT32 partition in the unallocated space"
$part = New-Partition -DiskNumber $disk.Number -Size ($InstallerGb * 1GB) -AssignDriveLetter
Start-Sleep -Seconds 2
$vol = Format-Volume -Partition $part -FileSystem FAT32 `
    -NewFileSystemLabel 'KUBUNTU' -Confirm:$false
Start-Sleep -Seconds 2
$dest = "$($vol.DriveLetter):"
Write-Host "    installer partition is $dest"

# UEFI firmware only scans partitions flagged as ESP on some machines. Setting
# the type makes the stick appear in the boot menu on stricter firmware,
# including Macs, instead of being silently skipped.
try {
    Set-Partition -DriveLetter $vol.DriveLetter `
        -GptType '{c12a7328-f81f-11d2-ba4b-00a0c93ec93b}'
    Write-Host '    marked as EFI System Partition'
} catch {
    Write-Host '    could not set ESP type; most firmware will still boot it' -ForegroundColor Yellow
}

# ---- Copy the ISO contents ------------------------------------------------
Step 'Mounting the ISO'
$img = Mount-DiskImage -ImagePath $IsoPath -PassThru
Start-Sleep -Seconds 3
$src = "$(($img | Get-Volume).DriveLetter):"
Write-Host "    mounted at $src"

try {
    Step "Copying installer files to $dest (a few minutes)"
    # /J uses unbuffered I/O, which matters for multi-GB files onto USB.
    robocopy "$src\" "$dest\" /E /J /NFL /NDL /NJH /NP /R:2 /W:2 | Out-Null
    if ($LASTEXITCODE -ge 8) { Fail "robocopy reported errors (exit $LASTEXITCODE)" }
    Write-Host "    copy finished (robocopy exit $LASTEXITCODE)"
} finally {
    Dismount-DiskImage -ImagePath $IsoPath | Out-Null
    Write-Host '    ISO dismounted'
}

# ---- Verify ---------------------------------------------------------------
# A stick that looks finished but will not boot is the failure mode worth
# catching here, not at the boot menu of a machine with no other OS.
Step 'Verifying'
$ok = $true
foreach ($f in @('EFI\BOOT\BOOTX64.EFI', 'casper\vmlinuz', 'casper\initrd',
                 'casper\filesystem.squashfs')) {
    if (Test-Path "$dest\$f") {
        Write-Host "    present: $f"
    } else {
        Write-Host "    MISSING: $f" -ForegroundColor Red; $ok = $false
    }
}

$after = Get-Partition -DiskNumber $disk.Number |
    Where-Object { $_.PartitionNumber -in $before.PartitionNumber }
foreach ($b in $before) {
    $a = $after | Where-Object PartitionNumber -eq $b.PartitionNumber
    if (-not $a -or $a.Size -ne $b.Size) {
        Write-Host "    ALTERED: partition $($b.PartitionNumber)" -ForegroundColor Red
        $ok = $false
    }
}
if ($ok) { Write-Host '    your existing partitions are unchanged' }

if (-not $ok) { Fail 'Verification failed. Delete the new partition and retry.' }

Write-Host ''
Write-Host 'Installer ready.' -ForegroundColor Green
Write-Host ''
Write-Host 'To install Kubuntu:'
Write-Host '  1. Reboot and pick the USB from the firmware boot menu.'
Write-Host '     Windows: hold Shift while clicking Restart, then Use a device.'
Write-Host '     MacBook A1286: hold Option/Alt at the chime and choose EFI Boot.'
Write-Host '  2. On the MacBook, press e at the GRUB menu and add nomodeset to'
Write-Host '     the linux line. The 2010 NVIDIA GT 330M hangs on a black screen'
Write-Host '     without it.'
Write-Host '  3. Choose "Something else" at the disk step, NOT "Erase disk".'
Write-Host ("     Create your root partition in the remaining ~{0} GB of free space." -f `
    ([math]::Round(($disk.LargestFreeExtent/1GB) - $InstallerGb, 0)))
Write-Host '     Leave the 1 GB ESP and the 62 GB exFAT partition alone; point the'
Write-Host '     bootloader at the existing ESP.'
Write-Host '  4. Broadcom WiFi on the MacBook needs bcmwl-kernel-source, so keep'
Write-Host '     an ethernet cable or a phone tether handy for the first boot.'
Write-Host ''
Write-Host 'When the install is done you can delete the KUBUNTU installer'
Write-Host 'partition and grow the Linux root into the space it frees.'
