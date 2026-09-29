<#
  ================================================================================
  HOW TO USE THIS SCRIPT
  ================================================================================
  The app must ALREADY be running, started with `bun run tauri dev` (which runs
  Vite, so the front end loads and the overlay is transparent).

  This script starts a NEW `fxcursor.exe --capture ...` process and relies on
  the single-instance plugin to forward the request to that running instance.
  If NO instance is running, the capture process becomes a PRIMARY instance: it
  ignores the capture args and starts the app with no Vite server on port 1420.
  A debug build has no front end bundled in, so the webview renders an error
  page and the overlay is drawn OPAQUE instead of transparent -- a stuck
  non-transparent window across the whole desktop. That has twice happened and
  required REBOOTING WINDOWS. A broken overlay is a broken desktop.

  Start the app first, confirm it is up, then run this. Never run it against a
  stopped app. Full policy: docs/RUNNING_AND_DEBUGGING.md
#>param(
  [string]$OutPath = (Join-Path $PSScriptRoot "..\..\target\snapshots\trail_motion.png"),
  [string]$Exe = (Join-Path $PSScriptRoot "..\..\target\debug\fxcursor.exe"),
  [int]$StartX = 600,
  [int]$StartY = 700,
  [int]$StopAtMs = 700,          # ms after the capture request when the motion changes
  [double]$Speed = 1.0,          # px per ms (1.0 = 1000 px/s)
  [string]$Motion = "stop",      # stop | reverse | turn | continue
  [string]$Size = "1000x600",
  [int]$WarmupMs = 700,
  [int]$Burst = 30,              # frames in the burst (files <stem>_NN.png)
  [int]$IntervalMs = 50
)
Add-Type @"
using System;
using System.Runtime.InteropServices;
public static class Win32Cur {
  [DllImport("user32.dll")] public static extern bool SetCursorPos(int X, int Y);
  [DllImport("user32.dll")] public static extern bool SetProcessDpiAwarenessContext(IntPtr ctx);
}
"@
[void][Win32Cur]::SetProcessDpiAwarenessContext([IntPtr]::op_Explicit([int64]-4))
$stem = [System.IO.Path]::GetFileNameWithoutExtension($OutPath)
$dir = [System.IO.Path]::GetDirectoryName($OutPath)
Get-ChildItem -Path $dir -Filter "$stem*.png" -ErrorAction SilentlyContinue | Remove-Item -Force
$lastFrame = Join-Path $dir ("{0}_{1:d2}.png" -f $stem, ($Burst - 1))

# Position as a function of time (ms) since motion start; the motion event happens at $eventMs.
function Pos([double]$t, [double]$eventMs) {
  if ($t -lt $eventMs) { return @(($StartX + $Speed * $t), $StartY) }
  $dt = $t - $eventMs
  $ex = $StartX + $Speed * $eventMs
  switch ($Motion) {
    "stop"     { return @($ex, $StartY) }
    "reverse"  { return @(($ex - $Speed * $dt), $StartY) }
    "turn"     { return @($ex, ($StartY - $Speed * $dt)) }
    default    { return @(($ex + $Speed * $dt), $StartY) }
  }
}

$sw = [System.Diagnostics.Stopwatch]::StartNew()
$eventMs = $WarmupMs + $StopAtMs
$launched = $false
$last = -1
$limit = $eventMs + ($Burst * $IntervalMs) + 3000
while ($sw.ElapsedMilliseconds -lt $limit) {
  $t = $sw.ElapsedMilliseconds
  if (-not $launched -and $t -ge $WarmupMs) {
    Start-Process -FilePath $Exe -ArgumentList @("--capture", "`"$OutPath`"", "--capture-size", $Size, "--capture-burst", $Burst, "--capture-interval", $IntervalMs) -WindowStyle Hidden | Out-Null
    $launched = $true
  }
  $p = Pos $t $eventMs
  [void][Win32Cur]::SetCursorPos([int]$p[0], [int]$p[1])
  if (Test-Path $lastFrame) {
    $len = (Get-Item $lastFrame).Length
    if ($len -gt 0 -and $len -eq $last) { break }
    $last = $len
  }
  Start-Sleep -Milliseconds 3
}
$frames = @(Get-ChildItem -Path $dir -Filter "${stem}_*.png" | Sort-Object Name)
if ($frames.Count -gt 0) {
  $origin = (Get-Date).AddMilliseconds(-$sw.ElapsedMilliseconds)
  $first = [int](($frames[0].LastWriteTime - $origin).TotalMilliseconds) - $eventMs
  $lastT = [int](($frames[-1].LastWriteTime - $origin).TotalMilliseconds) - $eventMs
  Write-Output "$($frames.Count) frames '$Motion': first frame ~${first} ms, last ~${lastT} ms relative to the event (interval $IntervalMs ms)"
} else { Write-Output "no frames written" }
