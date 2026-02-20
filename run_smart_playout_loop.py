#!/usr/bin/env python3
"""
run_smart_playout_loop.py
─────────────────────────
Runs `skat_aug23 smart-points-playout` in a loop and collects N *qualifying*
games.  A game qualifies when the Rust binary exits with code 0 (deal passed
the best-game / score-threshold filter).  Exit code 2 means the deal was
rejected; those iterations are silently skipped.

Usage
-----
    python run_smart_playout_loop.py [--games 1000] [--samples 20]
                                     [--out points_smart_log.txt]

Output
------
    points_smart_log.txt  – full stdout of every qualifying game,
                            separated by a header line.
    Progress is printed to stdout.
"""

import argparse
import re
import subprocess
import sys
import time
from datetime import datetime
from pathlib import Path

# ── Defaults ──────────────────────────────────────────────────────────────────
DEFAULT_N_GAMES = 1000
DEFAULT_SAMPLES = 20
DEFAULT_LOG     = Path("points_smart_log.txt")
EXE             = Path("./target/release/skat_aug23")

# ── Argument parsing ──────────────────────────────────────────────────────────
parser = argparse.ArgumentParser(description="Smart playout loop")
parser.add_argument("--games",   type=int,  default=DEFAULT_N_GAMES,
                    help="Number of qualifying games to collect (default: 1000)")
parser.add_argument("--samples", type=int,  default=DEFAULT_SAMPLES,
                    help="PIMC samples per move (default: 20)")
parser.add_argument("--out",     type=Path, default=DEFAULT_LOG,
                    help="Output log file")
args = parser.parse_args()

N_GAMES  = args.games
SAMPLES  = args.samples
LOG_FILE = args.out

# ── Helpers ───────────────────────────────────────────────────────────────────

def parse_result(output: str) -> dict:
    info = {}
    for key in ("Declarer", "Left", "Right", "Skat"):
        m = re.search(rf"^{key}\s*:\s*(.+)$", output, re.MULTILINE)
        info[key.lower()] = m.group(1).strip() if m else "?"

    m = re.search(r"Perfect Play Finished\. Result:\s*(\d+)", output)
    info["perfect_result"] = int(m.group(1)) if m else None

    # Format: "Total Point Loss: N (D:N O:N) | Final score: N pts"
    m = re.search(
        r"Total Point Loss:\s*(\d+)\s*\(D:(\d+)\s*O:(\d+)\)",
        output)
    if m:
        info["total_loss"]    = int(m.group(1))
        info["declarer_loss"] = int(m.group(2))
        info["opponent_loss"] = int(m.group(3))
    else:
        info["total_loss"] = info["declarer_loss"] = info["opponent_loss"] = None

    m = re.search(r"Final score:\s*(\d+)\s*pts", output)
    info["final_score"] = int(m.group(1)) if m else None

    # Deal info injected by SmartPointsPlayout handler
    m = re.search(r"SMART_DEAL_OK:\s*game=(\S+)\s+discard=(.+)", output)
    if m:
        info["game_label"] = m.group(1)
        info["discard"]    = m.group(2).strip()
    else:
        info["game_label"] = "?"
        info["discard"]    = "?"

    # Count PIMC card lines and lossless lines
    info["all_moves"]     = len(re.findall(r"PIMC:", output))
    info["perfect_moves"] = len(re.findall(r"loss=0\(", output))
    return info


# ── Main loop ─────────────────────────────────────────────────────────────────

def main():
    exe = EXE
    if sys.platform.startswith("win"):
        exe = Path("target/release/skat_aug23.exe")

    if not exe.exists():
        sys.exit(f"ERROR: binary not found at {exe}")

    cmd = [str(exe), "smart-points-playout", "--samples", str(SAMPLES)]

    with open(LOG_FILE, "w", encoding="utf-8") as log:
        log.write(f"Smart Playout Log – {datetime.now().isoformat()}\n")
        log.write(f"Target games: {N_GAMES}  PIMC samples/move: {SAMPLES}\n")
        log.write("=" * 80 + "\n\n")

    qualified   = 0
    attempts    = 0
    total_loss  = 0
    optimal_pcts = []
    start_time  = time.time()

    print(f"Running until {N_GAMES} qualifying games  →  {LOG_FILE}")

    while qualified < N_GAMES:
        attempts += 1
        result = subprocess.run(
            cmd,
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
        )

        if result.returncode == 2:
            # Deal rejected by Rust (score < 50 or no Grand/Suit)
            continue

        if result.returncode != 0:
            # Unexpected error – log and continue
            with open(LOG_FILE, "a", encoding="utf-8") as log:
                log.write(f"[Attempt {attempts}] ERROR exit={result.returncode}\n")
                log.write(result.stdout[:500])
                log.write("\n")
            continue

        qualified += 1
        output = result.stdout
        info   = parse_result(output)

        # Accumulate stats
        if info["total_loss"] is not None:
            total_loss += info["total_loss"]
        am = info["all_moves"]
        pm = info["perfect_moves"]
        pct = 100 * pm / am if am else 0.0
        optimal_pcts.append(pct)

        loss_str = f"{info['total_loss']:3}" if info["total_loss"] is not None else "  ?"
        perfect_str = f"{info['perfect_result']:3}" if info["perfect_result"] is not None else "  ?"
        score_str   = f"{info['final_score']:3}" if info["final_score"] is not None else "  ?"
        elapsed = time.time() - start_time
        rate = qualified / elapsed if elapsed > 0 else 0
        eta  = (N_GAMES - qualified) / rate if rate > 0 else 0

        print(
            f"  Game {qualified:4}/{N_GAMES}  "
            f"[{info['game_label']:<9}] "
            f"perf={perfect_str}  score={score_str}  loss={loss_str}  "
            f"opt={pct:5.1f}%  "
            f"discard={info['discard']}  "
            f"(attempt {attempts}, rate={rate:.1f}/s, ETA={eta:.0f}s)"
        )

        with open(LOG_FILE, "a", encoding="utf-8") as log:
            log.write(f"{'='*80}\n")
            log.write(f"Game {qualified}/{N_GAMES}  [attempt {attempts}]  "
                      f"game={info['game_label']}  discard={info['discard']}\n")
            log.write(output)
            log.write("\n")

    # ── Final summary ──────────────────────────────────────────────────────────
    elapsed   = time.time() - start_time
    avg_loss  = total_loss / qualified if qualified else 0
    avg_opt   = sum(optimal_pcts) / len(optimal_pcts) if optimal_pcts else 0
    skip_rate = 100 * (attempts - qualified) / attempts if attempts else 0

    summary = (
        f"\n{'='*80}\n"
        f"SUMMARY\n"
        f"{'='*80}\n"
        f"  Qualifying games : {qualified}\n"
        f"  Total attempts   : {attempts}  (skip rate: {skip_rate:.1f}%)\n"
        f"  Elapsed time     : {elapsed:.1f}s\n"
        f"  Avg point loss   : {avg_loss:.2f} pts\n"
        f"  Avg optimal rate : {avg_opt:.1f}%\n"
        f"{'='*80}\n"
    )
    print(summary)
    with open(LOG_FILE, "a", encoding="utf-8") as log:
        log.write(summary)

    print(f"\nDone!  Log: {LOG_FILE.resolve()}")


if __name__ == "__main__":
    main()
