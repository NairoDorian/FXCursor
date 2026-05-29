# generate_repomix.ps1
# Automates generating repo-summary.md containing project file metadata and running Repomix.

$ESC    = [char]27
$RESET  = "$ESC[0m"
$CYAN   = "$ESC[96m"
$GREEN  = "$ESC[92m"
$YELLOW = "$ESC[93m"
$BOLD   = "$ESC[1m"

Write-Host "${BOLD}${CYAN}=== Generating Repomix Pack with Architecture & Metadata Summary ===${RESET}"

$repoRoot = Resolve-Path "$PSScriptRoot/.."
Push-Location $repoRoot

try {
    # 1. Gather all files in the repository (excluding git-ignored files)
    Write-Host "Scanning workspace files..." -ForegroundColor Gray
    
    # Run git commands to get tracked + non-ignored untracked files
    $trackedFiles = git ls-files
    $untrackedFiles = git ls-files --others --exclude-standard
    $allFiles = ($trackedFiles + $untrackedFiles) | Select-Object -Unique | Sort-Object

    # Ignore list from repomix.config.json
    $ignorePatterns = @()
    if (Test-Path "repomix.config.json") {
        try {
            $config = Get-Content "repomix.config.json" -Raw | ConvertFrom-Json
            if ($config.ignore -and $config.ignore.customPatterns) {
                $ignorePatterns = $config.ignore.customPatterns
            }
        } catch {
            Write-Host "Failed to parse repomix.config.json: $_" -ForegroundColor Yellow
        }
    }

    # Add default code-based ignores to ensure repomix-output and temp files are never scanned
    $ignorePatterns += @("repo-summary.md", "repomix-output.md", "repomix-output-compressed.md", "repomix-output.json")
    $ignorePatterns = $ignorePatterns | Select-Object -Unique

    $filesToScan = @()
    foreach ($file in $allFiles) {
        # Check if file exists in filesystem
        if (-not (Test-Path $file)) {
            continue
        }
        
        $normalizedPath = $file.Replace("\", "/")
        
        # Check against ignores
        $ignored = $false
        foreach ($pattern in $ignorePatterns) {
            # Convert glob patterns with double asterisks to simple single asterisk for PowerShell -like matching
            $psPattern = $pattern.Replace("**", "*")
            
            # If pattern ends with /, also match anything starting with that prefix
            if ($psPattern.EndsWith("/")) {
                $prefix = $psPattern
                if ($normalizedPath.StartsWith($prefix) -or $normalizedPath -like $psPattern -or $normalizedPath -like "*$psPattern*") {
                    $ignored = $true
                    break
                }
            } else {
                if ($normalizedPath -like $psPattern -or $normalizedPath -like "*$psPattern*") {
                    $ignored = $true
                    break
                }
            }
        }
        if (-not $ignored) {
            $filesToScan += $file
        }
    }

    Write-Host "Found $($filesToScan.Count) active files to index." -ForegroundColor Gray

    # Descriptions mapping
    $descriptions = @{
        "project_cursor/Cargo.toml" = "Rust package manifest (dependencies & build configuration)"
        "project_cursor/run.bat" = "CMD shortcut to build and run the application"
        "project_cursor/src/main.rs" = "Application entry point; event loop, dual-window manager, tray setup, and frames coordinator"
        "project_cursor/src/config.rs" = "Configuration management (RON format parsing/saving, shared configuration state)"
        "project_cursor/src/tracker.rs" = "Global mouse coordinate and click tracking using polling (`device_query`)"
        "project_cursor/src/tray.rs" = "System tray initialization, menu actions, and tray icon rendering"
        "project_cursor/src/gui/mod.rs" = "Egui integration state wrapper and window event receiver"
        "project_cursor/src/gui/panel.rs" = "Settings GUI rendering (theme styling, control sliders, effect pickers, defaults)"
        "project_cursor/src/overlay/mod.rs" = "Fullscreen overlay manager; Win32 subclassing for mouse click-through and taskbar hiding"
        "project_cursor/src/overlay/renderer.rs" = "wgpu graphics renderer; buffers, WGSL pipeline, particle simulation, and render pass"
        "project_cursor/src/overlay/shader.wgsl" = "WebGPU shader (WGSL); particle trails, ripples, and glow rendering on the GPU"
        ".gitignore" = "Git patterns for files to exclude from source control"
        "README.md" = "Core project documentation, build guides, and repository goals"
        "memory.md" = "Architectural decision log and research log for structural tracking"
        "repomix-instruction.md" = "AI instruction sheet for context and style guides"
        "repomix.config.json" = "Repomix configuration file (styles, includes, and ignores)"
        "dev_scripts/build_instructions.md" = "Documentation for the build scripts and cross-compilation requirements"
        "dev_scripts/build.ps1" = "PowerShell script for native and cross-platform release/debug builds"
        "dev_scripts/build.bat" = "CMD batch script for native and cross-platform release/debug builds"
        "dev_scripts/cargo_build.ps1" = "PowerShell pre-build verification and compilation script"
        "dev_scripts/cargo_build.bat" = "CMD pre-build verification and compilation script"
        "dev_scripts/cargo_check.ps1" = "PowerShell script to run check, clippy, and rustdoc tests"
        "dev_scripts/cargo_check.bat" = "CMD script to run check, clippy, and rustdoc tests"
        "dev_scripts/cargo_run.ps1" = "PowerShell script to compile and run application with custom log levels"
        "dev_scripts/cargo_run.bat" = "CMD script to compile and run application with custom log levels"
        "dev_scripts/update_dependencies.ps1" = "PowerShell script to dynamically update dependencies to latest compatible versions"
        "dev_scripts/update_dependencies.bat" = "CMD script to dynamically update dependencies to latest compatible versions"
        "dev_scripts/generate_repomix.ps1" = "PowerShell script to auto-generate repository summaries and pack with Repomix"
        "dev_scripts/generate_repomix.bat" = "CMD batch wrapper script to run the Repomix generation workflow"
    }

    # Start compiling repo-summary.md content
    $summaryContent = @(
        "# Repository Summary & File Metadata",
        "",
        "This file contains high-level metadata, line counts, sizes, and descriptions for all files in this project. It provides architectural context without bloating LLM token usage with raw implementation details.",
        "",
        "## Overall Project Statistics",
        ""
    )

    $totalLines = 0
    $totalSize = 0
    $rows = @()

    foreach ($file in $filesToScan) {
        $lines = 0
        $item = Get-Item $file
        
        if (Test-Path $file -PathType Leaf) {
            # Get line count (safe for empty/binary files)
            try {
                $lines = [System.IO.File]::ReadAllLines($item.FullName).Length
            } catch {
                try {
                    $lines = (Get-Content $item.FullName -ErrorAction SilentlyContinue | Measure-Object).Count
                } catch {
                    $lines = 0
                }
            }
        }
        
        $sizeBytes = $item.Length
        $sizeKB = [Math]::Round($sizeBytes / 1KB, 2)
        $lastModified = $item.LastWriteTime.ToString("yyyy-MM-dd HH:mm:ss")
        
        $totalLines += $lines
        $totalSize += $sizeBytes
        
        # Determine description
        $normalizedPath = $file.Replace("\", "/")
        $desc = $descriptions[$normalizedPath]
        if (-not $desc) {
            # Try parsing first comment line in file
            if (Test-Path $file -PathType Leaf) {
                $firstLines = Get-Content $file -TotalCount 5 -ErrorAction SilentlyContinue
                foreach ($line in $firstLines) {
                    $trimmed = $line.Trim()
                    if ($trimmed.StartsWith("//") -or $trimmed.StartsWith("#")) {
                        $desc = $trimmed.TrimStart("/").TrimStart("#").Trim()
                        if ($desc.Length -gt 5) { break }
                    }
                }
            }
            if (-not $desc) {
                # Fallback based on file extension
                $ext = [System.IO.Path]::GetExtension($file)
                if ($ext -eq ".rs") { $desc = "Rust source code file" }
                elseif ($ext -eq ".wgsl") { $desc = "WebGPU Shading Language file" }
                elseif ($ext -eq ".bat") { $desc = "Windows Batch script" }
                elseif ($ext -eq ".ps1") { $desc = "PowerShell script" }
                elseif ($ext -eq ".md") { $desc = "Markdown documentation" }
                else { $desc = "Project file" }
            }
        }
        
        $rows += [PSCustomObject]@{
            Path = "$normalizedPath"
            Lines = $lines
            Size = "$sizeKB KB"
            LastEdited = $lastModified
            Description = $desc
        }
    }

    $totalSizeKB = [Math]::Round($totalSize / 1KB, 2)
    $summaryContent += "- **Total Scanned Files**: $($filesToScan.Count)"
    $summaryContent += "- **Total Lines of Code/Config**: $totalLines lines"
    $summaryContent += "- **Total Repository Size**: $totalSizeKB KB"
    $summaryContent += ""
    $summaryContent += "## Repository File Metadata Table"
    $summaryContent += ""
    $summaryContent += "| File Path | Lines | Size | Last Edited | Description / Purpose |"
    $summaryContent += "| :--- | :---: | :---: | :--- | :--- |"

    foreach ($row in $rows) {
        $summaryContent += ('| `{0}` | {1} | {2} | {3} | {4} |' -f $row.Path, $row.Lines, $row.Size, $row.LastEdited, $row.Description)
    }

    $summaryContent += ""
    
    # Save the summary file
    $summaryFilePath = Join-Path $repoRoot "repo-summary.md"
    $summaryContent | Out-File -FilePath $summaryFilePath -Encoding utf8 -Force
    Write-Host "Created metadata summary file at repo-summary.md" -ForegroundColor Green

    # Run repomix
    Write-Host "Running Repomix to compile package..." -ForegroundColor Gray
    npx repomix
    
    Write-Host "=== Pack Completed Successfully! ===" -ForegroundColor Green
    Write-Host "Output: repomix-output.md" -ForegroundColor Green
}
catch {
    Write-Error $_
    exit 1
}
finally {
    Pop-Location
}
