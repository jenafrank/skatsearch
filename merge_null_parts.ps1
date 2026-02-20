
$DataDir = "research/data"
$OutputFile = Join-Path $DataDir "null_sim_48h_combined.csv"

Write-Host "Merging Null Simulation Parts into $OutputFile..."

# Clear output file if exists
if (Test-Path $OutputFile) {
    Remove-Item $OutputFile
}

# Process Part 1 separately to include header
$Part1 = Join-Path $DataDir "null_sim_48h_part1.csv"
if (Test-Path $Part1) {
    Write-Host "Processing Part 1 (with header)..."
    Get-Content $Part1 | Add-Content $OutputFile
}
else {
    Write-Error "Part 1 not found!"
    exit
}

# Process remaining parts (skip header)
for ($i = 2; $i -le 16; $i++) {
    $PartFile = Join-Path $DataDir "null_sim_48h_part${i}.csv"
    if (Test-Path $PartFile) {
        Write-Host "Processing Part $i..."
        Get-Content $PartFile | Select-Object -Skip 1 | Add-Content $OutputFile
    }
    else {
        Write-Warning "Part $i not found, skipping."
    }
}

$LineCount = (Get-Content $OutputFile).Count
Write-Host "Merge complete. Total lines: $LineCount"
