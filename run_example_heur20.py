"""
Example output: Heur-20 PIMC playout (avg points + trump heuristic, 20 samples).

Runs 3 Grand + 3 Suit games and writes a fully annotated log showing:
  - Original 12-card hand with skat cards marked *
  - Per-card avgs with standard deviation and perfect-information value
  - Output markers:
      *   = chosen card deviates from perfect play
      ^   = trump heuristic overrode the pure avg-points best card
      (!) = the deviation caused a point loss

Log is written to heur20_example_output.txt.
"""
import subprocess
import os
import re
from datetime import datetime

GRAND_ROUNDS        = 3
SUIT_ROUNDS         = 3
TOTAL_ROUNDS        = GRAND_ROUNDS + SUIT_ROUNDS
DEAL_FILE           = "example_deal.json"
LOG_FILE            = "heur20_example_output.txt"
HEURISTIC_THRESHOLD = 2.0
MIN_SCORE           = 61


def run_cmd(args):
    result = subprocess.run(
        args, capture_output=True, text=True, encoding="utf-8", errors="replace"
    )
    return result.stdout, result.returncode


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


def parse_final_score(section_text):
    m = re.search(r"Final score:\s*(-?\d+)\s*pts", section_text)
    return int(m.group(1)) if m else None


def parse_total_loss(section_text):
    m = re.search(r"Total Point Loss:\s*(\d+)\s*\(D:(\d+)\s*O:(\d+)\)", section_text)
    if m:
        return int(m.group(1)), int(m.group(2)), int(m.group(3))
    return None, None, None


def parse_perfect_score(section_text):
    m = re.search(r"Perfect Play Finished\. Result:\s*(\d+)\s*pts", section_text)
    return int(m.group(1)) if m else None


def generate_deal(game_type_flag, attempts=25):
    for _ in range(attempts):
        out, rc = run_cmd([
            "cargo", "run", "--release", "--",
            "generate-smart-deal",
            "--min-value", str(MIN_SCORE),
            "--out", DEAL_FILE,
            "--game-type", game_type_flag,
        ])
        if rc == 0 and "SMART_DEAL_OK" in out:
            m_label  = re.search(r"SMART_DEAL_OK: game=(\S+)", out)
            m_disc   = re.search(r"discard=(.+)", out)
            m_orig12 = re.search(r"ORIGINAL_HAND_12: (.+)", out)
            m_pre    = re.search(r"HEURISTIC_PRE: (.+)", out)
            m_post   = re.search(r"HEURISTIC_POST: (.+)", out)
            m_samp   = re.search(r"HEURISTIC_SAMPLING: (.+)", out)
            return (
                m_label.group(1)           if m_label  else "?",
                m_disc.group(1).strip()    if m_disc   else "?",
                m_orig12.group(1).strip()  if m_orig12 else "?",
                m_pre.group(1)             if m_pre    else "?",
                m_post.group(1)            if m_post   else "?",
                m_samp.group(1)            if m_samp   else "?",
            )
    return None, None, None, None, None, None


def sampling_flag_for(label):
    return "smart-grand" if "Grand" in label else "smart-suit"


def run_playout(deal_file, sampling_mode):
    args = [
        "cargo", "run", "--release", "--",
        "points-playout",
        "-c", deal_file,
        "-d", sampling_mode,
        "--points-mode", "average",
        "-s", "20",
        "--trump-heuristic",
        "--trump-heuristic-threshold", str(HEURISTIC_THRESHOLD),
    ]
    out, _ = run_cmd(args)
    return out


