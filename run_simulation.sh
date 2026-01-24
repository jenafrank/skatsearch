#!/bin/bash
set -e

# ========================================================================================
# Skat Best Hand Game Simulation Runner
# ========================================================================================
# This script builds the simulation engine and runs the "Best Hand Game" analysis.
# It then generates visualization plots from the resulting data.
#
# Usage: ./run_simulation.sh [COUNT] [SAMPLES]
#   COUNT:   Number of random hands to simulate (default: 5000000)
#   SAMPLES: Number of opponent distributions per hand (default: 100)
# ========================================================================================

# Default Values
COUNT=${1:-5000000}
SAMPLES=${2:-100}

# Paths
OUTPUT_DIR="research/data"
PLOTS_DIR="research/plots"
OUTPUT_FILE="${OUTPUT_DIR}/hand_best_game_cluster.csv"
LOG_FILE="simulation.log"

# Create directories
mkdir -p "$OUTPUT_DIR"
mkdir -p "$PLOTS_DIR"

echo "=================================================="
echo "      SKAT SIMULATION RUNNER"
echo "=================================================="
echo "Hands to simulate:     $COUNT"
echo "Samples per hand:      $SAMPLES"
echo "Output file:           $OUTPUT_FILE"
echo "Log file:              $LOG_FILE"
echo "=================================================="

# 1. Build Project
echo "[1/3] Building Skat Search Engine (Release Mode)..."
cargo build --release

# 2. Run Simulation
echo "[2/3] Starting Simulation..."
echo "      This may take a while. Progress is logged to $LOG_FILE."

# Using nohup is optional here, but running directly allows user to see progress bar if implemented
# or just wait. Since user mentioned "compute server", usually batch jobs use scripts like this directly.
# We will run synchronously so the script waits for completion.

./target/release/skat_aug23 analyze-general-hand \
    --count "$COUNT" \
    --samples "$SAMPLES" \
    --output "$OUTPUT_FILE" \
    > "$LOG_FILE" 2>&1

# Check exit code
if [ $? -eq 0 ]; then
    echo "      Simulation finished successfully."
else
    echo "      Simulation FAILED. Check $LOG_FILE for details."
    exit 1
fi

# 3. Generate Plots
echo "[3/3] Generating Plots..."
if command -v python3 &> /dev/null; then
    python3 research/scripts/plot_hand.py --input "$OUTPUT_FILE"
    echo "      Plots generated in $PLOTS_DIR"
else
    echo "      WARNING: python3 not found. Skipping plot generation."
fi

echo "=================================================="
echo "      DONE!"
echo "=================================================="
