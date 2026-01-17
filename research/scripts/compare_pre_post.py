import pandas as pd
import sys

def get_hand_stats(cards_str):
    # Remove brackets if present
    cards_str = cards_str.replace('[', '').replace(']', '')
    cards = cards_str.split(' ')
    jacks = 0
    suits = {'C': 0, 'S': 0, 'H': 0, 'D': 0}
    
    for c in cards:
        if not c: continue
        if 'J' in c:
            jacks += 1
        else:
            s = c[0] # First char is suit
            if s in suits:
                suits[s] += 1
    
    max_suit = max(suits.values()) if suits else 0
    return jacks, max_suit

def analyze_file(filename, label):
    print(f"Analyzing {label} ({filename})...")
    df = pd.read_csv(filename)
    
    # Apply stats parsing
    stats = df['Cards'].apply(get_hand_stats)
    df['JacksCount'] = stats.apply(lambda x: x[0])
    df['MaxSuitLen'] = stats.apply(lambda x: x[1])
    
    # Filter for WinProb >= 0.75
    # actually we want to find *conditions* that lead to >= 0.75 mean win rate.
    
    # Group by Jacks, Aces, MaxSuitLen
    grouped = df.groupby(['JacksCount', 'Aces', 'MaxSuitLen'])['WinProb'].agg(['count', 'mean'])
    
    # Filter groups with sufficient sample size
    valid = grouped[grouped['count'] >= 5].copy()
    
    # Mark if they meet the 75% threshold
    valid['IsWin'] = valid['mean'] >= 0.75
    
    print(f"\n--- {label} Requirements (>= 75% Win) ---")
    
    for j in range(1, 5):
        print(f"\nJacks: {j}")
        subset = valid.loc[j] if j in valid.index.get_level_values(0) else None
        if subset is None:
            print("  No data")
            continue
            
        winners = subset[subset['IsWin']]
        if winners.empty:
            print("  No combination found > 75%")
            continue
            
        # For each Ace count, find the minimum Suit Length required
        for a in sorted(winners.index.get_level_values(0).unique()):
            ace_subset = winners.loc[a]
            min_suit = ace_subset.index.min()
            win_rate = ace_subset.loc[min_suit]['mean']
            print(f"  {a} Aces + Suit Len {min_suit}+ ({win_rate*100:.1f}%)")

if __name__ == "__main__":
    analyze_file("grand_pickup_1000.csv", "PRE-PICKUP")
    analyze_file("grand_post_discard_10k.csv", "POST-DISCARD")
