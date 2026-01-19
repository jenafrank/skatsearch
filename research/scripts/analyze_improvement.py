import pandas as pd
import sys

def analyze():
    csv_file = "../data/suit_large_10k.csv"
    try:
        df = pd.read_csv(csv_file)
    except:
        print(f"Could not read {csv_file}")
        return

    print(f"Analyzing {len(df)} games...")

    # Metrics
    improved_trump = 0
    improved_jacks = 0
    improved_aces = 0
    
    # Win Prob Categories
    # Note: We only have Post-WinProb. We can't compare WinProb boost directly.
    # But we can assume Pre-Discard WinProb is closely correlated with Pre-Structure.
    # We will analyze structure shift.

    for _, row in df.iterrows():
        # Pre
        jacks_mask = str(row['JacksMask']).strip()
        pre_jacks = 0 if jacks_mask in ["-", ""] else len(jacks_mask)
        pre_trump = row['TrumpCount']
        pre_aces = row['Aces']

        # Post
        post_mask = str(row['PostJacksMask']).strip()
        post_jacks = 0 if post_mask in ["-", ""] else len(post_mask)
        post_trump = row['PostTrumpCount']
        post_aces = row['PostAces']

        if post_trump > pre_trump:
            improved_trump += 1
        
        if post_jacks > pre_jacks:
            improved_jacks += 1
            
        if post_aces > pre_aces:
            improved_aces += 1

    print("\n--- Structural Improvement ---")
    print(f"Trump Count Increase: {improved_trump/len(df)*100:.1f}% of games")
    print(f"Jack Count Increase:  {improved_jacks/len(df)*100:.1f}% of games")
    print(f"Ace Count Increase:   {improved_aces/len(df)*100:.1f}% of games")

    # Deep Dive: 1 Jack Hands
    # Filter for Pre-Hand = 1 Jack
    # Count how often they become 2 Jacks
    
    j1_hands = df[df['JacksMask'].astype(str).apply(lambda x: len(x) if x not in ['-', 'nan'] else 0) == 1]
    cnt_j1 = len(j1_hands)
    if cnt_j1 > 0:
        j1_to_j2 = j1_hands[j1_hands['PostJacksMask'].astype(str).apply(lambda x: len(x) if x not in ['-', 'nan'] else 0) >= 2]
        print(f"\n--- 1 Jack Analysis (N={cnt_j1}) ---")
        print(f"Becomes 2+ Jacks: {len(j1_to_j2)/cnt_j1*100:.1f}%")

    # Trump 4 -> 5
    t4_hands = df[df['TrumpCount'] == 4]
    if len(t4_hands) > 0:
        t4_to_5 = t4_hands[t4_hands['PostTrumpCount'] >= 5]
        print(f"\n--- 4 Trump Analysis (N={len(t4_hands)}) ---")
        print(f"Becomes 5+ Trumps: {len(t4_to_5)/len(t4_hands)*100:.1f}%")

if __name__ == "__main__":
    analyze()
