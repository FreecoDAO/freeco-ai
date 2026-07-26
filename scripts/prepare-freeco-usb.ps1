<#
.SYNOPSIS
  Partition a USB disk into the FreEco.ai plug-and-play layout:
  bootable Kubuntu + a model store that BOTH Windows and Linux can use.

.DESCRIPTION
  Target layout on a >=64 GB USB disk (GPT):

    1. ESP        FAT32   1 GB    UEFI boot (GRUB / Ventoy / Kubuntu bootloader)
    2. FREECO     exFAT   ~55%    Models + portable FreEco.ai.
                                  exFAT because FAT32 caps files at 4 GB (a
                                  Gemma 4 E4B GGUF is ~9.6 GB and would NOT fit),
                                  and because Windows, macOS and Linux all
                                  read/write exFAT natively — one model file,
                                  usable from Windows AND from booted Kubuntu.
    3. (unallocated)      rest    Left free ON PURPOSE: the Kubuntu installer
                                  creates its own ext4 root here. Windows cannot
                                  create ext4, so we must not fake it.

  SAFETY: refuses to touch anything that is not a removable USB disk, refuses
  the disk that hosts the running Windows volume, and does nothing at all
  unless -Execute is passed (dry-run by default).

.EXAMPLE
  # 1. See the plan without changing anything:
  .\prepare-freeco-usb.ps1 -DiskNumber 1

  # 2. Actually do it (run PowerShell as Administrator):
  .\prepare-freeco-usb.ps1 -DiskNumber 1 -Execute
#>
[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [int]$DiskNumber,

    # Size of the shared exFAT model partition. Default: 55% of the disk.
    [int]$DataPercent = 55,

    # Without this, the script only prints the plan.
    [switch]$Execute
)

$ErrorActionPreference = 'Stop'

function Fail($msg) { Write-Host "ERROR: $msg" -ForegroundColor Red; exit 1 }
function Note($msg) { Write-Host $msg -ForegroundColor Cyan }
function Ok($msg)   { Write-Host "OK: $msg" -ForegroundColor Green }

# ---------------------------------------------------------------- guards ----
$id = [Security.Principal.WindowsIdentity]::GetCurrent()
$principal = New-Object Security.Principal.WindowsPrincipal($id)
if (-not $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
    Fail "Not elevated. Right-click PowerShell -> 'Run as Administrator', then re-run."
}

$disk = Get-Disk -Number $DiskNumber -ErrorAction SilentlyContinue
if (-not $disk) { Fail "No disk number $DiskNumber. Run Get-Disk to list them." }

# Never touch an internal disk.
if ($disk.BusType -ne 'USB') {
    Fail "Disk $DiskNumber is BusType '$($disk.BusType)', not USB. Refusing — this script only prepares removable USB media."
}

# Never touch the disk Windows is running from.
$sysDiskNumber = (Get-Partition -DriveLetter $env:SystemDrive.Trim(':') -ErrorAction SilentlyContinue).DiskNumber
if ($sysDiskNumber -eq $DiskNumber) { Fail "Disk $DiskNumber hosts $env:SystemDrive. Refusing." }

$sizeGB = [math]::Round($disk.Size / 1GB, 1)
if ($sizeGB -lt 32) { Fail "Disk is only $sizeGB GB. Need at least 32 GB (64 GB+ recommended)." }

# ------------------------------------------------------------- the plan ----
$espMB    = 1024
$dataGB   = [math]::Floor(($sizeGB - 1) * $DataPercent / 100)
$linuxGB  = [math]::Round($sizeGB - 1 - $dataGB, 1)

Write-Host ""
Note "==================== FreEco.ai USB layout ===================="
Note "Disk $DiskNumber : $($disk.FriendlyName)  ($sizeGB GB, $($disk.BusType))"
Write-Host ""
Note "  1. ESP            FAT32   1 GB        UEFI boot"
Note "  2. FREECO         exFAT   $dataGB GB      models + portable app (Windows AND Linux)"
Note "  3. (unallocated)          $linuxGB GB      for Kubuntu's ext4 root, made by its installer"
Write-Host ""

