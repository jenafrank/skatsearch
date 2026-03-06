import subprocess
import os
import json
from datetime import datetime

ROUNDS = 3
DEAL_FILE = "temp_comparison_deal.json"
LOG_FILE = "pimc_strategy_comparison_log.txt"

# Only accept deals where perfect-play score >= MIN_SCORE.
# 61 = guaranteed win = approximates >70% real win probability.
MIN_SCORE = 61

def run_cmd(args):
    result = subprocess.run(args, capture_output=True, text=True, encoding="utf-8")
    return result.stdout

def extract_section(text, start_marker, end_marker=None):
    idx = text.find(start_marker)
    if idx == -1:
        return ""
    start = idx + len(start_marker)
    if end_marker:
        end_idx = text.find(end_marker, start)
        if end_idx != -1:
            return text[start:end_idx].strip()
    return text[start:].strip()

def get_timestamp():
    return datetime.now().strftime("%Y-%m-%d %H:%M:%S")

def generate_smart_deal():
    """
    Calls generate-smart-deal in a retry loop until a qualifying deal is found.
    Returns (game_type_str, dist_mode) where game_type_str is "grand" or "clubs".
    The deal JSON is written to DEAL_FILE.

    Selection logic (inside Rust):
      - Shuffles all 32 cards; Declarer gets 12 (incl. skat), Left/Right each 10.
      - Evaluates best game across Grand + all Suit variants (Spades/Hearts/Diamonds
        are transformed to Clubs internally so only one Suit type matters).
      - Applies optimal 2-card skat discard via perfect-information solver.
      - Accepts only deals where perfect-play score >= MIN_SCORE (61 = guaranteed win,
        which corresponds to the >70% winning-probability heuristic threshold).
    """
    attempts = 0
    while True:
        attempts += 1
        result = subprocess.run(
            [
                "cargo", "run", "--release", "--",
                "generate-smart-deal",
                "--min-value", str(MIN_SCORE),
                "--out", DEAL_FILE,
            ],
            capture_output=True, text=True, encoding="utf-8"
        )
        if result.returncode == 0:
            with open(DEAL_FILE, encoding="utf-8") as f:
                deal = json.load(f)
            game_type_json = deal.get("game_type", "Suit")
            # Extract heuristic log lines emitted by generate-smart-deal
            heuristic_lines = [
                line for line in result.stdout.splitlines()
                if line.startswith("HEURISTIC_")
            ]
            if game_type_json == "Grand":
                return ("grand", "smart-grand", attempts, heuristic_lines)
            else:
                # Suit (Clubs/Spades/Hearts/Diamonds) — all mapped to Clubs internally
                return ("clubs", "smart-suit", attempts, heuristic_lines)
        # exit code 2 = deal didn't qualify, retry silently

