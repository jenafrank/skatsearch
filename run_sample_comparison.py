"""
Sample-count comparison: 20 vs 100 PIMC samples, with and without trump heuristic.

5 modes per game (same deal for all):
  1) Perfect Play            (reference, deterministic)
  2) Avg-20   – average points, 20 samples
  3) Avg-100  – average points, 100 samples
  4) Heur-20  – average + trump heuristic, 20 samples
  5) Heur-100 – average + trump heuristic, 100 samples

100 games: 50 Grand + 50 Suit.
Each game uses the IDENTICAL deal JSON for all five runs.

Runtime estimate: ~4–6 hours on a modern desktop.
"""

import subprocess
import os
import re
from datetime import datetime

# ── Configuration ──────────────────────────────────────────────────────────────
GRAND_ROUNDS         = 50
SUIT_ROUNDS          = 50
TOTAL_ROUNDS         = GRAND_ROUNDS + SUIT_ROUNDS
DEAL_FILE            = "sample_cmp_deal.json"
LOG_FILE             = "pimc_sample_comparison_log.txt"
HEURISTIC_THRESHOLD  = 2.0
MIN_SCORE            = 61


# ── Helpers ────────────────────────────────────────────────────────────────────

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
    """Generate a qualifying smart deal. Returns (label, discard, orig12, pre, post, samp) or Nones."""
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


def run_playout(deal_file, sampling_mode, samples, trump_heuristic=False):
    args = [
        "cargo", "run", "--release", "--",
        "points-playout",
        "-c", deal_file,
        "-d", sampling_mode,
        "--points-mode", "average",
        "-s", str(samples),
    ]
    if trump_heuristic:
        args += [
            "--trump-heuristic",
            "--trump-heuristic-threshold", str(HEURISTIC_THRESHOLD),
        ]
    out, _ = run_cmd(args)
    return out


# ── Accumulators ──────────────────────────────────────────────────────────────

class Stats:
    def __init__(self, label):
        self.label  = label
        self.scores = []
        self.losses = []

    def add(self, score, loss):
        if score is not None:
            self.scores.append(score)
        if loss is not None:
            self.losses.append(loss)

    def n(self):       return len(self.scores)
    def avg(self):     return sum(self.scores) / len(self.scores) if self.scores else 0.0
    def wins(self):    return sum(1 for s in self.scores if s >= 61)
    def win_pct(self): return 100.0 * self.wins() / len(self.scores) if self.scores else 0.0
    def avg_loss(self):return sum(self.losses) / len(self.losses) if self.losses else 0.0

    def summary_line(self):
        return (
            f"{self.label:<12}: n={self.n():>3}  "
            f"avg={self.avg():>5.1f}  "
            f"wins={self.wins():>3}/{self.n():<3} ({self.win_pct():>5.1f}%)  "
            f"avg_loss={self.avg_loss():>5.1f}"
        )


# ── Main ──────────────────────────────────────────────────────────────────────

