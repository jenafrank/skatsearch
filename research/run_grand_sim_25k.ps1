# Run a massive Grand Hand simulation (25,000 samples)
# This will generate 'grand_pickup_25k.csv' which provides a solid statistical basis.

$count = 25000
$samples = 20 # PIMC samples per hand (lower samples per hand to allow more hands, PIMC converges fast for Win/Loss)
$output = "research/data/grand_pickup_25k.csv"

Write-Host "Starting Grand Hand Simulation with $count hands..."
Write-Host "Output file: $output"

# Ensure the executable exists (assuming standard build location)
$exe = ".\target\release\skat_aug23.exe"
if (-not (Test-Path $exe)) {
    Write-Host "Executable not found at $exe. Please build first with 'cargo build --release'."
    exit 1
}

# Run the command
# --hand flag is NOT used because we WANT to simulate the Pickup process (Pick up Skat -> Discard -> Play).
# The default behavior of analyze-grand (without --hand) is exactly what we want: Analyze "Grand with Pickup".
& $exe analyze-grand --count $count --samples $samples --output $output

Write-Host "Simulation complete."