def main():
    if os.path.exists(LOG_FILE):
        os.remove(LOG_FILE)

    with open(LOG_FILE, "w", encoding="utf-8") as log:
        log.write(f"=== SKAT PIMC Strategy Comparative Analysis ({ROUNDS} Games) ===\n")
        log.write(f"Started at: {get_timestamp()}\n")
        log.write(f"Deal selection: generate-smart-deal --min-value {MIN_SCORE} (guaranteed win, heuristic-filtered)\n")
        log.write("All suit games transformed to Clubs internally.\n")
        log.write("Comparing: a) WinProb (100 samples), b) Hybrid-Avg, c) Hybrid-Min, d) Average Only\n\n")

        for i in range(1, ROUNDS + 1):
            game_type, dist_mode, attempts, heuristic_lines = generate_smart_deal()

            log.write(f"{'=' * 80}\n")
            log.write(f"GAME {i} - Type: {game_type.upper()} (min_score={MIN_SCORE}, attempts={attempts})\n")
            log.write(f"Timestamp: {get_timestamp()}\n")
            log.write(f"{'=' * 80}\n\n")

            # Log heuristic findings for this deal
            log.write("--- Heuristic Analysis ---\n")
            for hl in heuristic_lines:
                # Strip the HEURISTIC_ prefix and format nicely
                tag, _, rest = hl.partition(": ")
                category = tag.replace("HEURISTIC_", "")
                if category == "PRE":
                    log.write(f"  1) Game biddable? {rest}\n")
                elif category == "POST":
                    log.write(f"  2) Post-discard strength: {rest}\n")
                elif category == "SAMPLING":
                    log.write(f"  3) PIMC sampling filter: {rest}\n")
            log.write("\n")

            # a) Win Probability PIMC — also runs Perfect Play benchmark
            start_t = get_timestamp()
            win_prob_out = run_cmd([
                "cargo", "run", "--release", "--",
                "playout",
                "--game-type", game_type,
                "-c", DEAL_FILE,
                "-d", dist_mode,
                "--samples", "100"
            ])
            end_t = get_timestamp()

            dist_section = extract_section(win_prob_out, "Cards Distribution", "=== Perfect Play Simulation ===")
            if dist_section:
                log.write("Cards Distribution:\n")
                log.write(dist_section + "\n\n")

            perfect_section = extract_section(win_prob_out, "=== Perfect Play Simulation ===", "=== PIMC Play Simulation ===")
            log.write(f"--- Perfect Play (Benchmark) [{start_t}] ---\n")
            if perfect_section:
                log.write(perfect_section + "\n\n")

            pimc_section = extract_section(win_prob_out, "=== PIMC Play Simulation ===")
            log.write(f"--- a) PIMC: Win Probability Only [{start_t} - {end_t}] ---\n")
            if pimc_section:
                log.write(pimc_section + "\n\n")

            # b) Hybrid with Average fallback
            start_t = get_timestamp()
            hybrid_avg_out = run_cmd([
                "cargo", "run", "--release", "--",
                "points-playout",
                "--game-type", game_type,
                "-c", DEAL_FILE,
                "-d", dist_mode,
                "--points-mode", "hybrid",
                "--hybrid-fallback", "average",
                "--hybrid-delta", "0.05"
            ])
            end_t = get_timestamp()
            log.write(f"--- b) PIMC: Hybrid Mode (Average Fallback) [{start_t} - {end_t}] ---\n")
            section = extract_section(hybrid_avg_out, "=== PIMC Points Play Simulation ===")
            if section:
                log.write(section + "\n\n")

            # c) Hybrid with Minimum fallback
            start_t = get_timestamp()
            hybrid_min_out = run_cmd([
                "cargo", "run", "--release", "--",
                "points-playout",
                "--game-type", game_type,
                "-c", DEAL_FILE,
                "-d", dist_mode,
                "--points-mode", "hybrid",
                "--hybrid-fallback", "minimum",
                "--hybrid-delta", "0.05"
            ])
            end_t = get_timestamp()
            log.write(f"--- c) PIMC: Hybrid Mode (Minimum Fallback) [{start_t} - {end_t}] ---\n")
            section = extract_section(hybrid_min_out, "=== PIMC Points Play Simulation ===")
            if section:
                log.write(section + "\n\n")

            # d) Average Points Only
            start_t = get_timestamp()
            avg_out = run_cmd([
                "cargo", "run", "--release", "--",
                "points-playout",
                "--game-type", game_type,
                "-c", DEAL_FILE,
                "-d", dist_mode,
                "--points-mode", "average"
            ])
            end_t = get_timestamp()
            log.write(f"--- d) PIMC: Average Points Only [{start_t} - {end_t}] ---\n")
            section = extract_section(avg_out, "=== PIMC Points Play Simulation ===")
            if section:
                log.write(section + "\n\n")

            log.write("\n")
            print(f"Game {i} ({game_type}, attempts={attempts}) finished at {get_timestamp()}.")

        log.write(f"\nSimulation ended at: {get_timestamp()}\n")

if __name__ == "__main__":
    main()
