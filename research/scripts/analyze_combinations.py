import csv
import collections
import sys

def parse_jacks_count(val):
    if val.isdigit():
        mask = int(val)
    else:
        mask = 0
        if 'C' in val: mask |= 8
        if 'S' in val: mask |= 4
        if 'H' in val: mask |= 2
        if 'D' in val: mask |= 1
    return bin(mask).count('1')

def analyze():
    filename = 'grand_pickup_1000.csv'
    if len(sys.argv) > 1:
        filename = sys.argv[1]
        
    print(f"Analyzing combinations in {filename}...")
    
    # Key: (Jacks, Aces, Tens) -> list of win probs
    stats = collections.defaultdict(list)
    
    try:
        with open(filename, 'r') as f:
            reader = csv.DictReader(f)
            for row in reader:
                try:
                    jacks = parse_jacks_count(row['JacksMask'])
                    aces = int(row['Aces'])
                    tens = int(row['Tens'])
                    win_prob = float(row['WinProb'])
                    
                    stats[(jacks, aces, tens)].append(win_prob)
                except (ValueError, KeyError):
                    continue
                    
    except FileNotFoundError:
        print("File not found.")
        return

    print("\nCombination | Avg Win % | Count | Reliability")
    print("-" * 50)
    
    # Sort by Jacks then Aces then Tens
    for combo in sorted(stats.keys(), reverse=True):
        probs = stats[combo]
        count = len(probs)
        avg_win = sum(probs) / count
        
        # Reliability: How many hands were actually > 50%?
        wins = len([p for p in probs if p > 0.5])
        rel = (wins / count) * 100
        
        star = ""
        if avg_win > 0.5: star = "*"
        if avg_win > 0.8: star = "**"
        
        print(f"J:{combo[0]} A:{combo[1]} T:{combo[2]} | {avg_win*100:5.1f}%    | {count:3d}   | {rel:5.1f}% {star}")

if __name__ == "__main__":
    analyze()
