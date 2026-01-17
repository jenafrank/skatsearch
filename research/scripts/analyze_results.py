import sys
import csv
import collections

def analyze():
    filename = 'big_run.csv'
    if len(sys.argv) > 1:
        filename = sys.argv[1]
        
    print(f"Analyzing {filename}...")
    
    total_hands = 0
    winning_hands = 0
    
    # Store hands with WinProb > 0.8
    strong_hands = []
    
    # Store stats for specific categories
    four_jacks_stats = []
    club_jack_only_stats = []
    
    # Specific Analysis: 3 Aces
    j1_a3_stats = []
    j2_a3_stats = []
    j2_spade_a3_stats = []
    
    try:
        with open('big_run.csv', 'r') as f:
            reader = csv.DictReader(f)
            for row in reader:
                total_hands += 1
                win_prob = float(row['WinProb'])
                
                # JacksMask: 1=D, 2=H, 4=S, 8=C
                jacks = int(row['JacksMask'])
                aces = int(row['Aces'])
                tens = int(row['Tens'])
                
                if win_prob > 0.5:
                    winning_hands += 1
                
                if win_prob > 0.8:
                    strong_hands.append(row)
                    
                # 4 Jacks (Mask 15 = 8+4+2+1)
                if jacks == 15:
                    four_jacks_stats.append(win_prob)
                    
                # Club Jack only (Mask 8)
                if jacks == 8:
                    club_jack_only_stats.append(win_prob)

    except FileNotFoundError:
        print("Error: big_run.csv not found.")
        return

    print(f"Total Hands Analyzed: {total_hands}")
    print(f"Grand Hand Frequency (Winning > 50%): {winning_hands/total_hands*100:.2f}%")
    
    # Specific Analysis: Tens and Combinations
    tens_stats = collections.defaultdict(list)
    protected_tens_stats = []
    unprotected_tens_stats = []
    ten_king_small_stats = collections.defaultdict(list)

    try:
        with open('big_run.csv', 'r') as f:
            reader = csv.DictReader(f)
            for row in reader:
                win_prob = float(row['WinProb'])
                tens = int(row['Tens'])
                attached = int(row['AttachedTens'])
                tks = int(row['TenKingSmall'])
                
                tens_stats[tens].append(win_prob)
                ten_king_small_stats[tks].append(win_prob)
                
                if tens > 0:
                    # All tens protected
                    if attached == tens:
                        protected_tens_stats.append(win_prob)
                    # All tens unprotected (none attached to Ace)
                    elif attached == 0:
                        unprotected_tens_stats.append(win_prob)

    except FileNotFoundError:
        pass
        
    print("\n--- Tens Analysis ---")
    for t in sorted(tens_stats.keys()):
        if tens_stats[t]:
             print(f"Count of Tens: {t} -> Avg Win Prob {sum(tens_stats[t])/len(tens_stats[t]):.4f} (n={len(tens_stats[t])})")
    
    if protected_tens_stats:
        print(f"All Tens Protected (with Ace): Avg Win Prob {sum(protected_tens_stats)/len(protected_tens_stats):.4f} (n={len(protected_tens_stats)})")
    if unprotected_tens_stats:
        print(f"All Tens UNPROTECTED (no Ace): Avg Win Prob {sum(unprotected_tens_stats)/len(unprotected_tens_stats):.4f} (n={len(unprotected_tens_stats)})")

    print("\n--- 10-King-Small Analysis ---")
    for c in sorted(ten_king_small_stats.keys()):
         if ten_king_small_stats[c]:
             print(f"Count of 10-K-Small Suits: {c} -> Avg Win Prob {sum(ten_king_small_stats[c])/len(ten_king_small_stats[c]):.4f} (n={len(ten_king_small_stats[c])})")

    print("\n--- 3 Aces Analysis ---")
    if j1_a3_stats:
        print(f"1 Jack + 3 Aces: Avg Win Prob {sum(j1_a3_stats)/len(j1_a3_stats):.2f} (n={len(j1_a3_stats)})")
    if j2_a3_stats:
        print(f"2 Jacks + 3 Aces: Avg Win Prob {sum(j2_a3_stats)/len(j2_a3_stats):.2f} (n={len(j2_a3_stats)})")
    if j2_spade_a3_stats:
        print(f"  -> whereof One is Spade Jack: Avg Win Prob {sum(j2_spade_a3_stats)/len(j2_spade_a3_stats):.2f} (n={len(j2_spade_a3_stats)})")

    if four_jacks_stats:
        avg_4j = sum(four_jacks_stats) / len(four_jacks_stats)
        print(f"\n4 Jacks Average Win Prob: {avg_4j:.4f} (Count: {len(four_jacks_stats)})")
        # Filter 4 Jacks with 0 Aces
        four_jacks_no_aces = [p for i, p in enumerate(four_jacks_stats) if int(strong_hands[i]['Aces']) == 0] if len(strong_hands) > 0 else [] 
        # Note: logic above is flawed b/c not aligned. Let's rely on averages for now or iterate again if needed.
    
    if club_jack_only_stats:
        avg_cj = sum(club_jack_only_stats) / len(club_jack_only_stats)
        print(f"Club Jack Only Average Win Prob: {avg_cj:.4f} (Count: {len(club_jack_only_stats)})")

    print("\n--- Top Winning Signatures (Sample of >99%) ---")
    for hand in strong_hands[:10]: # Just first 10
        if float(hand['WinProb']) > 0.99:
           print(f"Jacks: {hand['JacksMask']}, Aces: {hand['Aces']}, Tens: {hand['Tens']} -> {hand['WinProb']}")

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
    pass
    # Aggregation: Group by Jacks + Aces
    print("\n--- Win Rate by Jacks + Aces Count ---")
    combo_stats = collections.defaultdict(list)
    for row in strong_hands: # Using strong hands to see what 'works'
        sig = f"J{row['JacksMask']}-A{row['Aces']}"
        combo_stats[sig].append(float(row['WinProb']))
    
    # Sort by consistency
    sorted_combos = sorted(combo_stats.items(), key=lambda x: len(x[1]), reverse=True)
    for sig, probs in sorted_combos[:10]:
         print(f"{sig}: Avg {sum(probs)/len(probs):.2f} (Count: {len(probs)})")

if __name__ == "__main__":
    analyze()
