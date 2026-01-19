import pandas as pd
import numpy as np

def parse_cards(card_str):
    # Parse card string like "[CJ SJ CA ...]" and return stats
    # Return: Max Suit Length (excluding Jacks)
    if not isinstance(card_str, str): return 0
    content = card_str.strip('[]')
    cards = content.split()
    
    # Counts by suit (excluding Jacks)
    # Suits: C, S, H, D
    # Jacks: CJ, SJ, HJ, DJ -> Treat as Trumps (not suit)
    
    suits = {'C': 0, 'S': 0, 'H': 0, 'D': 0}
    jacks = 0
    
    for c in cards:
        if 'J' in c:
            jacks += 1
        else:
            suit = c[1] # e.g. 'A' of 'C' -> 'CA' ? No, 'CA' is second char?
            # Rust Output format: CJ, C9, etc.
            # Wait, Rust output is usually Value then Suit or Suit then Value?
            # Let's check a sample: "CA" -> Clubs Ace? or Ace Clubs?
            # Standard German: "CA" usually Clubs Ace.
            # Checking CSV: "CA CQ C8" -> All start with C. So C=Clubs.
            # "SJ DJ" -> Spades Jack, Diamonds Jack.
            # "S8" -> Spades 8.
            # So First char is Suit. Second is Value.
            # J is special.
            
            s_char = c[0]
            if s_char in suits:
                suits[s_char] += 1
                
    if not suits: return 0
    return max(suits.values())

