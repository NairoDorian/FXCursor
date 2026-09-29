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
#>param([string]$Dir, [string]$Stem, [int]$From = 0, [int]$To = 7, [int]$CropW = 240, [int]$CropH = 140, [int]$Scale = 3, [int]$Cols = 4, [string]$Out)
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
