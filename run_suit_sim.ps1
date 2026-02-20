$ErrorActionPreference = "Stop"

Write-Host "Building Release Binary..."
cargo build --release --bin skat_aug23
if ($LASTEXITCODE -ne 0) {
    Write-Error "Build failed."
    exit 1
}

$bin = "target\release\skat_aug23.exe"
$logFile = "suit_playout_log.txt"
$count = 100
$samples = 20

Write-Host "Starting simulation of $count hands (Samples per move: $samples)..."
Write-Host "Output will be appended to $logFile"

# Clear log file
"" | Out-File -FilePath $logFile -Encoding utf8

$totalLoss = 0
$perfectWins = 0

for ($i = 1; $i -le $count; $i++) {
    Write-Host -NoNewline "Run $i/$count... "
    
    $output = & $bin playout --game-type suit --start-player declarer --samples $samples 2>&1
    
    # Append to log
    "=== Run $i ===" | Out-File -FilePath $logFile -Append -Encoding utf8
    $output | Out-File -FilePath $logFile -Append -Encoding utf8
    
    # Extract loss for quick summary
    $lossLine = $output | Select-String "Total Point Loss: (\d+)"
    if ($lossLine) {
        $loss = $lossLine.Matches.Groups[1].Value
        $totalLoss += [int]$loss
        if ([int]$loss -eq 0) {
            $perfectWins += 1
        }
        Write-Host "Loss: $loss"
    }
    else {
        Write-Host "Done (Loss parsing failed)"
    }
}

Write-Host "`nSimulation Complete."
Write-Host "Total Loss: $totalLoss"
Write-Host "Perfect Games: $perfectWins / $count"
"Simulation Complete. Total Loss: $totalLoss. Perfect Games: $perfectWins / $count" | Out-File -FilePath $logFile -Append -Encoding utf8
