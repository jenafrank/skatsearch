# Fetch Results from Server
# Downloads the latest 'general_pre_stats_final_50k' CSV file.

$ServerIP = "62.171.133.101"
$User = "root"
$RemotePath = "~/skatsearch/research/data"
$LocalPath = "research/data"

Write-Host "Checking for latest file on server..."
# Check for latest file
$Cmd = "ls -t $RemotePath/general_pre_stats_final_50k_*.csv 2>/dev/null | head -n 1"
$LatestFile = ssh $User@$ServerIP $Cmd

if ([string]::IsNullOrWhiteSpace($LatestFile)) {
    Write-Host "No file found on server." -ForegroundColor Red
    exit
}

$FileName = Split-Path $LatestFile -Leaf
Write-Host "Found: $FileName"

$LocalFile = Join-Path $LocalPath $FileName

if (Test-Path $LocalFile) {
    Write-Host "File already exists locally." -ForegroundColor Yellow
    $resp = Read-Host "Overwrite? (y/n)"
    if ($resp -ne "y") { exit }
}

Write-Host "Downloading..."
scp "$User@${ServerIP}:$LatestFile" "$LocalFile"

if ($?) {
    Write-Host "Download successful: $LocalFile" -ForegroundColor Green
}
else {
    Write-Host "Download failed." -ForegroundColor Red
}

Read-Host "Press Enter to exit"
