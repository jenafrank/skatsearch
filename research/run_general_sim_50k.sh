#!/bin/bash
set -e

timestamp=$(date +"%Y%m%d-%H%M")
output="research/data/general_pre_stats_final_50k_${timestamp}.csv"

echo "Starting General Pre-Discard Simulation (50,000 Hands)..."
echo "Output file: $output"

# Ensure output directory exists (optional, but good practice)
mkdir -p research/data

# Use streaming output (writes immediately)
# Note: Ensure you run this from the project root (where Cargo.toml is)
cargo run --release -- analyze-general --count 50000 --samples 20 --output "$output"

echo "Simulation Complete."
