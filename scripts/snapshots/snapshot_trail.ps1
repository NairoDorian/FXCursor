param(
  [string]$OutPath = (Join-Path $PSScriptRoot "..\..\target\snapshots\trail_snapshot.png"),
  [string]$Exe = (Join-Path $PSScriptRoot "..\..\target\debug\fxcursor.exe"),
  [int]$StartX = 900,
  [int]$StartY = 700,
  [string]$Shape = "hairpins",
  [int]$Laps = 3,
  [string]$Size = "1400x800",
  [int]$StepPx = 8,
  [int]$DelayMs = 4
)
Add-Type @"
using System;
using System.Runtime.InteropServices;
public static class Win32Cur {
  [DllImport("user32.dll")] public static extern bool SetCursorPos(int X, int Y);
  [DllImport("user32.dll")] public static extern bool GetCursorPos(out POINT p);
  [DllImport("user32.dll")] public static extern bool SetProcessDpiAwarenessContext(IntPtr ctx);
  [DllImport("user32.dll")] public static extern int GetSystemMetrics(int i);
  [StructLayout(LayoutKind.Sequential)] public struct POINT { public int X; public int Y; }
}
"@
# Per-monitor-v2 DPI awareness so coordinates are physical pixels.
$ctx = [IntPtr]::op_Explicit([int64]-4)
$aware = [Win32Cur]::SetProcessDpiAwarenessContext($ctx)
$sw = [Win32Cur]::GetSystemMetrics(0); $sh = [Win32Cur]::GetSystemMetrics(1)
Write-Output "dpi-aware=$aware screen=${sw}x${sh}"

# NOTE: in PowerShell the comma binds tighter than + / -, so every coordinate expression is
# parenthesised: @(($x + 1), ($y - 1)).
function P([int]$a, [int]$b) { return ,@($a, $b) }

function Move-Along($points) {
  for ($i = 0; $i -lt $points.Count - 1; $i++) {
    $a = $points[$i]; $b = $points[$i + 1]
    $dx = $b[0] - $a[0]; $dy = $b[1] - $a[1]
    $len = [Math]::Sqrt(($dx * $dx) + ($dy * $dy))
    $n = [Math]::Max(1, [int]($len / $StepPx))
    for ($s = 0; $s -le $n; $s++) {
      $t = $s / $n
      [Win32Cur]::SetCursorPos([int]($a[0] + ($dx * $t)), [int]($a[1] + ($dy * $t))) | Out-Null
      Start-Sleep -Milliseconds $DelayMs
    }
  }
}

$x = $StartX; $y = $StartY
$path = New-Object System.Collections.ArrayList
switch ($Shape) {
  "hairpins" {
    [void]$path.Add((P $x $y))
    [void]$path.Add((P ($x + 220) ($y - 160)))
    [void]$path.Add((P ($x + 460) ($y + 20)))
    [void]$path.Add((P ($x + 260) ($y + 160)))
    [void]$path.Add((P ($x + 700) ($y + 180)))
    [void]$path.Add((P ($x + 720) ($y - 120)))
    [void]$path.Add((P ($x + 500) ($y - 140)))
    [void]$path.Add((P ($x + 900) ($y - 60)))
    [void]$path.Add((P $x $y))
  }
  "zigzag" {
    [void]$path.Add((P $x $y))
    for ($k = 1; $k -le 6; $k++) {
      $yy = if ($k % 2 -eq 1) { $y - 140 } else { $y }
      [void]$path.Add((P ($x + ($k * 120)) $yy))
    }
    [void]$path.Add((P $x $y))
  }
  "loop" {
    for ($i = 0; $i -le 96; $i++) {
      $a = ($i / 96.0) * 6.2832 * 2.0
      [void]$path.Add((P ([int]($x + 300 + ([Math]::Cos($a) * 170))) ([int]($y + ([Math]::Sin($a) * 120)))))
    }
  }
}
Write-Output "path points=$($path.Count) first=$($path[0] -join ',') second=$($path[1] -join ',')"

if (Test-Path $OutPath) { Remove-Item $OutPath -Force }
Move-Along $path
$p = New-Object Win32Cur+POINT; [Win32Cur]::GetCursorPos([ref]$p) | Out-Null
Write-Output "cursor after warm-up lap: $($p.X),$($p.Y)"
# The capture process takes ~1 s to start; keep moving so the request lands mid-motion.
Start-Process -FilePath $Exe -ArgumentList @("--capture", "`"$OutPath`"", "--capture-size", $Size) -WindowStyle Hidden | Out-Null
for ($lap = 0; $lap -lt $Laps; $lap++) { Move-Along $path }

$deadline = (Get-Date).AddSeconds(8); $last = -1
while ((Get-Date) -lt $deadline) {
  if (Test-Path $OutPath) {
    $len = (Get-Item $OutPath).Length
    if ($len -gt 0 -and $len -eq $last) { break }
    $last = $len
  }
  Start-Sleep -Milliseconds 150
}
if (Test-Path $OutPath) { Write-Output "snapshot: $OutPath ($((Get-Item $OutPath).Length) bytes)" } else { Write-Output "snapshot NOT written" }