def main():
    if os.path.exists(LOG_FILE):
        os.remove(LOG_FILE)

    perf   = Stats("Perfect")
    avg20  = Stats("Avg-20")
    avg100 = Stats("Avg-100")
    h20    = Stats("Heur-20")
    h100   = Stats("Heur-100")

    game_schedule = (["grand"] * GRAND_ROUNDS) + (["suit"] * SUIT_ROUNDS)
    start_time = datetime.now()

    with open(LOG_FILE, "w", encoding="utf-8") as log:
        # ── Header ────────────────────────────────────────────────────────────
        log.write("=" * 80 + "\n")
        log.write("PIMC Sample-Count Comparison  (20 vs 100 samples)\n")
        log.write("=" * 80 + "\n")
        log.write(f"Started  : {start_time.strftime('%Y-%m-%d %H:%M:%S')}\n")
        log.write(f"Games    : {GRAND_ROUNDS} Grand + {SUIT_ROUNDS} Suit = {TOTAL_ROUNDS} total\n")
        log.write(f"Min score: {MIN_SCORE} pts (perfect-play acceptance threshold)\n")
        log.write(f"Heuristic: trump threshold = {HEURISTIC_THRESHOLD} pts\n\n")
        log.write("Modes per game (identical deal JSON for all five):\n")
        log.write("  1) Perfect Play  – deterministic reference\n")
        log.write("  2) Avg-20        – average points, 20 PIMC samples\n")
        log.write("  3) Avg-100       – average points, 100 PIMC samples\n")
        log.write("  4) Heur-20       – avg + trump heuristic, 20 samples\n")
        log.write("  5) Heur-100      – avg + trump heuristic, 100 samples\n\n")

        # ── Game loop ─────────────────────────────────────────────────────────
        game_idx = 0
        for game_type_flag in game_schedule:
            game_idx += 1
            elapsed = (datetime.now() - start_time).seconds // 60
            ts = datetime.now().strftime("%H:%M:%S")
            print(
                f"[{ts}] Game {game_idx:>3}/{TOTAL_ROUNDS} "
                f"({game_type_flag.upper()}, +{elapsed}min)...",
                end=" ", flush=True,
            )

            # Generate deal
            label, discard, orig12, pre_str, post_str, samp_str = generate_deal(game_type_flag)
            if label is None:
                print("SKIP")
                log.write(f"GAME {game_idx}: SKIPPED\n\n")
                continue

            sampling_mode = sampling_flag_for(label)
            is_suit = "Grand" not in label

            log.write("=" * 80 + "\n")
            log.write(f"GAME {game_idx:>3}  ({label})  [{ts}]\n")
            log.write("=" * 80 + "\n\n")

            # Heuristic block
            log.write("--- Heuristic Analysis ---\n")
            log.write(f"  Game type    : {label}\n")
            log.write(f"  Original 12  : [{orig12}]\n")
            log.write(f"  Skat discard : {discard}\n")
            log.write(f"  Pre-discard  : {pre_str}\n")
            log.write(f"  Post-discard : {post_str}\n")
            log.write(f"  PIMC filter  : {samp_str}\n\n")

            # ── Run 1: Avg-20 (also gives us Perfect Play) ────────────────────
            out20 = run_playout(DEAL_FILE, sampling_mode, 20, trump_heuristic=False)

            dist_section = extract_section(
                out20, "Cards Distribution", "=== Perfect Play Simulation ===")
            if dist_section:
                log.write("Cards Distribution:\n" + dist_section + "\n\n")

            perf_section = extract_section(
                out20,
                "=== Perfect Play Simulation ===",
                "=== PIMC Points Play Simulation ===")
            perf_score = parse_perfect_score(perf_section)
            perf.add(perf_score, 0)

            avg20_section = extract_section(out20, "=== PIMC Points Play Simulation ===")
            s20 = parse_final_score(avg20_section)
            l20, _, _ = parse_total_loss(avg20_section)
            avg20.add(s20, l20)

            log.write("--- 1) Perfect Play ---\n" + perf_section + "\n\n")
            log.write("--- 2) Avg-20 ---\n" + avg20_section + "\n\n")

            # ── Run 2: Avg-100 ────────────────────────────────────────────────
            out100 = run_playout(DEAL_FILE, sampling_mode, 100, trump_heuristic=False)
            avg100_section = extract_section(out100, "=== PIMC Points Play Simulation ===")
            s100 = parse_final_score(avg100_section)
            l100, _, _ = parse_total_loss(avg100_section)
            avg100.add(s100, l100)
            log.write("--- 3) Avg-100 ---\n" + avg100_section + "\n\n")

            # ── Run 3: Heur-20 ────────────────────────────────────────────────
            outh20 = run_playout(DEAL_FILE, sampling_mode, 20, trump_heuristic=True)
            heur20_section = extract_section(outh20, "=== PIMC Points Play Simulation ===")
            sh20 = parse_final_score(heur20_section)
            lh20, _, _ = parse_total_loss(heur20_section)
            h20.add(sh20, lh20)
            log.write("--- 4) Heur-20 ---\n" + heur20_section + "\n\n")

            # ── Run 4: Heur-100 ──────────────────────────────────────────────
            outh100 = run_playout(DEAL_FILE, sampling_mode, 100, trump_heuristic=True)
            heur100_section = extract_section(outh100, "=== PIMC Points Play Simulation ===")
            sh100 = parse_final_score(heur100_section)
            lh100, _, _ = parse_total_loss(heur100_section)
            h100.add(sh100, lh100)
            log.write("--- 5) Heur-100 ---\n" + heur100_section + "\n\n")

            # Per-game summary
            def wm(s):
                return "WIN " if s is not None and s >= 61 else "LOSS"
            summary = (
                f"  >> GAME {game_idx} ({label}):  "
                f"Perf={perf_score}  "
                f"Avg20={s20}({wm(s20)},L={l20})  "
                f"Avg100={s100}({wm(s100)},L={l100})  "
                f"H20={sh20}({wm(sh20)},L={lh20})  "
                f"H100={sh100}({wm(sh100)},L={lh100})\n"
            )
            log.write(summary + "\n")
            log.flush()

            print(
                f"Perf={perf_score}  "
                f"Avg20={s20}({wm(s20)})  Avg100={s100}({wm(s100)})  "
                f"H20={sh20}({wm(sh20)})  H100={sh100}({wm(sh100)})"
            )

        # ── Final Summary ─────────────────────────────────────────────────────
        end_time = datetime.now()
        duration = end_time - start_time
        hours, rem = divmod(int(duration.total_seconds()), 3600)
        mins = rem // 60

        log.write("=" * 80 + "\n")
        log.write("FINAL SUMMARY\n")
        log.write("=" * 80 + "\n\n")
        log.write(f"Total runtime: {hours}h {mins}min\n\n")

        log.write("Per-mode statistics (all games):\n")
        for st in [perf, avg20, avg100, h20, h100]:
            log.write("  " + st.summary_line() + "\n")

        # Sample-count delta table
        log.write("\nSample-count effect (100 vs 20 samples, same mode):\n")
        d_avg  = avg100.avg()  - avg20.avg()
        d_heur = h100.avg()    - h20.avg()
        dw_avg  = avg100.wins()  - avg20.wins()
        dw_heur = h100.wins()    - h20.wins()
        dl_avg  = avg100.avg_loss() - avg20.avg_loss()
        dl_heur = h100.avg_loss()   - h20.avg_loss()
        log.write(
            f"  Average mode : Δscore={d_avg:+.1f}  Δwins={dw_avg:+d}"
            f"  Δloss={dl_avg:+.1f}  "
            f"({'100-sample better' if d_avg > 0 else '20-sample better' if d_avg < 0 else 'tied'})\n"
        )
        log.write(
            f"  Heuristic    : Δscore={d_heur:+.1f}  Δwins={dw_heur:+d}"
            f"  Δloss={dl_heur:+.1f}  "
            f"({'100-sample better' if d_heur > 0 else '20-sample better' if d_heur < 0 else 'tied'})\n"
        )

        # Heuristic effect per sample count
        log.write("\nHeuristic effect (heuristic vs plain avg, same sample count):\n")
        d20   = h20.avg()   - avg20.avg()
        d100  = h100.avg()  - avg100.avg()
        dw20  = h20.wins()  - avg20.wins()
        dw100 = h100.wins() - avg100.wins()
        log.write(
            f"  At 20  samples: Δscore={d20:+.1f}  Δwins={dw20:+d}"
            f"  ({'heuristic better' if d20 > 0 else 'plain better' if d20 < 0 else 'tied'})\n"
        )
        log.write(
            f"  At 100 samples: Δscore={d100:+.1f}  Δwins={dw100:+d}"
            f"  ({'heuristic better' if d100 > 0 else 'plain better' if d100 < 0 else 'tied'})\n"
        )

        # Verdict
        log.write("\nVERDICT:\n")
        best_mode = max([avg20, avg100, h20, h100], key=lambda s: s.avg())
        log.write(f"  Best overall mode: {best_mode.label}  (avg score {best_mode.avg():.1f})\n")

        sample_matters = abs(d_avg) >= 1.0 or abs(d_heur) >= 1.0
        log.write(
            f"  Sample count matters: {'YES' if sample_matters else 'NO (< 1 pt difference)'}\n"
        )
        heur_helps = d20 > 0.5 or d100 > 0.5
        log.write(
            f"  Heuristic helps: {'YES' if heur_helps else 'NO (< 0.5 pt difference)'}\n"
        )

        log.write(f"\nFinished: {end_time.strftime('%Y-%m-%d %H:%M:%S')}  "
                  f"(runtime {hours}h {mins}min)\n")

    print(f"\nDone - results saved to {LOG_FILE}  (runtime {hours}h {mins}min)")


if __name__ == "__main__":
    main()
