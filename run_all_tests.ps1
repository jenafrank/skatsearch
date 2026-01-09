$ErrorActionPreference = "Stop"
$contextsDir = "tests/contexts"
$files = Get-ChildItem -Path $contextsDir -Filter "*.json"

foreach ($file in $files) {
    Write-Host "----------------------------------------------------------------"
    Write-Host "Testing: $($file.Name)"
    
    $content = Get-Content -Raw $file.FullName | ConvertFrom-Json
    
    if (-not $content.usage) {
        Write-Host "SKIP: No 'usage' field found." -ForegroundColor Yellow
        continue
    }

    $command = $content.usage
    $expectFailure = if ($content.expect_failure) { $true } else { $false }

    Write-Host "Command: $command"
    
    try {
        # Split command into exe and args for Start-Process or just Invoke-Expression
        $process = Start-Process -FilePath "powershell" -ArgumentList "-Command $command" -NoNewWindow -PassThru -Wait
        $exitCode = $process.ExitCode
    } catch {
        $exitCode = -1
        Write-Host "Error running command: $_" -ForegroundColor Red
    }

    if ($expectFailure) {
        if ($exitCode -ne 0) {
            Write-Host "PASS (Expected Failure)" -ForegroundColor Green
        } else {
            Write-Host "FAIL (Expected Failure but succeeded)" -ForegroundColor Red
        }
    } else {
        if ($exitCode -eq 0) {
            Write-Host "PASS" -ForegroundColor Green
        } else {
            Write-Host "FAIL (Exit Code: $exitCode)" -ForegroundColor Red
        }
    }
}
Write-Host "----------------------------------------------------------------"
Write-Host "Done."
