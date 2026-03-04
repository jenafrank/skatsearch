import subprocess
import os

ROUNDS = 10
DEAL_FILE = "generated_smart_suit_deal.json"
LOG_FILE = "simulation_comparison_minimum_log.txt"

def run_cmd(args):
    result = subprocess.run(args, capture_output=True, text=True)
    return result.stdout

def extract_section(text, start_marker, end_marker=None):
    """Extract text from start_marker to end_marker (or end of string if None)."""
    idx = text.find(start_marker)
    if idx == -1:
        return ""
    start = idx + len(start_marker)
    if end_marker:
        end_idx = text.find(end_marker, start)
        if end_idx != -1:
            return text[start:end_idx].strip()
    return text[start:].strip()

def main():
    if os.path.exists(LOG_FILE):
        os.remove(LOG_FILE)

    with open(LOG_FILE, "w", encoding="utf-8") as log:
        log.write("=== SKAT Playout Comparative Analysis (Minimum Mode) ===\n")
        log.write("Deals generated with smart-grand heuristic (playable grand hand for Declarer)\n\n")

        for i in range(1, ROUNDS + 1):
            log.write(f"{'=' * 80}\n")
            log.write(f"GAME {i}\n")
            log.write(f"{'=' * 80}\n\n")

            # 1. Generate a valid smart-grand deal (declarer has a plausible winning hand)
            run_cmd([
                "cargo", "run", "--release", "--",
                "generate-deal",
                "--game-type", "grand",
                "--distribution", "smart-grand",
                "--out", DEAL_FILE
            ])

            # 2. Run playout (Perfect Play + Win Probability PIMC) using smart-grand sampling
            win_prob_out = run_cmd([
                "cargo", "run", "--release", "--",
                "playout",
                "--game-type", "grand",
                "-c", DEAL_FILE,
                "-d", "smart-grand"
            ])

            # Extract card distribution header
            dist_section = extract_section(win_prob_out, "Cards Distribution", "=== Perfect Play Simulation ===")
            if dist_section:
                log.write("Cards Distribution:\n")
                log.write(dist_section + "\n\n")

            # Extract start player / game type from config header
            config_section = extract_section(win_prob_out, "=== Playout Configuration ===", "Cards Distribution")
            if config_section:
                for line in config_section.splitlines():
                    if "Start Player" in line or "Game Type" in line:
                        log.write(line.strip() + "\n")
                log.write("\n")

            # Perfect Play
            perfect_section = extract_section(win_prob_out, "=== Perfect Play Simulation ===", "=== PIMC Play Simulation ===")
            log.write("--- Perfect Play (Reference) ---\n")
            if perfect_section:
                log.write(perfect_section + "\n\n")

            # b3: Win Probability only (standard PIMC)
            pimc_section = extract_section(win_prob_out, "=== PIMC Play Simulation ===")
            log.write("--- b3) PIMC: Win Probability Only ---\n")
            if pimc_section:
                log.write(pimc_section + "\n\n")

            # b1: Hybrid with Minimum fallback (smart-grand sampling)
            hybrid_out = run_cmd([
                "cargo", "run", "--release", "--",
                "points-playout",
                "--game-type", "grand",
                "-c", DEAL_FILE,
                "-d", "smart-grand",
                "--points-mode", "hybrid",
                "--hybrid-fallback", "minimum",
                "--hybrid-delta", "0.05"
            ])
            hybrid_section = extract_section(hybrid_out, "=== PIMC Points Play Simulation ===")
            log.write("--- b1) PIMC Points Playout: Hybrid Mode (Minimum Fallback) ---\n")
            if hybrid_section:
                log.write(hybrid_section + "\n\n")

            # b2: Minimum only (smart-grand sampling)
            min_out = run_cmd([
                "cargo", "run", "--release", "--",
                "points-playout",
                "--game-type", "grand",
                "-c", DEAL_FILE,
                "-d", "smart-grand",
                "--points-mode", "minimum"
            ])
            min_section = extract_section(min_out, "=== PIMC Points Play Simulation ===")
            log.write("--- b2) PIMC Points Playout: Minimum Mode Only ---\n")
            if min_section:
                log.write(min_section + "\n\n")

            log.write("\n")
            print(f"Game {i} finished.")

if __name__ == "__main__":
    main()