Write-Host "Volumes that will be ERASED:" -ForegroundColor Yellow
Get-Partition -DiskNumber $DiskNumber -ErrorAction SilentlyContinue |
    Where-Object DriveLetter |
    ForEach-Object {
        $v = Get-Volume -DriveLetter $_.DriveLetter -ErrorAction SilentlyContinue
        "   {0}: {1} ({2} GB used of {3} GB)" -f $_.DriveLetter, $v.FileSystemLabel,
            [math]::Round(($v.Size - $v.SizeRemaining)/1GB,1), [math]::Round($v.Size/1GB,1)
    }
Write-Host ""

if (-not $Execute) {
    Write-Host "DRY RUN — nothing changed. Re-run with -Execute to apply." -ForegroundColor Yellow
    exit 0
}

Write-Host "This ERASES EVERYTHING on disk $DiskNumber." -ForegroundColor Red
$answer = Read-Host "Type the word ERASE to continue"
if ($answer -ne 'ERASE') { Fail "Cancelled (you typed '$answer')." }

# ---------------------------------------------------------------- apply ----
Note "Clearing disk $DiskNumber ..."
Clear-Disk -Number $DiskNumber -RemoveData -RemoveOEM -Confirm:$false
Initialize-Disk -Number $DiskNumber -PartitionStyle GPT -Confirm:$false
Ok "Disk cleared and initialised as GPT."

Note "Creating ESP (FAT32, $espMB MB) ..."
$esp = New-Partition -DiskNumber $DiskNumber -Size ($espMB * 1MB) -GptType '{c12a7328-f81f-11d2-ba4b-00a0c93ec93b}'
# ESP has no drive letter by default; give it one only long enough to format.
$esp | Add-PartitionAccessPath -AssignDriveLetter -ErrorAction SilentlyContinue
Start-Sleep -Seconds 2
$espLetter = (Get-Partition -DiskNumber $DiskNumber -PartitionNumber $esp.PartitionNumber).DriveLetter
if ($espLetter) {
    Format-Volume -DriveLetter $espLetter -FileSystem FAT32 -NewFileSystemLabel 'FREECO-EFI' -Confirm:$false | Out-Null
    Ok "ESP formatted FAT32 as ${espLetter}: (FREECO-EFI)."
} else {
    Ok "ESP created (no letter assigned — normal; the Linux installer will use it)."
}

Note "Creating shared data partition (exFAT, $dataGB GB) ..."
$data = New-Partition -DiskNumber $DiskNumber -Size ($dataGB * 1GB) -AssignDriveLetter
Start-Sleep -Seconds 2
Format-Volume -DriveLetter $data.DriveLetter -FileSystem exFAT -NewFileSystemLabel 'FREECO' -Confirm:$false | Out-Null
Ok "Data partition formatted exFAT as $($data.DriveLetter): (FREECO)."

New-Item -ItemType Directory -Path "$($data.DriveLetter):\models"   -Force | Out-Null
New-Item -ItemType Directory -Path "$($data.DriveLetter):\freeco"   -Force | Out-Null
Ok "Created \models and \freeco."

Write-Host ""
Note "==================== DONE ===================="
Note "Shared drive : $($data.DriveLetter):  (exFAT, no 4 GB file limit)"
Note "Models go in : $($data.DriveLetter):\models"
Note "Free space   : $linuxGB GB unallocated, for Kubuntu's installer to make ext4."
Write-Host ""
Note "Next steps:"
Note "  1. Copy your GGUF model(s) into $($data.DriveLetter):\models"
Note "  2. Write a Kubuntu 24.04 LTS ISO to a SECOND stick (or use Ventoy),"
Note "     boot it, and install Kubuntu into the $linuxGB GB unallocated space"
Note "     on disk $DiskNumber. Point its bootloader at the FREECO-EFI partition."
Note "  3. Both Windows and Kubuntu then read the SAME model from $($data.DriveLetter):\models"
