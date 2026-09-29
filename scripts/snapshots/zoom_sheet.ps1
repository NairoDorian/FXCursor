<#
  ================================================================================
  DO NOT RUN THIS SCRIPT FROM AN AUTOMATED AGENT.
  ================================================================================
  This script starts a NEW `fxcursor.exe --capture ...` process and relies on the
  single-instance plugin to forward the request to an already-running instance.

  If FXCursor is NOT already running, that process becomes a PRIMARY instance: it
  ignores the capture arguments and starts the app normally. And a debug build has
  NO FRONT END BUNDLED IN -- it loads from "devUrl": "http://localhost:1420", which
  only `bun run tauri dev` starts. So the webview renders an error page and the
  overlay is drawn OPAQUE instead of transparent: a stuck non-transparent window
  across the whole desktop. That has twice happened and the user had to REBOOT
  WINDOWS to recover. A broken overlay is a broken desktop.

  Running this script IS launching the app. Only the user runs it, deliberately,
  with the app already open.

  Full rule: docs/DO_NOT_LAUNCH_THE_APP.md
#>
param([string]$Dir, [string]$Stem, [int]$From = 0, [int]$To = 7, [int]$CropW = 240, [int]$CropH = 140, [int]$Scale = 3, [int]$Cols = 4, [string]$Out)
Add-Type -AssemblyName System.Drawing
$frames = @(Get-ChildItem -Path $Dir -Filter "${Stem}_*.png" | Sort-Object Name | Select-Object -Skip $From -First ($To - $From + 1))
$rows = [Math]::Ceiling($frames.Count / $Cols)
$cw = $CropW * $Scale; $ch = $CropH * $Scale
$sheet = New-Object System.Drawing.Bitmap ($Cols * $cw), ($rows * $ch)
$g = [System.Drawing.Graphics]::FromImage($sheet)
$g.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::NearestNeighbor
$g.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::Half
$g.Clear([System.Drawing.Color]::FromArgb(20, 20, 24))
$font = New-Object System.Drawing.Font "Consolas", 14
for ($i = 0; $i -lt $frames.Count; $i++) {
  $img = [System.Drawing.Image]::FromFile($frames[$i].FullName)
  $sx = [int](($img.Width - $CropW) / 2); $sy = [int](($img.Height - $CropH) / 2)
  $dx = ($i % $Cols) * $cw; $dy = [Math]::Floor($i / $Cols) * $ch
  $g.DrawImage($img, (New-Object System.Drawing.Rectangle $dx, $dy, $cw, $ch), (New-Object System.Drawing.Rectangle $sx, $sy, $CropW, $CropH), [System.Drawing.GraphicsUnit]::Pixel)
  $g.DrawString($frames[$i].BaseName, $font, [System.Drawing.Brushes]::Yellow, ($dx + 4), ($dy + 2))
  $g.DrawRectangle([System.Drawing.Pens]::DimGray, $dx, $dy, ($cw - 1), ($ch - 1))
  $cx = $dx + [int]($cw / 2); $cy = $dy + [int]($ch / 2)
  $g.DrawEllipse([System.Drawing.Pens]::Red, ($cx - 5), ($cy - 5), 10, 10)
  $img.Dispose()
}
$sheet.Save($Out, [System.Drawing.Imaging.ImageFormat]::Png)
$g.Dispose(); $sheet.Dispose()
Write-Output "zoom sheet: $Out ($($frames.Count) frames)"
