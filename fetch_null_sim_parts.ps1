# Fetch Null Simulation Parts

$ServerIP = "62.171.133.101"
$User = "root"
$RemotePath = "~/skatsearch" 
$LocalPath = "research/data"

# Ensure local directory exists
if (-not (Test-Path $LocalPath)) {
    New-Item -ItemType Directory -Force -Path $LocalPath
}

Write-Host "Starting download of Null Simulation parts from $ServerIP..."

for ($i = 1; $i -le 16; $i++) {
    $PartFile = "null_sim_48h_part${i}.csv"
    $RemoteFile = "$RemotePath/$PartFile"
    $LocalFile = Join-Path $LocalPath $PartFile

    if (Test-Path $LocalFile) {
        Write-Host "Skipping $PartFile (already exists)" -ForegroundColor Yellow
        continue
    }

    Write-Host "Downloading $PartFile..."
    scp "${User}@${ServerIP}:${RemoteFile}" "$LocalFile"

    if ($?) {
        Write-Host "Successfully downloaded $PartFile" -ForegroundColor Green
    }
    else {
        Write-Host "Failed to download $PartFile" -ForegroundColor Red
    }
}

Write-Host "All downloads processed."
