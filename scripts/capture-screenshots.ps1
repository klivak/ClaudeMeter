<#
.SYNOPSIS
    Regenerates the README screenshots from a running ClaudeMeter build.

.DESCRIPTION
    Drives the real app instead of hand-cropping: sets a theme in config.json,
    restarts the app, opens the tray popup by posting the same tray message the
    shell sends, and captures the popup window rect exactly.

    Run it after `cargo build --release`. The caller's config.json is backed up
    and restored verbatim at the end.

.EXAMPLE
    powershell -NoProfile -ExecutionPolicy Bypass -File scripts/capture-screenshots.ps1
#>
[CmdletBinding()]
param(
    [string]$Exe = "$PSScriptRoot\..\target\release\claudemeter.exe",
    [string]$OutDir = "$PSScriptRoot\..\screenshots",
    # Seconds to wait after launch before capturing. The first usage poll has to
    # land, otherwise the popup renders a shorter "no data yet" layout and the
    # footer shows the stale-data warning instead of a timestamp.
    [int]$WarmupSeconds = 30
)

$ErrorActionPreference = 'Stop'

Add-Type @"
using System;
using System.Text;
using System.Runtime.InteropServices;
public class ShotCap {
  [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] public static extern bool PostMessageW(IntPtr h, uint msg, IntPtr wp, IntPtr lp);
  [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
  [DllImport("user32.dll")] public static extern void mouse_event(uint f, uint dx, uint dy, uint d, IntPtr e);
  [DllImport("user32.dll")] public static extern bool EnumWindows(EnumProc cb, IntPtr p);
  [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern int GetClassNameW(IntPtr h, StringBuilder s, int n);
  [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
  public delegate bool EnumProc(IntPtr h, IntPtr p);
  [StructLayout(LayoutKind.Sequential)] public struct RECT { public int Left, Top, Right, Bottom; }

  public static IntPtr ByClass(uint targetPid, string cls) {
    IntPtr found = IntPtr.Zero;
    EnumWindows((h, p) => {
      uint pid; GetWindowThreadProcessId(h, out pid);
      if (pid == targetPid) {
        var sb = new StringBuilder(256); GetClassNameW(h, sb, 256);
        if (sb.ToString() == cls) { found = h; return false; }
      }
      return true;
    }, IntPtr.Zero);
    return found;
  }

  public static void Click(int x, int y) {
    SetCursorPos(x, y);
    System.Threading.Thread.Sleep(150);
    mouse_event(0x0002, 0, 0, 0, IntPtr.Zero);
    System.Threading.Thread.Sleep(70);
    mouse_event(0x0004, 0, 0, 0, IntPtr.Zero);
  }
}
"@

[ShotCap]::SetProcessDPIAware() | Out-Null
Add-Type -AssemblyName System.Drawing

$exePath = (Resolve-Path $Exe).Path
$cfg = Join-Path (Split-Path $exePath) 'config.json'
$outDir = (Resolve-Path $OutDir).Path
if (-not (Test-Path $cfg)) { throw "config.json not found next to the exe — run the app once first." }

# Windows PowerShell's -Encoding utf8 writes a BOM, which serde_json rejects;
# the app then silently falls back to defaults. Write plain UTF-8.
function Write-Json($path, $obj) {
    $text = $obj | ConvertTo-Json -Depth 12
    [System.IO.File]::WriteAllText($path, $text, (New-Object System.Text.UTF8Encoding $false))
}

function Restart-App {
    Stop-Process -Name claudemeter -Force -ErrorAction SilentlyContinue
    Start-Sleep -Seconds 3
    Start-Process $exePath
    Start-Sleep -Seconds $WarmupSeconds
}

function Capture([string]$file, [switch]$Settings) {
    $proc = Get-Process claudemeter -ErrorAction SilentlyContinue
    if (-not $proc) { throw 'claudemeter is not running' }
    $procId = [uint32]$proc.Id

    $main = [ShotCap]::ByClass($procId, 'ClaudeMeterMain')
    if ($main -eq [IntPtr]::Zero) { throw 'main window not found' }

    # The tray click toggles, so only send it when the popup is hidden.
    # WM_TRAY_ICON = WM_USER+1 = 1025, lparam = WM_LBUTTONUP (0x0202).
    $popup = [ShotCap]::ByClass($procId, 'ClaudeMeterPopup')
    if ($popup -eq [IntPtr]::Zero -or -not [ShotCap]::IsWindowVisible($popup)) {
        [ShotCap]::PostMessageW($main, 1025, [IntPtr]1, [IntPtr]0x0202) | Out-Null
        Start-Sleep -Milliseconds 2500
        $popup = [ShotCap]::ByClass($procId, 'ClaudeMeterPopup')
    }
    if ($popup -eq [IntPtr]::Zero -or -not [ShotCap]::IsWindowVisible($popup)) { throw 'popup did not open' }

    $r = New-Object ShotCap+RECT
    [ShotCap]::GetWindowRect($popup, [ref]$r) | Out-Null

    if ($Settings) {
        [ShotCap]::Click(($r.Right - 62), ($r.Top + 25))   # gear icon in the header
        Start-Sleep -Milliseconds 2500
        [ShotCap]::GetWindowRect($popup, [ref]$r) | Out-Null
    }

    $w = $r.Right - $r.Left
    $h = $r.Bottom - $r.Top
    $bmp = New-Object System.Drawing.Bitmap $w, $h
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.CopyFromScreen($r.Left, $r.Top, 0, 0, (New-Object System.Drawing.Size $w, $h))
    $g.Dispose()
    $path = Join-Path $outDir $file
    $bmp.Save($path, [System.Drawing.Imaging.ImageFormat]::Png)
    $bmp.Dispose()
    Write-Output ("saved {0} ({1}x{2})" -f $file, $w, $h)
}

Copy-Item $cfg "$cfg.shotbak" -Force
try {
    $shots = @(
        @{ theme = 'dark';     file = 'dashboard-dark-v5.1.png' },
        @{ theme = 'light';    file = 'dashboard-light-v5.1.png' },
        @{ theme = 'sunset';   file = 'theme-sunset.png' },
        @{ theme = 'midnight'; file = 'theme-midnight.png' }
    )

    foreach ($s in $shots) {
        $j = Get-Content $cfg -Raw | ConvertFrom-Json
        $j.theme = $s.theme
        $j.accessibility_patterns = $false   # shipped default; keeps the bars clean
        Write-Json $cfg $j
        Restart-App
        Capture $s.file
    }

    $j = Get-Content $cfg -Raw | ConvertFrom-Json
    $j.theme = 'light'
    $j.accessibility_patterns = $false
    Write-Json $cfg $j
    Restart-App
    Capture 'settings.png' -Settings
}
finally {
    Copy-Item "$cfg.shotbak" $cfg -Force
    Remove-Item "$cfg.shotbak" -Force -ErrorAction SilentlyContinue
    Stop-Process -Name claudemeter -Force -ErrorAction SilentlyContinue
    Start-Sleep -Seconds 2
    Start-Process $exePath
    Write-Output 'config restored, app restarted'
}
