"""
Comparative PIMC playout: Perfect Play vs Average Points vs Average + Trump Heuristic.

Runs 10 games (5 Grand + 5 Suit) for each of the three modes and reports
final scores, win rates, and cumulative losses.

Output markers in the playout log:
  *  = chosen card deviates from perfect play
  ^  = trump heuristic overrode the pure avg-points best card (mode 3 only)
  (!) = the deviation caused a point loss
"""
import subprocess
import os
import re
from datetime import datetime

GRAND_ROUNDS = 5
SUIT_ROUNDS  = 5
TOTAL_ROUNDS = GRAND_ROUNDS + SUIT_ROUNDS

DEAL_FILE            = "generated_smart_suit_deal.json"
LOG_FILE             = "pimc_avg_heuristic_comparison_log.txt"
HEURISTIC_THRESHOLD  = 2.0
MIN_SCORE            = 61          # minimum perfect-play score accepted


# ── helpers ──────────────────────────────────────────────────────────────────

def run_cmd(args):
    result = subprocess.run(args, capture_output=True, text=True, encoding="utf-8", errors="replace")
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


def generate_deal(game_type_flag, attempts=20):
    """
    Calls generate-smart-deal (with retry) until a qualifying deal is produced.
    Returns (stdout, game_label, heuristic_pre, heuristic_post, heuristic_sampling).
    """
    for _ in range(attempts):
        out, rc = run_cmd([
            "cargo", "run", "--release", "--",
            "generate-smart-deal",
            "--min-value", str(MIN_SCORE),
            "--out", DEAL_FILE,
            "--game-type", game_type_flag,
        ])
        if rc == 0 and "SMART_DEAL_OK" in out:
            m_label   = re.search(r"SMART_DEAL_OK: game=(\S+)", out)
            m_discard = re.search(r"discard=(.+)", out)
            m_orig12  = re.search(r"ORIGINAL_HAND_12: (.+)", out)
            m_pre     = re.search(r"HEURISTIC_PRE: (.+)", out)
            m_post    = re.search(r"HEURISTIC_POST: (.+)", out)
            m_samp    = re.search(r"HEURISTIC_SAMPLING: (.+)", out)
            label    = m_label.group(1)           if m_label   else "?"
            discard  = m_discard.group(1).strip() if m_discard else "?"
            orig12   = m_orig12.group(1).strip()  if m_orig12  else "?"
            pre_str  = m_pre.group(1)             if m_pre     else "?"
            post_str = m_post.group(1)            if m_post    else "?"
            samp_str = m_samp.group(1)            if m_samp    else "?"
            return label, discard, orig12, pre_str, post_str, samp_str
    return None, None, None, None, None, None  # failed


def sampling_flag_for(game_label):
    """Map the game label returned by generate-smart-deal to the correct -d flag."""
    if "Grand" in game_label:
        return "smart-grand"
    return "smart-suit"


def run_points_playout(deal_file, sampling_mode, trump_heuristic=False):
    """Run points-playout (average mode, optionally with trump heuristic)."""
    args = [
        "cargo", "run", "--release", "--",
        "points-playout",
        "-c", deal_file,
        "-d", sampling_mode,
        "--points-mode", "average",
    ]
    if trump_heuristic:
        args += ["--trump-heuristic",
                 "--trump-heuristic-threshold", str(HEURISTIC_THRESHOLD)]
    out, _ = run_cmd(args)
    return out


# ── main ─────────────────────────────────────────────────────────────────────

