# Check Hand Simulation Status (Server)
$ServerIP = "62.171.133.101"
$User = "root"

# Script to run remotely
$RemoteScript = @'
cd ~/skatsearch 2>/dev/null || echo "[WARN] Directory not found."

echo "--- Process Status ---"
if pgrep -f "analyze-general-hand" > /dev/null; then
    MYPID=$(pgrep -f "analyze-general-hand" | head -n 1)
    echo "[RUNNING] Simulation Active (PID: $MYPID)"
elif pgrep -f "cargo run" > /dev/null; then
     # Fallback if just cargo run is detected
    MYPID=$(pgrep -f "cargo run" | head -n 1)
    echo "[RUNNING] Simulation Active (PID: $MYPID) (Generic Cargo)"
else
    echo "[STOPPED] Simulation Inactive"
fi

echo ""
echo "--- Progress Check ---"
# LLook for the file defined in run_simulation.sh
LATEST_FILE=$(ls -t research/data/hand_best_game_cluster.csv research/data/hand_best_game_*.csv 2>/dev/null | head -n 1)

if [ -z "$LATEST_FILE" ]; then
    echo "[WARN] No CSV file found in research/data/."
else
    echo "File: $LATEST_FILE"
    # Robust line counting using awk
    LINES=$(awk 'END {print NR}' "$LATEST_FILE")
    
    # Ensure LINES is a number (default 0)
    if [ -z "$LINES" ]; then LINES=0; fi
    
    if [ "$LINES" -gt 0 ]; then
        HANDS=$((LINES - 1))
    else
        HANDS=0
    fi
    
    TARGET=50000
    
    echo "Completed Hands: $HANDS / $TARGET"
    
    PERC=$((HANDS * 100 / TARGET))
    echo "Progress: $PERC%"

    # Calculate Time stats if file is modified recently
    END_EPOCH=$(stat -c %Y "$LATEST_FILE" 2>/dev/null)
    if [ -z "$END_EPOCH" ]; then END_EPOCH=$(date +%s); fi
    
    # We don't have start time easily unless we parse it from log or filename (but typical custom filenames don't have timestamp here)
    # We can try to infer from file creation time or "DurationMs" column sum
    
    # Calculate Progress based on Process Elapsed Time (Throughput)
    if [ ! -z "$MYPID" ]; then
        # Get elapsed time in seconds for the process
        ELAPSED_SEC=$(ps -o etimes= -p "$MYPID" 2>/dev/null | tr -d ' ')
        
        if [ ! -z "$ELAPSED_SEC" ] && [ "$ELAPSED_SEC" -gt 0 ] && [ "$HANDS" -gt 0 ]; then
             HANDS_PER_SEC=$(awk -v h="$HANDS" -v e="$ELAPSED_SEC" 'BEGIN {print h / e}')
             
             # Avoid division by zero if rate is super low (start)
             if (( $(echo "$HANDS_PER_SEC > 0" | bc -l) )); then
                 REM_HANDS=$((TARGET - HANDS))
                 REM_SEC=$(awk -v r="$REM_HANDS" -v s="$HANDS_PER_SEC" 'BEGIN {print r / s}')
                 
                 REM_HOURS=$(awk -v s="$REM_SEC" 'BEGIN {printf "%.2f", s / 3600}')
                 
                 echo "Runtime:    $ELAPSED_SEC s"
                 echo "Throughput: $HANDS_PER_SEC hands/sec"
                 echo "Est. Left:  $REM_HOURS hours"
             fi
        fi
    else
        # Fallback if process not running (using Avg Netto is misleading for multicore, so show warning)
        echo "Process not active. Cannot calculate multicore throughput accurately without runtime."
    fi

    echo "Last Result:"
    tail -n 1 "$LATEST_FILE"
fi

echo ""
echo "--- Log Tail (last 5 lines) ---"
if [ -f "simulation.log" ]; then
    tail -n 5 simulation.log
else
    echo "simulation.log not found."
fi

echo "[DONE]"
'@

# Ensure Linux Line Endings (LF)
$ScriptToRun = $RemoteScript -replace "`r", ""

Write-Host "Connecting to $User@$ServerIP..."

# Use bash -s to explicitly read commands from stdin
$ScriptToRun | ssh $User@$ServerIP "bash -s"

Write-Host ""
Read-Host -Prompt "Press Enter to exit"
