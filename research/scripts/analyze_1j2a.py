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
        
    print(f"Analyzing 1J + 2A Winners in {filename}...")
    
    winners = []
    
    try:
        with open(filename, 'r') as f:
            reader = csv.DictReader(f)
            for row in reader:
                try:
                    jacks = parse_jacks_count(row['JacksMask'])
                    aces = int(row['Aces'])
                    win_prob = float(row['WinProb'])
                    
                    if jacks == 1 and aces == 2 and win_prob > 0.5:
                        winners.append((win_prob, row['Cards'], row['Tens']))
                        
                except (ValueError, KeyError):
                    continue
                    
    except FileNotFoundError:
        print("File not found.")
        return

    print(f"\nFound {len(winners)} winning 1J 2A hands:")
    print("-" * 60)
    
    # Sort by probability descending
    winners.sort(key=lambda x: x[0], reverse=True)
    
    for prob, cards, tens in winners:
        print(f"Win: {prob*100:.1f}% | Tens: {tens} | {cards}")

if __name__ == "__main__":
    analyze()
