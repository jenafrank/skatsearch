"""
Parses pimc_strategy_comparison_log.txt and prints statistics on point losses
for each PIMC strategy (a-d), broken down by game type and by player role.

Strategies:
  a) Win Probability Only    (playout / estimate_probability_of_all_cards)
  b) Hybrid Mode / Avg FB    (points-playout hybrid average)
  c) Hybrid Mode / Min FB    (points-playout hybrid minimum)
  d) Average Points Only     (points-playout average)

For each game a "summary line" is emitted:
  Game Finished. Total Point Loss: N (D:N O:N) | Final score: N pts

"Loss" here is the deviation from the perfect-play optimal value.
  D = extra points lost by the Declarer's wrong moves (bad for Declarer)
  O = extra points won by Opponents due to their wrong moves (bad for Opponents)
  Total = D + O  (lower = better strategy overall)
"""

import re, os, sys
from collections import defaultdict

# Force UTF-8 output on Windows consoles
sys.stdout.reconfigure(encoding="utf-8", errors="replace")

LOG_FILE = "pimc_strategy_comparison_log.txt"

# ── Regex patterns ─────────────────────────────────────────────────────────────
RE_GAME_HEADER  = re.compile(r"^GAME (\d+) - Type: (\w+)")
RE_STRATEGY_HDR = re.compile(
    r"^--- ([abcd]\).*?) \[")          # "--- a) PIMC: Win Probability Only [..."
RE_SUMMARY      = re.compile(
    r"Game Finished\. Total Point Loss: (\d+) \(D:(\d+) O:(\d+)\) \| Final score: (\d+) pts")

# Short label → internal key
STRATEGY_KEYS = {
    "a": "WinProb",
    "b": "Hybrid-Avg",
    "c": "Hybrid-Min",
    "d": "Average",
}

def parse_log(path: str):
    """Returns list of dicts, one per completed game/strategy combination."""
    results = []
    cur_game = None
    cur_type = None
    cur_strategy = None

    with open(path, encoding="utf-8") as f:
        for line in f:
            line = line.rstrip("\n")

            m = RE_GAME_HEADER.match(line)
            if m:
                cur_game = int(m.group(1))
                cur_type = m.group(2).lower()   # "grand", "clubs", ...
                cur_strategy = None
                continue

            m = RE_STRATEGY_HDR.match(line)
            if m:
                label = m.group(1)  # e.g. "a) PIMC: Win Probability Only"
                key_letter = label[0]              # "a", "b", "c", "d"
                cur_strategy = STRATEGY_KEYS.get(key_letter, label)
                continue

            m = RE_SUMMARY.search(line)
            if m and cur_game is not None and cur_strategy is not None:
                total_loss = int(m.group(1))
                d_loss     = int(m.group(2))
                o_loss     = int(m.group(3))
                final_score = int(m.group(4))
                results.append({
                    "game":     cur_game,
                    "type":     cur_type,
                    "strategy": cur_strategy,
                    "total":    total_loss,
                    "d_loss":   d_loss,
                    "o_loss":   o_loss,
                    "score":    final_score,
                })

    return results


def stats(values: list[int]):
    if not values:
        return {"n": 0, "mean": 0, "median": 0, "min": 0, "max": 0}
    n = len(values)
    s = sorted(values)
    mean = sum(s) / n
    mid = n // 2
    median = s[mid] if n % 2 else (s[mid-1] + s[mid]) / 2
    return {"n": n, "mean": mean, "median": median, "min": s[0], "max": s[-1]}


def fmt(s: dict, key: str = "mean") -> str:
    v = s[key]
    return f"{v:6.2f}"


def print_table(title: str, rows: list[tuple], headers: list[str]):
    print(f"\n{'─' * 72}")
    print(f"  {title}")
    print(f"{'─' * 72}")
    col_w = [max(len(h), 14) for h in headers]
    col_w[0] = 16
    hdr = "  ".join(f"{h:>{col_w[i]}}" for i, h in enumerate(headers))
    print("  " + hdr)
    print("  " + "─" * (sum(col_w) + 2 * len(col_w)))
    for row in rows:
        line = "  ".join(f"{str(v):>{col_w[i]}}" for i, v in enumerate(row))
        print("  " + line)


