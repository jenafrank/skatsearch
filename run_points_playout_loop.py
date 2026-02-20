#!/usr/bin/env python3
"""
run_points_playout_loop.py
100 Zufallsspiele mit points-playout (suit / declarer).
Schreibt jeden Spielverlauf SOFORT in points_playout_log.txt.
"""

import subprocess
import re
import sys
from pathlib import Path
from datetime import datetime

# ── Konfiguration ──────────────────────────────────────────────────────────────
N_GAMES   = 100
LOG_FILE  = Path("points_playout_log.txt")
SAMPLES   = 20
# ──────────────────────────────────────────────────────────────────────────────

SEP_THICK = "═" * 76
SEP_THIN  = "─" * 76


def run_game() -> str:
    result = subprocess.run(
        [r"target\release\skat_aug23.exe",
         "points-playout", "--game-type", "suit",
         "--start-player", "declarer",
         "--samples", str(SAMPLES)],
        capture_output=True, text=True,
        encoding="utf-8", errors="replace",
    )
    return result.stdout


def parse_result(output: str) -> dict:
    info = {}
    for key in ("Declarer", "Left", "Right", "Skat"):
        m = re.search(rf"^{key}\s*:\s*(.+)$", output, re.MULTILINE)
        info[key.lower()] = m.group(1).strip() if m else "?"

    m = re.search(r"Perfect Play Finished\. Result:\s*(\d+)", output)
    info["perfect_result"] = int(m.group(1)) if m else None

    # New format: "Total Point Loss: N (D:N O:N) | Final score: N pts"
    m = re.search(
        r"Total Point Loss:\s*(\d+)\s*\(D:(\d+)\s*O:(\d+)\)",
        output)
    if m:
        info["total_loss"]    = int(m.group(1))
        info["declarer_loss"] = int(m.group(2))
        info["opponent_loss"] = int(m.group(3))
    else:
        info["total_loss"] = info["declarer_loss"] = info["opponent_loss"] = None

    # Count card lines and lossless lines in new format
    all_moves     = len(re.findall(r"PIMC:", output))
    perfect_moves = len(re.findall(r"loss=0\(", output))
    info["all_moves"]     = all_moves
    info["perfect_moves"] = perfect_moves
    return info


def write_game(f, game_num: int, info: dict, raw: str):
    """Write full game trace + mini header to log, flush immediately."""

    won = (info["perfect_result"] or 0) >= 61
    result_tag = "WIN " if won else "LOSS"

    header_lines = [
        "",
        SEP_THICK,
        f"  GAME {game_num:3d}/{N_GAMES}  │  {result_tag}  │  "
        f"Perfect: {info['perfect_result']} pts  │  "
        f"PIMC Loss: {info['total_loss']} pts (D:{info['declarer_loss']} O:{info['opponent_loss']})"
        if info["total_loss"] is not None else
        f"  GAME {game_num:3d}/{N_GAMES}",
        SEP_THIN,
        f"  Declarer : {info['declarer']}",
        f"  Left     : {info['left']}",
        f"  Right    : {info['right']}",
        f"  Skat     : {info['skat']}",
        SEP_THIN,
    ]
    for line in header_lines:
        f.write(line + "\n")

    # Full game trace (strip cargo noise, keep only relevant lines)
    noise = re.compile(
        r"(warning:|error\[|note:|--\>|Compiling|Finished|Blocking|Downloading"
        r"|cargo|Checking|Running|Updating|\s*\||\s*=|^\s*$\n)"
    )
    in_pimc = False
    for line in raw.splitlines():
        strip = line.strip()
        # Skip cargo build noise
        if re.match(
            r"^(warning:|error\[|note:|\s*--|Compiling|Finished|Blocking|Running"
            r"|Updating|Downloading|Checking|cargo )", strip
        ):
            continue
        if strip == "":
            continue
        # Start of PIMC Points section
        if "PIMC Points Play" in strip:
            in_pimc = True
        f.write("  " + strip + "\n")

    pm = info["perfect_moves"]
    am = info["all_moves"]
    pct = 100 * pm / am if am else 0
    f.write(f"  {SEP_THIN}\n")
    f.write(f"  Optimal moves: {pm}/{am} ({pct:.1f}%)\n")
    f.flush()


def main():
    start_time = datetime.now()

    # Open log and write header immediately
    f = LOG_FILE.open("w", encoding="utf-8", buffering=1)  # line-buffered
    f.write(SEP_THICK + "\n")
    f.write(f"  PIMC Points-Playout  │  {N_GAMES} Suit Games  │  Declarer leads\n")
    f.write(f"  Started: {start_time:%Y-%m-%d %H:%M:%S}  │  Samples/move: {SAMPLES}\n")
    f.write(SEP_THICK + "\n")
    f.flush()

    print(f"Running {N_GAMES} games → {LOG_FILE.resolve()}")

    total_losses    = []
    perfect_results = []
    optimal_pcts    = []

    for i in range(1, N_GAMES + 1):
        print(f"  Game {i:3d}/{N_GAMES}", end="", flush=True)
        raw  = run_game()
        info = parse_result(raw)
        write_game(f, i, info, raw)   # written + flushed immediately

        if info["total_loss"] is not None:
            total_losses.append(info["total_loss"])
        if info["perfect_result"] is not None:
            perfect_results.append(info["perfect_result"])
        am = info.get("all_moves", 0)
        pm = info.get("perfect_moves", 0)
        pct = 100 * pm / am if am else 0
        optimal_pcts.append(pct)

        loss_str = f"{info['total_loss']:3}" if info['total_loss'] is not None else "  ?"
        print(f"  perfect={info['perfect_result']:3}  loss={loss_str}  "
              f"optimal={pct:.1f}%")

    # ── Final summary ──────────────────────────────────────────────────────────
    finish_time = datetime.now()
    elapsed     = finish_time - start_time

    def avg(lst): return sum(lst) / len(lst) if lst else 0.0

    summary = "\n".join([
        "",
        SEP_THICK,
        "  OVERALL SUMMARY",
        SEP_THICK,
        f"  Games played          : {N_GAMES}",
        f"  Elapsed               : {elapsed}",
        "",
        "  Perfect Play (optimal declarer points):",
        f"    Average             : {avg(perfect_results):.1f} pts",
        f"    Min / Max           : {min(perfect_results, default=0)} / {max(perfect_results, default=0)} pts",
        f"    Games won (≥61 pts) : {sum(1 for v in perfect_results if v >= 61)}/{len(perfect_results)}",
        "",
        "  PIMC Point Loss vs Perfect:",
        f"    Avg total loss      : {avg(total_losses):.2f} pts",
        f"    Min / Max           : {min(total_losses, default=0)} / {max(total_losses, default=0)} pts",
        "",
        "  Move quality (fraction of moves matching perfect play):",
        f"    Average             : {avg(optimal_pcts):.1f}%",
        f"    Min / Max           : {min(optimal_pcts, default=0):.1f}% / {max(optimal_pcts, default=0):.1f}%",
        SEP_THICK,
        "",
    ])
    f.write(summary)
    f.flush()
    f.close()

    print(summary)
    print(f"Done!  Log: {LOG_FILE.resolve()}")


if __name__ == "__main__":
    main()
