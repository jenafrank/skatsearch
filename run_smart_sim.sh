#!/usr/bin/env bash
# ─────────────────────────────────────────────────────────────────────────────
# run_smart_sim.sh
# ─────────────────────────────────────────────────────────────────────────────
# Runs the smart playout simulation on a Linux compute server.
#
# Usage:  ./run_smart_sim.sh [N_GAMES] [SAMPLES] [PARALLEL]
#
#   N_GAMES   – qualifying games to collect per worker  (default: 1000)
#   SAMPLES   – PIMC samples per move                   (default: 20)
#   PARALLEL  – number of parallel workers              (default: nproc)
#
# Each worker writes to its own log file:
#   smart_log_worker_1.txt … smart_log_worker_N.txt
#
# After all workers finish, results are merged into smart_log_merged.txt.
#
# Prerequisites
# ─────────────
#   • Rust binary compiled in release mode:
#       cargo build --bin skat_aug23 --release
#   • Python 3 installed:
#       python3 run_smart_playout_loop.py --help
# ─────────────────────────────────────────────────────────────────────────────

set -euo pipefail

# ── Configuration ─────────────────────────────────────────────────────────────
N_GAMES=${1:-1000}
SAMPLES=${2:-20}
N_WORKERS=${3:-$(nproc)}
REPO_DIR="$(cd "$(dirname "$0")" && pwd)"
BIN="${REPO_DIR}/target/release/skat_aug23"
SCRIPT="${REPO_DIR}/run_smart_playout_loop.py"
LOG_DIR="${REPO_DIR}/smart_logs"
MERGED_LOG="${REPO_DIR}/smart_log_merged.txt"

# ── Sanity checks ─────────────────────────────────────────────────────────────
if [[ ! -f "${BIN}" ]]; then
  echo "ERROR: Binary not found at ${BIN}"
  echo "  Build first:  cargo build --bin skat_aug23 --release"
  exit 1
fi

if [[ ! -f "${SCRIPT}" ]]; then
  echo "ERROR: Python script not found at ${SCRIPT}"
  exit 1
fi

if ! command -v python3 &>/dev/null; then
  echo "ERROR: python3 not found in PATH"
  exit 1
fi

mkdir -p "${LOG_DIR}"

echo "════════════════════════════════════════════════════════════"
echo " Smart Skat Playout Simulation"
echo "════════════════════════════════════════════════════════════"
echo "  Directory : ${REPO_DIR}"
echo "  Binary    : ${BIN}"
echo "  Workers   : ${N_WORKERS}"
echo "  Games/wkr : ${N_GAMES}"
echo "  Samples   : ${SAMPLES} per move"
echo "  Total tgt : $((N_GAMES * N_WORKERS)) qualifying games"
echo "  Logs      : ${LOG_DIR}/smart_log_worker_*.txt"
echo "════════════════════════════════════════════════════════════"
echo

# ── Launch workers ─────────────────────────────────────────────────────────────
PIDS=()
for i in $(seq 1 "${N_WORKERS}"); do
  LOG_FILE="${LOG_DIR}/smart_log_worker_${i}.txt"
  echo "▶  Starting worker ${i}/${N_WORKERS}  →  ${LOG_FILE}"
  python3 "${SCRIPT}" \
      --games   "${N_GAMES}" \
      --samples "${SAMPLES}" \
      --out     "${LOG_FILE}" \
      > "${LOG_DIR}/worker_${i}_stdout.txt" 2>&1 &
  PIDS+=($!)
done

echo
echo "All ${N_WORKERS} workers started.  Waiting for completion..."
echo "(monitor with:  tail -f ${LOG_DIR}/worker_1_stdout.txt)"
echo

# ── Wait for all workers ───────────────────────────────────────────────────────
FAILED=0
for i in "${!PIDS[@]}"; do
  PID="${PIDS[$i]}"
  WORKER=$((i + 1))
  if wait "${PID}"; then
    echo "✔  Worker ${WORKER} finished (PID ${PID})"
  else
    echo "✘  Worker ${WORKER} FAILED   (PID ${PID})"
    FAILED=$((FAILED + 1))
  fi
done

# ── Merge logs ────────────────────────────────────────────────────────────────
echo
echo "Merging logs → ${MERGED_LOG}"
{
  echo "Smart Playout – Merged Log"
  echo "Workers: ${N_WORKERS}  Games/worker: ${N_GAMES}  Samples: ${SAMPLES}"
  echo "Generated: $(date -u '+%Y-%m-%dT%H:%M:%SZ')"
  echo "════════════════════════════════════════════════════════════"
  echo
  for i in $(seq 1 "${N_WORKERS}"); do
    LOG_FILE="${LOG_DIR}/smart_log_worker_${i}.txt"
    if [[ -f "${LOG_FILE}" ]]; then
      echo "──── Worker ${i} ────"
      cat "${LOG_FILE}"
      echo
    fi
  done
} > "${MERGED_LOG}"

echo "Merge complete."
echo

# ── Final status ──────────────────────────────────────────────────────────────
if [[ "${FAILED}" -gt 0 ]]; then
  echo "WARNING: ${FAILED} worker(s) encountered errors.  Check logs in ${LOG_DIR}."
  exit 1
else
  echo "SUCCESS: All workers completed."
  echo "  Merged log : ${MERGED_LOG}"
  echo "  Total games: $((N_GAMES * N_WORKERS))"
fi
