# Run a General Pre-Discard simulation (10,000 samples)
# This checks Win Probability for Grand + 4 Suits for each hand and takes the MAX.
# Used to answer: "Do I have a playable game?"
# Output: research/data/general_pre_stats.csv

$count = 10000
$samples = 20 # 20 samples per game type (5 types) = 100 simulations per hand. Total 1M simulations.
$output = "research/data/general_pre_stats_final.csv"

Write-Host "Starting General Pre-Discard Analysis with $count hands..."
Write-Host "Output file: $output"

# Ensure the executable exists
$exe = ".\target\release\skat_aug23.exe"
if (-not (Test-Path $exe)) {
    Write-Host "Executable not found at $exe. Please build first with 'cargo build --release'."
    exit 1
}

# Run the command with new 'analyze-general' mode
& $exe analyze-general --count $count --samples $samples --output $output

Write-Host "General Analysis complete."