def main():
    if not os.path.exists(LOG_FILE):
        print(f"Log file not found: {LOG_FILE}", file=sys.stderr)
        sys.exit(1)

    results = parse_log(LOG_FILE)
    if not results:
        print("No results parsed from log.", file=sys.stderr)
        sys.exit(1)

    strategies = list(STRATEGY_KEYS.values())
    game_types = ["grand", "clubs", "spades", "hearts", "diamonds"]

    # ── Build per-strategy collections ────────────────────────────────────────
    by_strat = defaultdict(lambda: {"total": [], "d": [], "o": [], "score": []})
    by_strat_type = defaultdict(lambda: {"total": [], "d": [], "o": []})

    games_seen = set()
    for r in results:
        games_seen.add(r["game"])
        s = r["strategy"]
        by_strat[s]["total"].append(r["total"])
        by_strat[s]["d"].append(r["d_loss"])
        by_strat[s]["o"].append(r["o_loss"])
        by_strat[s]["score"].append(r["score"])
        by_strat_type[(s, r["type"])]["total"].append(r["total"])
        by_strat_type[(s, r["type"])]["d"].append(r["d_loss"])
        by_strat_type[(s, r["type"])]["o"].append(r["o_loss"])

    n_games = len(games_seen)
    print(f"\n{'═' * 72}")
    print(f"  SKAT PIMC Strategy Analysis  —  {n_games} games")
    print(f"{'═' * 72}")
    print("  Loss = extra points vs. perfect play  (lower is better)")
    print("  D-Loss = Declarer's excess losses  |  O-Loss = Opponents' excess losses")

    # ── Overall summary table ──────────────────────────────────────────────────
    headers = ["Strategy", "N", "Mean Total", "Mean D-Loss", "Mean O-Loss",
               "Med Total", "Max Total", "Mean Score"]
    rows = []
    for s in strategies:
        d = by_strat[s]
        if not d["total"]:
            continue
        st = stats(d["total"])
        sd = stats(d["d"])
        so = stats(d["o"])
        ss = stats(d["score"])
        rows.append((
            s,
            st["n"],
            f"{st['mean']:5.2f}",
            f"{sd['mean']:5.2f}",
            f"{so['mean']:5.2f}",
            f"{st['median']:5.2f}",
            st["max"],
            f"{ss['mean']:5.1f}",
        ))
    print_table("OVERALL  (mean/median point loss per game)", rows, headers)

    # ── Best strategy per metric ───────────────────────────────────────────────
    print(f"\n{'─' * 72}")
    print("  BEST STRATEGY (lower loss = better)")
    print(f"{'─' * 72}")
    metrics = [
        ("Total loss (D+O combined)", lambda s: stats(by_strat[s]["total"])["mean"]),
        ("Declarer loss only",        lambda s: stats(by_strat[s]["d"])["mean"]),
        ("Opponent loss only",        lambda s: stats(by_strat[s]["o"])["mean"]),
    ]
    for label, key_fn in metrics:
        scored = [(key_fn(s), s) for s in strategies if by_strat[s]["total"]]
        scored.sort()
        winner = scored[0][1]
        runner = scored[1][1] if len(scored) > 1 else "—"
        print(f"  {label:40s}  ➜  {winner}  (2nd: {runner})")
        for val, s in scored:
            print(f"       {s:12s}  {val:5.2f}")

    # ── Per game-type breakdown ────────────────────────────────────────────────
    print(f"\n{'═' * 72}")
    print("  PER GAME TYPE  —  mean total point loss")
    print(f"{'═' * 72}")
    hdr2 = ["GameType"] + strategies
    rows2 = []
    for gt in game_types:
        row = [gt.capitalize()]
        for s in strategies:
            d = by_strat_type[(s, gt)]["total"]
            if d:
                row.append(f"{sum(d)/len(d):5.1f}  (n={len(d)})")
            else:
                row.append("—")
        rows2.append(row)
    # Print manually (variable-width)
    print(f"\n  {'GameType':<12}", end="")
    for s in strategies:
        print(f"  {s:>18}", end="")
    print()
    print("  " + "─" * 80)
    for row in rows2:
        print(f"  {row[0]:<12}", end="")
        for cell in row[1:]:
            print(f"  {cell:>18}", end="")
        print()

    # ── Declarer vs Opponent per game type ─────────────────────────────────────
    for role, role_key in [("Declarer", "d"), ("Opponents", "o")]:
        print(f"\n  {role} loss by game type:")
        print(f"  {'GameType':<12}", end="")
        for s in strategies:
            print(f"  {s:>14}", end="")
        print()
        print("  " + "─" * 70)
        for gt in game_types:
            print(f"  {gt.capitalize():<12}", end="")
            for s in strategies:
                d = by_strat_type[(s, gt)][role_key]
                cell = f"{sum(d)/len(d):5.1f}" if d else "  —  "
                print(f"  {cell:>14}", end="")
            print()

    # ── Verdict ────────────────────────────────────────────────────────────────
    print(f"\n{'═' * 72}")
    print("  VERDICT")
    print(f"{'═' * 72}")

    def rank(metric_key):
        scored = [(stats(by_strat[s][metric_key])["mean"], s)
                  for s in strategies if by_strat[s]["total"]]
        scored.sort()
        return scored  # (score, name), lowest = best

    total_rank    = rank("total")
    d_rank        = rank("d")
    o_rank        = rank("o")

    print(f"\n  Best for DECLARER (lowest D-Loss):")
    for i, (v, s) in enumerate(d_rank, 1):
        marker = " ✓" if i == 1 else ""
        print(f"    {i}. {s:<14}  avg D-Loss = {v:.2f}{marker}")

    print(f"\n  Best for OPPONENTS (lowest O-Loss):")
    for i, (v, s) in enumerate(o_rank, 1):
        marker = " ✓" if i == 1 else ""
        print(f"    {i}. {s:<14}  avg O-Loss = {v:.2f}{marker}")

    print(f"\n  Overall winner (lowest Total Loss = D+O combined):")
    for i, (v, s) in enumerate(total_rank, 1):
        marker = " ✓ WINNER" if i == 1 else ""
        print(f"    {i}. {s:<14}  avg Total Loss = {v:.2f}{marker}")

    print()


if __name__ == "__main__":
    main()
