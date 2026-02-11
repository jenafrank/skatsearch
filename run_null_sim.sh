#!/bin/bash
# 48h Null Simulation on 16 Cores

# Configuration
# 40 samples per hand, ~46s execution time assumed.
# 48 hours = 172,800 seconds.
# 172,800 / 46 = ~3756 hands.
# Using 3500 hands for safety.
COUNT_PER_CORE=3500
SAMPLES=40
OUTPUT_PREFIX="null_sim_48h"
CORES=16

echo "Starting Skat Null Simulation on $CORES cores..."
echo "Configuration: Count=$COUNT_PER_CORE (per core), Samples=$SAMPLES, Hand=true"
echo "Estimated Total Hands: $(($CORES * $COUNT_PER_CORE))"

# Cleanup previous runs if needed
# rm ${OUTPUT_PREFIX}_*.csv 2>/dev/null

pids=""

for i in $(seq 1 $CORES); do
   OUT_FILE="${OUTPUT_PREFIX}_part${i}.csv"
   echo "Launching worker $i -> $OUT_FILE"
   # Running in background
   ./target/release/skat_aug23 analyze-null --count $COUNT_PER_CORE --samples $SAMPLES --output "$OUT_FILE" --hand &
   # Store PID
   pids="$pids $!"
done

echo "All workers started. PIDs: $pids"
echo "Waiting for completion..."

wait

echo "Simulation finished."
