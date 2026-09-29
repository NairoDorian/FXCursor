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
#>param([string]$Dir, [string]$Stem, [int]$CropW = 400, [int]$CropH = 240, [int]$Cols = 5, [string]$Out)
Add-Type -AssemblyName System.Drawing
$frames = @(Get-ChildItem -Path $Dir -Filter "${Stem}_*.png" | Sort-Object Name)
if ($frames.Count -eq 0) { Write-Output "no frames for $Stem"; exit 0 }
$rows = [Math]::Ceiling($frames.Count / $Cols)
$sheet = New-Object System.Drawing.Bitmap ($Cols * $CropW), ($rows * $CropH)
$g = [System.Drawing.Graphics]::FromImage($sheet)
$g.Clear([System.Drawing.Color]::FromArgb(20, 20, 24))
$font = New-Object System.Drawing.Font "Consolas", 12
$brush = [System.Drawing.Brushes]::Yellow
for ($i = 0; $i -lt $frames.Count; $i++) {
  $img = [System.Drawing.Image]::FromFile($frames[$i].FullName)
  $sx = [int](($img.Width - $CropW) / 2); $sy = [int](($img.Height - $CropH) / 2)
  $dx = ($i % $Cols) * $CropW; $dy = [Math]::Floor($i / $Cols) * $CropH
  $g.DrawImage($img, (New-Object System.Drawing.Rectangle $dx, $dy, $CropW, $CropH), (New-Object System.Drawing.Rectangle $sx, $sy, $CropW, $CropH), [System.Drawing.GraphicsUnit]::Pixel)
  $g.DrawString(("{0:d2}" -f $i), $font, $brush, ($dx + 4), ($dy + 2))
  $g.DrawRectangle([System.Drawing.Pens]::DimGray, $dx, $dy, ($CropW - 1), ($CropH - 1))
  # cursor position marker (image centre)
  $cx = $dx + [int]($CropW / 2); $cy = $dy + [int]($CropH / 2)
  $g.DrawEllipse([System.Drawing.Pens]::Red, ($cx - 4), ($cy - 4), 8, 8)
  $img.Dispose()
}
$sheet.Save($Out, [System.Drawing.Imaging.ImageFormat]::Png)
$g.Dispose(); $sheet.Dispose()
Write-Output "sheet: $Out ($($frames.Count) frames, $Cols x $rows)"
