# Run from project root (skatsearch)
Write-Host "Starting Skat Suit Analysis Simulation (10,000 Games, 40 Samples)..."
Write-Host "This may take 3-5 hours."

$startTime = Get-Date

cargo run --release -- analyze-suit --count 10000 --samples 40 --output research/data/suit_large_10k.csv

$endTime = Get-Date
$duration = $endTime - $startTime
Write-Host "Simulation Complete."
Write-Host "Duration: $($duration.TotalHours) hours"
Write-Host "Output: research/data/suit_large_10k.csv"