def main():
    if os.path.exists(LOG_FILE):
        os.remove(LOG_FILE)

    scores  = []
    losses  = []
    game_schedule = (["grand"] * GRAND_ROUNDS) + (["suit"] * SUIT_ROUNDS)
    start_time = datetime.now()

    with open(LOG_FILE, "w", encoding="utf-8") as log:
        log.write("=" * 80 + "\n")
        log.write("Heur-20 PIMC Example Output\n")
        log.write("=" * 80 + "\n")
        log.write(f"Date   : {start_time.strftime('%Y-%m-%d %H:%M:%S')}\n")
        log.write(f"Games  : {GRAND_ROUNDS} Grand + {SUIT_ROUNDS} Suit\n")
        log.write(f"Mode   : PIMC average points + trump heuristic, 20 samples\n")
        log.write(f"Thresh : {HEURISTIC_THRESHOLD} pts\n\n")
        log.write("Output markers:\n")
        log.write("  *   = chosen card deviates from perfect play\n")
        log.write("  ^   = trump heuristic overrode the pure avg-points best card\n")
        log.write("  (!) = the deviation caused a point loss\n\n")
        log.write("avgs format: CARD=avg(\u00B1std_dev)|perfect_score\n\n")

        game_idx = 0
        for game_type_flag in game_schedule:
            game_idx += 1
            print(f"Game {game_idx}/{TOTAL_ROUNDS} ({game_type_flag.upper()})...", end=" ", flush=True)

            label, discard, orig12, pre_str, post_str, samp_str = generate_deal(game_type_flag)
            if label is None:
                print("SKIP")
                log.write(f"GAME {game_idx}: SKIPPED\n\n")
                continue

            sampling_mode = sampling_flag_for(label)
            is_suit = "Grand" not in label

            log.write("=" * 80 + "\n")
            log.write(f"GAME {game_idx}  ({label})\n")
            log.write("=" * 80 + "\n\n")

            # Heuristic analysis block
            log.write("--- Deal & Heuristic Analysis ---\n")
            log.write(f"  Game type    : {label}\n")
            log.write(f"  Original 12  : [{orig12}]\n")
            log.write(f"                  Cards marked * are in the skat (hidden before pickup)\n")
            log.write(f"  Skat discard : {discard}\n")
            log.write(f"\n  Pre-discard biddability (full 12-card hand):\n")
            log.write(f"    {pre_str}\n")
            log.write(f"\n  Post-discard playability (final 10-card hand):\n")
            log.write(f"    {post_str}\n")
            log.write(f"\n  PIMC sampling filter: {samp_str}\n\n")

            # Run playout
            out = run_playout(DEAL_FILE, sampling_mode)

            # Extract and write distribution
            dist_section = extract_section(out, "Cards Distribution", "=== Perfect Play Simulation ===")
            if dist_section:
                log.write("--- Cards Distribution ---\n")
                log.write(dist_section + "\n\n")

            # Perfect play (reference)
            perfect_section = extract_section(
                out, "=== Perfect Play Simulation ===", "=== PIMC Points Play Simulation ===")
            perf_score = parse_perfect_score(perfect_section)
            log.write("--- Perfect Play (Reference) ---\n")
            log.write(perfect_section + "\n\n")

            # Heur-20 playout
            heur_section = extract_section(out, "=== PIMC Points Play Simulation ===")
            heur_score = parse_final_score(heur_section)
            total_loss, d_loss, o_loss = parse_total_loss(heur_section)
            log.write("--- Heur-20 Playout ---\n")
            log.write(heur_section + "\n\n")

            if heur_score is not None:
                scores.append(heur_score)
            if total_loss is not None:
                losses.append(total_loss)

            wm = "WIN " if heur_score is not None and heur_score >= 61 else "LOSS"
            log.write(
                f"  >> Game {game_idx} ({label}):  "
                f"Perfect={perf_score}  Heur20={heur_score} {wm}  "
                f"total_loss={total_loss} (D={d_loss} O={o_loss})\n\n"
            )
            print(f"Perfect={perf_score}  Heur20={heur_score} ({wm.strip()})")

        # Summary
        end_time = datetime.now()
        log.write("=" * 80 + "\n")
        log.write("SUMMARY\n")
        log.write("=" * 80 + "\n\n")
        if scores:
            avg = sum(scores) / len(scores)
            wins = sum(1 for s in scores if s >= 61)
            avg_loss = sum(losses) / len(losses) if losses else 0.0
            log.write(f"  Games played : {len(scores)}\n")
            log.write(f"  Avg score    : {avg:.1f}\n")
            log.write(f"  Wins         : {wins}/{len(scores)}\n")
            log.write(f"  Avg loss     : {avg_loss:.1f} pts vs perfect play\n")
        log.write(f"\nFinished: {end_time.strftime('%Y-%m-%d %H:%M:%S')}\n")

    print(f"\nDone - log saved to {LOG_FILE}")


if __name__ == "__main__":
    main()
