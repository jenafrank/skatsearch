import csv
import collections
import sys

def parse_jacks_mask(val):
    # If it's a digit, return as is
    if val.isdigit():
        return int(val)
    
    # If it's a string like "CS", convert to int mask
    mask = 0
    if 'C' in val: mask |= 8
    if 'S' in val: mask |= 4
    if 'H' in val: mask |= 2
    if 'D' in val: mask |= 1
    return mask

def get_signature_key(row):
    mask = parse_jacks_mask(row['JacksMask'])
    return f"{mask}-{row['Aces']}-{row['Tens']}-{row['AttachedTens']}-{row['TenKingSmall']}"

def load_baseline(filename):
    print(f"Loading baseline from {filename}...")
    baseline = collections.defaultdict(list)
    try:
        with open(filename, 'r') as f:
            reader = csv.DictReader(f)
            for row in reader:
                key = get_signature_key(row)
                baseline[key].append(float(row['WinProb']))
    except FileNotFoundError:
        print("Baseline file not found.")
        return {}
        
    # Average the probabilities
    return {k: sum(v)/len(v) for k, v in baseline.items()}

def compare():
    baseline_file = 'big_run.csv'
    target_file = 'grand_pickup.csv'
    
    if len(sys.argv) > 1:
        target_file = sys.argv[1]
        
    baseline_map = load_baseline(baseline_file)
    if not baseline_map:
        return

    print(f"Comparing {target_file} against baseline...")
    
    improvements = []
    winning_pickup = 0
    total = 0
    
    try:
        with open(target_file, 'r') as f:
            reader = csv.DictReader(f)
            for row in reader:
                total += 1
                key = get_signature_key(row)
                pickup_prob = float(row['WinProb'])
                
                if pickup_prob > 0.5:
                    winning_pickup += 1
                
                if key in baseline_map:
                    hand_prob = baseline_map[key]
                    improvements.append(pickup_prob - hand_prob)
                else:
                    # Signature not seen in baseline (rare?)
                    pass

    except FileNotFoundError:
        print(f"Target file {target_file} not found.")
        return

    print(f"\nTotal Hands in Pickup Simulation: {total}")
    print(f"Winning Hands (>50%): {winning_pickup} ({winning_pickup/total*100:.2f}%)")
    
    if improvements:
        avg_imp = sum(improvements) / len(improvements)
        print(f"Average Improvement over Grand Hand: +{avg_imp*100:.2f}%")
        
    print("\n--- NEW Viable Hands (Lost before, Win now) ---")
    # Group by simple signature metrics
    new_winners = collections.defaultdict(list)
    
    try:
        with open(target_file, 'r') as f:
            reader = csv.DictReader(f)
            for row in reader:
                key = get_signature_key(row)
                pickup_prob = float(row['WinProb'])
                
                # Check baseline prob
                baseline_prob = 0.0
                if key in baseline_map:
                    baseline_prob = baseline_map[key]
                
                # Identify "Flippers": Bad (<40%) -> Good (>50%)
                if baseline_prob < 0.4 and pickup_prob > 0.5:
                    sig_desc = f"Jacks:{bin(int(row['JacksMask'])).count('1')} Aces:{row['Aces']} Tens:{row['Tens']}"
                    new_winners[sig_desc].append(pickup_prob)
                    
    except Exception:
        pass

    for sig, probs in sorted(new_winners.items()):
        print(f"{sig} -> Avg Win {sum(probs)/len(probs):.2f} (Count: {len(probs)})")
        
    print("\n--- Tens Analysis (Discard Potential) ---")
    # Do hands with many tens improve MORE?
    tens_improvement = collections.defaultdict(list)
    with open(target_file, 'r') as f:
        reader = csv.DictReader(f)
        for row in reader:
            key = get_signature_key(row)
            if key in baseline_map:
                diff = float(row['WinProb']) - baseline_map[key]
                tens = int(row['Tens'])
                tens_improvement[tens].append(diff)
                
    for t in sorted(tens_improvement.keys()):
        avg = sum(tens_improvement[t])/len(tens_improvement[t])
        print(f"Tens: {t} -> Avg Improvement: +{avg*100:.2f}%")

if __name__ == "__main__":
    compare()