def analyze_thresholds():
    # 1. Load Data
    suit_path = "../data/suit_large_10k.csv"
    grand_path = "../data/grand_post_discard_10k.csv"
    
    try:
        df_suit = pd.read_csv(suit_path)
        df_grand = pd.read_csv(grand_path)
    except Exception as e:
        print(f"Error loading data: {e}")
        return

    # 2. Suit Analysis (Post Discard)
    # Filter for Trump Counts 5, 6, 7
    # Group by (NumJacks, Fulls). WinProb > 0.70
    
    # Helper to get Jacks Count from Mask
    def get_jacks_count(mask):
        mask = str(mask).strip()
        return 0 if mask in ['-', 'nan', ''] else len(mask)

    df_suit['NumJacks'] = df_suit['PostJacksMask'].apply(get_jacks_count)
    df_suit['Fulls'] = df_suit['PostAces'] + df_suit['PostAttachedTens'] + df_suit.get('PostSkatFulls', 0)
    
    print("\n# Suit Game Thresholds (>70% Win Probability)")
    print("Metrics: Side Fulls (Ass + Stehende 10 in Nebenfarben)")
    
    for trumps in [5, 6, 7]:
        subset = df_suit[df_suit['PostTrumpCount'] == trumps]
        if subset.empty: continue
        
        # pivot: Index=NumJacks, Col=Fulls, Val=WinProb
        pivot = subset.groupby(['NumJacks', 'Fulls'])['WinProb'].mean().unstack()
        
        # Filter for > 0.70
        print(f"\n## {trumps} Trümpfe (Post-Discard)")
        print("| Jacks | Fulls Needed for >70% | Avg Win % |")
        print("|---|---|---|")
        
        for jacks in sorted(pivot.index):
            row = pivot.loc[jacks]
            # Find first column > 0.70
            found = False
            for fulls in sorted(row.index):
                prob = row[fulls]
                if prob >= 0.70:
                    print(f"| {jacks} | {fulls}+ | {prob*100:.1f}% |")
                    found = True
                    break
            if not found:
                 print(f"| {jacks} | - | (Max: {row.max()*100:.1f}%) |")

    # 3. Grand Analysis
    df_grand['NumJacks'] = df_grand['JacksMask'].apply(get_jacks_count)
    df_grand['Fulls'] = df_grand['Aces'] + df_grand['AttachedTens'] + df_grand.get('SkatFulls', 0)
    
    # Calculate Max Suit Length
    df_grand['MaxSuit'] = df_grand['Cards'].apply(parse_cards)
    
    print("\n# Grand Game Thresholds (>70%)")
    print("Metrics: Total Fulls (Alle Asse + Stehende 10)")
    
    pivot_grand = df_grand.groupby(['NumJacks', 'Fulls'])['WinProb'].mean().unstack()
    
    print("\n## Grand (General)")
    print("| Jacks | Fulls Needed | Avg Win % |")
    print("|---|---|---|")
    for jacks in sorted(pivot_grand.index):
        row = pivot_grand.loc[jacks]
        found = False
        for fulls in sorted(row.index):
            prob = row[fulls]
            if prob >= 0.70:
                print(f"| {jacks} | {fulls}+ | {prob*100:.1f}% |")
                found = True
                break
        if not found:
             print(f"| {jacks} | - | (Max: {row.max()*100:.1f}%) |")

    # 4. Comparison Window
    # When to play Suit 5-Trump over Grand?
    # Filter Grand for "Hands that look like 5-Trump Suits" (MaxSuit + Jacks = 5 approx? Or MaxSuit = 4, Jacks=1?)
    # Suit 5 Trumps usually = Jacks + Suit.
    # Case: 1 Jack.
    # Suit: 1 Jack + 4 Trumps -> 5 Trumps.
    # Grand Equivalence: 1 Jack + 4-card Suit.
    
    print("\n# Decision Support: 5-Trump Suit vs Grand")
    print("Comparing: Suit (5 Trumps) vs Grand (with matching Suit Length)")
    
    # Compare for 1, 2, 3 Jacks
    comparison_data = []
    
    for jacks in [1, 2, 3, 4]:
        # Suit 5 Trumps (Fixed)
        suit_row = df_suit[(df_suit['PostTrumpCount'] == 5) & (df_suit['NumJacks'] == jacks)]
        if suit_row.empty: continue
        suit_prob = suit_row['WinProb'].mean()
        
        # Grand with Equivalent Suit
        # Equivalent Suit Length = 5 - Jacks
        req_suit_len = 5 - jacks
        if req_suit_len < 0: continue # Impossible to have 5 trumps with 6 jacks...
        
        grand_row = df_grand[(df_grand['NumJacks'] == jacks) & (df_grand['MaxSuit'] >= req_suit_len)]
        if grand_row.empty: continue
        grand_prob = grand_row['WinProb'].mean()
        
        comparison_data.append({
            'Jacks': jacks,
            'Suit (5T) Win%': suit_prob,
            'Grand (Sim) Win%': grand_prob,
            'Diff': suit_prob - grand_prob
        })
        
    print("\n## Base Comparison (Suit=5 Trumps vs Grand with ~4 card suit)")
    res_df = pd.DataFrame(comparison_data)
    if not res_df.empty:
        print(res_df.to_string(index=False, float_format="%.1f%%"))

    # Comparison for 4 Trumps (User Request)
    print("\n## Base Comparison (Suit=4 Trumps vs Grand with ~4 card suit)")
    comp_4t = []
    
    for jacks in [0, 1, 2, 3]: # 4 trumps possible with 0-4 Jacks.
        # Suit 4 Trumps
        suit_row = df_suit[(df_suit['PostTrumpCount'] == 4) & (df_suit['NumJacks'] == jacks)]
        if suit_row.empty: continue
        suit_prob = suit_row['WinProb'].mean()
        
        # Grand with Equivalent Suit
        # If I have 4 Trumps, I have a 4-card suit.
        # Grand Equivalent: Jacks = J, MaxSuit >= 4.
        
        grand_row = df_grand[(df_grand['NumJacks'] == jacks) & (df_grand['MaxSuit'] >= 4)]
        if grand_row.empty: continue
        grand_prob = grand_row['WinProb'].mean()
        
        comp_4t.append({
            'Jacks': jacks,
            'Suit (4T) Win%': suit_prob,
            'Grand (Sim) Win%': grand_prob,
            'Diff': suit_prob - grand_prob,
            'Recommendation': "Suit" if suit_prob > grand_prob + 0.05 else ("Grand" if grand_prob > suit_prob + 0.05 else "Neutral")
        })

    res_4t = pd.DataFrame(comp_4t)
    if not res_4t.empty:
        print(res_4t.to_string(index=False, float_format="%.1f%%"))

    # Detailed Comparison: Impact of Fulls
    # "If I have 1 Jack and X Fulls..."
    print("\n## Detailed Decision (1 Jack, 5-Trump Structure)")
    j1_suit = df_suit[(df_suit['PostTrumpCount'] == 5) & (df_suit['NumJacks'] == 1)]
    j1_grand = df_grand[(df_grand['NumJacks'] == 1) & (df_grand['MaxSuit'] >= 4)]
    
    if not j1_suit.empty and not j1_grand.empty:
        # Group by fulls
        s_grp = j1_suit.groupby('Fulls')['WinProb'].mean()
        g_grp = j1_grand.groupby('Fulls')['WinProb'].mean()
        
        print("| Fulls (Side/Total) | Suit (5T) Win% | Grand (Long Suit) Win% | Preference |")
        print("|---|---|---|---|")
        # Note: Suit Fulls = Side. Grand Fulls = Total.
        # Approx: Grand Fulls = Side Fulls + (0.25 * Trump Count?)
        # Let's align by Side Fulls simply (Assuming Trump Ace is implicitly captured in WinProb of Suit).
        # We compare "I have X Side Aces".
        # For Grand, if I have X Side Aces, I have X Total Aces (mostly, unless Trump Ace exists).
        # If I have Trump Ace -> Suit is stronger? Grand is stronger?
        # Let's just compare raw "Fulls" count.
        
        idx = sorted(list(set(s_grp.index) | set(g_grp.index)))
        for i in idx:
            s_p = s_grp.get(i, 0)
            g_p = g_grp.get(i, 0)
            pref = "Suit" if s_p > g_p + 0.05 else ("Grand" if g_p > s_p + 0.05 else "Neutral")
            if s_p > 0 or g_p > 0:
                print(f"| {i} | {s_p:.1%} | {g_p:.1%} | {pref} |")

    # 5. Grand Pre-Discard (Pickup) Analysis
    print("\n# Grand Pre-Discard Thresholds (Pickup)")
    pickup_path = "../data/grand_pickup_1000.csv"
    try:
        df_pickup = pd.read_csv(pickup_path)
        df_pickup['NumJacks'] = df_pickup['JacksMask'].apply(get_jacks_count)
        df_pickup['Fulls'] = df_pickup['Aces'] + df_pickup['AttachedTens']
        
        pivot_pre = df_pickup.groupby(['NumJacks', 'Fulls'])['WinProb'].mean().unstack()
        print("| Jacks | Fulls Needed (>70%) | Avg Win % |")
        print("|---|---|---|")
        for jacks in sorted(pivot_pre.index):
            row = pivot_pre.loc[jacks]
            found = False
            for fulls in sorted(row.index):
                prob = row[fulls]
                if prob >= 0.70:
                    print(f"| {jacks} | {fulls}+ | {prob*100:.1f}% |")
                    found = True
                    break
            if not found:
                 print(f"| {jacks} | - | (Max: {row.max()*100:.1f}%) |")

    except Exception as e:
        print(f"Could not load Grand Pickup data: {e}")

if __name__ == "__main__":
    analyze_thresholds()
