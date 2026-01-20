$ErrorActionPreference = "Stop"

$timestamp = Get-Date -Format "yyyyMMdd-HHmm"
$output = "research/data/general_pre_stats_final_50k_${timestamp}.csv"

Write-Host "Starting General Pre-Discard Simulation (50,000 Hands)..."
Write-Host "Output file: $output"

# Use streaming output (writes immediately)
cargo run --release -- analyze-general --count 50000 --samples 20 --output $output

Write-Host "Simulation Complete."