def main():
    if os.path.exists(LOG_FILE):
        os.remove(LOG_FILE)

    # Summary accumulators  {mode: [scores]}
    perf_scores  = []
    avg_scores   = []
    heur_scores  = []
    avg_losses   = []
    heur_losses  = []

    # schedule: 5 Grand then 5 Suit
    game_schedule = (["grand"] * GRAND_ROUNDS) + (["suit"] * SUIT_ROUNDS)

    with open(LOG_FILE, "w", encoding="utf-8") as log:
        log.write("=" * 80 + "\n")
        log.write("SKAT PIMC Comparison: Average Points  vs  Average + Trump Heuristic\n")
        log.write("=" * 80 + "\n")
        log.write(f"Date       : {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}\n")
        log.write(f"Games      : {GRAND_ROUNDS} Grand + {SUIT_ROUNDS} Suit  (total {TOTAL_ROUNDS})\n")
        log.write(f"Min score  : {MIN_SCORE} pts (perfect-play threshold for deal acceptance)\n")
        log.write(f"Modes      :\n")
        log.write(f"  1) Perfect Play (reference, omniscient)\n")
        log.write(f"  2) PIMC Average Points Only\n")
        log.write(f"  3) PIMC Average Points + Trump Move Heuristic (threshold={HEURISTIC_THRESHOLD} pts)\n")
        log.write(f"\nTrump Move Heuristic explanation:\n")
        log.write(f"  Within {HEURISTIC_THRESHOLD} avg-pts of the best-scored card:\n")
        log.write(f"    Declarer  → always prefers a trump card\n")
        log.write(f"    Opponents → always prefer a non-trump card\n")
        log.write(f"  Output marker '^' = heuristic overrode the pure avg-points choice\n\n")

        game_idx = 0
        for game_type_flag in game_schedule:
            game_idx += 1
            print(f"Game {game_idx}/{TOTAL_ROUNDS} ({game_type_flag.upper()})...", end=" ", flush=True)

            # ── Generate deal ────────────────────────────────────────────────
            label, discard, orig12, pre_str, post_str, samp_str = generate_deal(game_type_flag)
            if label is None:
                print("SKIP (no qualifying deal after retries)")
                log.write(f"GAME {game_idx}: SKIPPED - no qualifying {game_type_flag} deal generated\n\n")
                continue

            sampling_mode = sampling_flag_for(label)
            is_suit_game = "Grand" not in label

            log.write("=" * 80 + "\n")
            log.write(f"GAME {game_idx}  ({label})\n")
            log.write("=" * 80 + "\n\n")

            # ── Heuristic Analysis block ─────────────────────────────────────
            log.write("--- Heuristic Analysis (Deal Selection Criteria) ---\n")
            log.write(f"  Game type selected  : {label}\n")
            log.write(f"\n  Original 12-card hand (before skat discard, in transformed suit space):\n")
            log.write(f"       [{orig12}]\n")
            log.write(f"  Optimal skat discard: {discard}\n")
            log.write(f"  Post-discard 10-card hand: [{orig12}] minus [{discard}]\n")

            log.write(f"\n  1) Biddability check (pre-discard, on the full 12-card hand)\n")
            log.write(f"       {pre_str}\n")
            log.write(f"       Meaning: T=Jacks+longest_suit, J=Jacks, S=Aces+attached_Tens.\n")
            log.write(f"       BIDDABLE => hand qualifies to bid (50k-sample boundary).\n")

            log.write(f"\n  2) Playability check (post-discard, final 10-card hand)\n")
            log.write(f"       {post_str}\n")
            if not is_suit_game:
                log.write(f"       Meaning (Grand): J=Jacks, S=Aces+attached_Tens.\n")
                log.write(f"       PLAYABLE => post-discard hand wins >=50% in 10k simulations.\n")
            else:
                log.write(f"       Meaning (Suit/Clubs): TC=total trump count (Jacks+Clubs), S=side strength.\n")
                log.write(f"       PLAYABLE => post-discard hand wins >=50% in 10k simulations.\n")

            log.write(f"\n  3) PIMC sampling filter used during playout\n")
            log.write(f"       {samp_str}\n")
            log.write(f"       Only distributions where the declarer's sampled hand passes\n")
            if not is_suit_game:
                log.write(f"       the Grand post-discard filter (J/S boundary) are accepted.\n")
            else:
                log.write(f"       the Suit/Clubs post-discard filter (TC/S boundary) are accepted.\n")

            log.write(f"\n  4) Trump move heuristic (mode 3 only)\n")
            log.write(f"       Threshold : {HEURISTIC_THRESHOLD} avg-pts\n")
            log.write(f"       Declarer  : within {HEURISTIC_THRESHOLD} pts -> prefer trump card (Grand+Suit)\n")
            if is_suit_game:
                log.write(f"       Opponents : within {HEURISTIC_THRESHOLD} pts -> prefer non-trump card (Suit game)\n")
            else:
                log.write(f"       Opponents : no trump preference (Grand game - only 4 Jacks are trump)\n")
            log.write(f"       3rd player: if all threshold-candidates win trick -> play highest value card\n")
            log.write(f"                   if all threshold-candidates lose trick -> play lowest value card\n")
            log.write(f"       Marker '^': appears next to '*' (perf-deviation) when heuristic overrides\n\n")

            # ── Mode 2: Average Points Only ──────────────────────────────────
            avg_out = run_points_playout(DEAL_FILE, sampling_mode, trump_heuristic=False)

            # Extract distribution from avg_out (shared for all modes)
            dist_section = extract_section(
                avg_out, "Cards Distribution", "=== Perfect Play Simulation ===")
            if dist_section:
                log.write("Cards Distribution:\n")
                log.write(dist_section + "\n\n")

            # Perfect play (embedded in the points-playout run)
            perfect_section = extract_section(
                avg_out,
                "=== Perfect Play Simulation ===",
                "=== PIMC Points Play Simulation ===")
            perf_score = parse_perfect_score(perfect_section)
            if perf_score is not None:
                perf_scores.append(perf_score)
            log.write("--- 1) Perfect Play (Reference) ---\n")
            log.write(perfect_section + "\n\n")

            # Average Points Only section
            avg_section = extract_section(avg_out, "=== PIMC Points Play Simulation ===")
            avg_score = parse_final_score(avg_section)
            avg_loss, avg_d_loss, avg_o_loss = parse_total_loss(avg_section)
            if avg_score is not None:
                avg_scores.append(avg_score)
            if avg_loss is not None:
                avg_losses.append(avg_loss)
            log.write("--- 2) PIMC Average Points Only ---\n")
            log.write(avg_section + "\n\n")

            # ── Mode 3: Average Points + Trump Heuristic ─────────────────────
            heur_out = run_points_playout(DEAL_FILE, sampling_mode, trump_heuristic=True)
            heur_section = extract_section(heur_out, "=== PIMC Points Play Simulation ===")
            heur_score = parse_final_score(heur_section)
            heur_loss, heur_d_loss, heur_o_loss = parse_total_loss(heur_section)
            if heur_score is not None:
                heur_scores.append(heur_score)
            if heur_loss is not None:
                heur_losses.append(heur_loss)
            log.write(f"--- 3) PIMC Average Points + Trump Heuristic (threshold={HEURISTIC_THRESHOLD} pts) ---\n")
            log.write(heur_section + "\n\n")

            # Per-game summary
            def wm(s):
                return "WIN " if s is not None and s >= 61 else ("LOSS" if s is not None else "?   ")

            summary = (
                f"  >> Game {game_idx} ({label}) summary:\n"
                f"       Perfect Play  : {perf_score} pts\n"
                f"       Avg Only      : {avg_score} pts  {wm(avg_score)}  total_loss={avg_loss}"
                f"  (D={avg_d_loss} O={avg_o_loss})\n"
                f"       Avg+Heuristic : {heur_score} pts  {wm(heur_score)}  total_loss={heur_loss}"
                f"  (D={heur_d_loss} O={heur_o_loss})\n"
            )
            log.write(summary + "\n")
            print(f"Perf={perf_score}  Avg={avg_score}({wm(avg_score).strip()})  "
                  f"Heur={heur_score}({wm(heur_score).strip()})")

        # ── Final Summary ─────────────────────────────────────────────────────
        log.write("=" * 80 + "\n")
        log.write("FINAL SUMMARY\n")
        log.write("=" * 80 + "\n\n")

        def stats(scores, label):
            if not scores:
                return f"{label}: no data\n"
            avg = sum(scores) / len(scores)
            wins = sum(1 for s in scores if s >= 61)
            return (f"{label}: avg_score={avg:.1f}  wins={wins}/{len(scores)}"
                    f"  scores={scores}\n")

        def loss_stats(losses, label):
            if not losses:
                return f"{label}: no data\n"
            avg = sum(losses) / len(losses)
            return f"{label}: avg_cumulative_loss={avg:.1f}  per_game={losses}\n"

        log.write(stats(perf_scores,  "1) Perfect Play    "))
        log.write(stats(avg_scores,   "2) Average Only    "))
        log.write(stats(heur_scores,  "3) Avg+Heuristic   "))
        log.write("\n")
        log.write(loss_stats(avg_losses,  "2) Average Only  "))
        log.write(loss_stats(heur_losses, "3) Avg+Heuristic "))
        log.write("\n")

        # Verdict
        if avg_scores and heur_scores:
            avg_mean   = sum(avg_scores)  / len(avg_scores)
            heur_mean  = sum(heur_scores) / len(heur_scores)
            avg_wins   = sum(1 for s in avg_scores  if s >= 61)
            heur_wins  = sum(1 for s in heur_scores if s >= 61)
            avg_lm     = sum(avg_losses)  / len(avg_losses)  if avg_losses  else 0.0
            heur_lm    = sum(heur_losses) / len(heur_losses) if heur_losses else 0.0

            log.write("VERDICT:\n")
            if heur_mean > avg_mean:
                log.write(f"  >> Avg+Heuristic is BETTER  by {heur_mean - avg_mean:.1f} avg pts"
                          f"  ({heur_wins} vs {avg_wins} wins)\n")
            elif avg_mean > heur_mean:
                log.write(f"  >> Average Only  is BETTER  by {avg_mean - heur_mean:.1f} avg pts"
                          f"  ({avg_wins} vs {heur_wins} wins)\n")
            else:
                log.write("  >> Both strategies perform equally in terms of avg score.\n")
            log.write(f"     Avg loss per game: Avg={avg_lm:.1f}  Heur={heur_lm:.1f}"
                      f"  (lower = closer to perfect play)\n")

    print(f"\nDone - results saved to {LOG_FILE}")


if __name__ == "__main__":
    main()
