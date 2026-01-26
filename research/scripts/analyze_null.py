import pandas as pd
import numpy as np
import argparse
import os

# Null Rank Order: 7, 8, 9, 10, J, Q, K, A
NULL_RANKS = {'7': 0, '8': 1, '9': 2, 'T': 3, 'J': 4, 'Q': 5, 'K': 6, 'A': 7}
RANK_CHARS = "789TJQKA"

def parse_hand(hand_str):
    """
    Parses a hand string like '[CJ DJ SA ST ...]' into a dict of suits.
    Returns: {'C': [ranks...], 'S': [ranks...], ...} (ranks sorted ascending)
    """
    hand_str = hand_str.replace('[', '').replace(']', '')
    cards = hand_str.split()
    suits = {'C': [], 'S': [], 'H': [], 'D': []}
    
    for card in cards:
        if not card: continue
        suit = card[0]
        rank_char = card[1]
        if rank_char in NULL_RANKS:
            suits[suit].append(NULL_RANKS[rank_char])
            
    for s in suits:
        suits[s].sort()
        
    return suits

def is_suit_safe(held_ranks, skat_ranks=None):
    """
    Determines if a suit distribution is 'safe' in Null.
    Safe means for every opponent card, we have a lower card to play under it.
    Logic:
    1. Identify opponent ranks (missing from held_ranks AND skat_ranks).
    2. Sort opponent ranks ascending.
    3. For each opponent rank o_j (j=1..m), we must have at least j cards < o_j.
    """
    held_set = set(held_ranks)
    if skat_ranks is None:
        skat_ranks = []
    skat_set = set(skat_ranks)
    
    all_ranks = set(range(8))
    # Opponents have whatever we don't hold and isn't in Skat
    opponent_ranks = sorted(list(all_ranks - held_set - skat_set))
    
    # If no opponent cards, suit is safe
    if not opponent_ranks:
        return True, 0
    
    # If we have no cards but opponents do, it's safe (we discard)
    if not held_ranks:
        return True, 0

    unsafe_gaps = 0
    
    # Check condition
    # We must cover the i-th opponent card with our i-th card (or better).
    # Condition: H[i] < O[i] for all i < min(len(H), len(O))
    
    num_checks = min(len(held_ranks), len(opponent_ranks))
    
    for i in range(num_checks):
        if held_ranks[i] > opponent_ranks[i]:
            unsafe_gaps += 1
            
    is_safe = (unsafe_gaps == 0)
    return is_safe, unsafe_gaps

def analyze_null_safety(input_file):
    print(f"Loading {input_file}...")
    try:
        df = pd.read_csv(input_file)
    except Exception as e:
        print(f"Error: {e}")
        return

    # Clean headers
    df.columns = df.columns.str.strip()
    
    print(f"Total rows: {len(df)}")
    
    results = []
    
    for idx, row in df.iterrows():
        best_game = str(row.get('BestGame', '')).strip()
        if best_game != 'Null':
            continue

        hand_str = row['FinalHand'] # Post-discard
        prob_null = float(row.get('ProbNull', 0.0))
        skat_str = str(row.get('SkatCards', '[]')).strip()
        
        suits = parse_hand(hand_str)
        skat_suits = parse_hand(skat_str)
        
        safe_suits_count = 0
        total_uncovered = 0
        
        suit_gaps = {}
        
        for s in ['C', 'S', 'H', 'D']:
            # Pass Skat cards for this suit
            skat_ranks = skat_suits.get(s, [])
            safe, gaps = is_suit_safe(suits[s], skat_ranks)
            if safe:
                safe_suits_count += 1
            total_uncovered += gaps
            suit_gaps[s] = gaps
            
        # Singleton 8 Analysis & Void Analysis
        cnt_blank_8 = 0
        cnt_voids = 0
        for s in ['C', 'S', 'H', 'D']:
            if len(suits[s]) == 0:
                cnt_voids += 1
            if len(suits[s]) == 1 and suits[s][0] == 1:
                cnt_blank_8 += 1
                
        # Skat Cards
        skat_str = str(row.get('SkatCards', '[]')).strip()

        results.append({
            'Hand': hand_str,
            'Skat': skat_str,
            'BestGame': best_game,
            'ProbNull': prob_null,
            'SafeSuits': safe_suits_count,
            'VoidSuits': cnt_voids,
            'TotalGaps': total_uncovered,
            'Blank8s': cnt_blank_8,
            'Gaps_C': suit_gaps['C'],
            'Gaps_S': suit_gaps['S'],
            'Gaps_H': suit_gaps['H'],
            'Gaps_D': suit_gaps['D']
        })
        
    res_df = pd.DataFrame(results)
    
    # Save to CSV
    output_csv = os.path.join(os.path.dirname(input_file), "null_games_annotated.csv")
    res_df.to_csv(output_csv, index=False)
    print(f"Annotated Null games saved to {output_csv} (Rows: {len(res_df)})")
    
    # Prepare Report (on the filtered data)
    report_path = os.path.join(os.path.dirname(input_file), "null_analysis_report.txt")
    with open(report_path, "w") as f:
        f.write("--- Analysis of Null Games (BestGame = Null) ---\n\n")
        f.write(f"Total Null Games Found: {len(res_df)}\n\n")
        
        # Group by Safe Suits Count
        f.write("Win Probability by Number of Safe Suits:\n")
        f.write(str(res_df.groupby('SafeSuits')['ProbNull'].agg(['count', 'mean', 'std', 'min', 'max'])))
        f.write("\n\n")
        
        # Singleton 8 Analysis
        f.write("Win Probability by Number of 'Blank 8s' (Singleton 8) - All Hands:\n")
        f.write(str(res_df.groupby('Blank8s')['ProbNull'].agg(['count', 'mean', 'std', 'min', 'max'])))
        f.write("\n\n")

        # Conditional Analysis: 3 Safe Suits
        f.write("--- Conditional Analysis: Hands with EXACTLY 3 Safe Suits ---\n")
        safe3_df = res_df[res_df['SafeSuits'] == 3]
        f.write(f"Total Hands with 3 Safe Suits: {len(safe3_df)}\n")
        f.write("Win Probability by Number of 'Blank 8s' (given 3 Safe Suits):\n")
        f.write(str(safe3_df.groupby('Blank8s')['ProbNull'].agg(['count', 'mean', 'std', 'min', 'max'])))
        f.write("\n\n")

        # Conditional Analysis: 2 Safe Suits
        f.write("--- Conditional Analysis: Hands with EXACTLY 2 Safe Suits ---\n")
        safe2_df = res_df[res_df['SafeSuits'] == 2]
        f.write(f"Total Hands with 2 Safe Suits: {len(safe2_df)}\n")
        f.write("Win Probability by Number of 'Blank 8s' (given 2 Safe Suits):\n")
        f.write(str(safe2_df.groupby('Blank8s')['ProbNull'].agg(['count', 'mean', 'std', 'min', 'max'])))
        f.write("\n\n")

        # Conditional Analysis: 3 Safe Suits with 7-10 Gap
        # Logic: 
        # 1. Filter SafeSuits == 3
        # 2. Find the ONE unsafe suit.
        # 3. Check if Held has 7 (0) and 10 (3).
        # 4. Check if Skat/Held does NOT have 8 (1) or 9 (2).
        
        gap_7_10_hands = []
        
        for idx, row in safe3_df.iterrows():
            hand_str = row['Hand']
            skat_str = str(row.get('Skat', '[]')).strip()
            
            suits = parse_hand(hand_str)
            skat_suits = parse_hand(skat_str)
            
            # Identify unsafe suit
            unsafe_suit = None
            for s in ['C', 'S', 'H', 'D']:
                skat_ranks = skat_suits.get(s, [])
                safe, _ = is_suit_safe(suits[s], skat_ranks)
                if not safe:
                    unsafe_suit = s
                    break
            
            if unsafe_suit:
                held = set(suits[unsafe_suit])
                skat = set(skat_suits.get(unsafe_suit, []))
                
                # Check 7-10 Gap conditions
                # Held must have 7 (0) and 10 (3)
                has_7_10 = (0 in held) and (3 in held)
                
                # Opponents must have 8 (1) and 9 (2)
                # So Held+Skat must NOT have 1 or 2
                missing_8 = (1 not in held) and (1 not in skat)
                missing_9 = (2 not in held) and (2 not in skat)
                
                if has_7_10 and missing_8 and missing_9:
                     gap_7_10_hands.append(row)

        f.write("--- Conditional Analysis: 3 Safe Suits + Specific '7-10 Gap' in Unsafe Suit ---\n")
        if gap_7_10_hands:
            gap_df = pd.DataFrame(gap_7_10_hands)
            f.write(f"Total Hands with 7-10 Gap: {len(gap_df)}\n")
            f.write("Win Probability for 7-10 Gap Hands:\n")
            f.write(str(gap_df['ProbNull'].agg(['count', 'mean', 'std', 'min', 'max'])))
        else:
            f.write("No hands found with specific 7-10 Gap configuration (3 Safe Suits).\n")
        f.write("\n\n")

        # Conditional Analysis: 3 Safe Suits with 7-9-High (3 cards)
        # Patterns: 7-9-Q, 7-9-K, 7-9-A
        patterns_3card = {
            '7-9-Q (0,2,5)': {0, 2, 5},
            '7-9-K (0,2,6)': {0, 2, 6},
            '7-9-A (0,2,7)': {0, 2, 7}
        }
        
        # Patterns: 7-9-High-High (4 cards)
        # Patterns: 7-9-Q-K, 7-9-Q-A, 7-9-K-A
        patterns_4card = {
            '7-9-Q-K (0,2,5,6)': {0, 2, 5, 6},
            '7-9-Q-A (0,2,5,7)': {0, 2, 5, 7},
            '7-9-K-A (0,2,6,7)': {0, 2, 6, 7}
        }
        
        results_patterns = {k: [] for k in list(patterns_3card.keys()) + list(patterns_4card.keys())}

        for idx, row in safe3_df.iterrows():
            hand_str = row['Hand']
            suits = parse_hand(hand_str)
            # Find unsafe suit
            # Optimization: We know safe_suits_count=3, so exactly one unsafe suit.
            # But we don't have this pre-calculated in safe3_df row easily (except re-parsing).
            # Let's rely on re-check or just check suits length?
            # Re-check is safer.
            
            skat_str = str(row.get('Skat', '[]')).strip()
            skat_suits = parse_hand(skat_str)
            
            unsafe_suit = None
            for s in ['C', 'S', 'H', 'D']:
                skat_ranks = skat_suits.get(s, [])
                safe, _ = is_suit_safe(suits[s], skat_ranks)
                if not safe:
                    unsafe_suit = s
                    break
            
            if unsafe_suit:
                held_set = set(suits[unsafe_suit])
                
                # Check 3-card patterns
                if len(held_set) == 3:
                     for name, patt in patterns_3card.items():
                         if held_set == patt:
                             results_patterns[name].append(row['ProbNull'])
                             
                # Check 4-card patterns
                if len(held_set) == 4:
                     for name, patt in patterns_4card.items():
                         if held_set == patt:
                             results_patterns[name].append(row['ProbNull'])
                             
        f.write("--- Conditional Analysis: 3 Safe Suits + Specific 7-9-X Configurations ---\n")
        
        for name, probs in results_patterns.items():
            f.write(f"\nConfiguration: {name}\n")
            if probs:
                s = pd.Series(probs)
                f.write(str(s.agg(['count', 'mean', 'std', 'min', 'max'])))
            else:
                f.write("No matching hands found.")
        f.write("\n\n")

        # Frequency Analysis of Playable Null Games
        # Criteria: SafeSuits >= 3 AND ProbNull > 0.66
        f.write("--- Frequency Analysis: Playable Null Hands (SafeSuits >= 3, Win > 66%) ---\n")
        
        playable_df = res_df[(res_df['SafeSuits'] >= 3) & (res_df['ProbNull'] > 0.66)]
        f.write(f"Total Playable Hands: {len(playable_df)}\n\n")
        
        categories = []
        
        for idx, row in playable_df.iterrows():
            safe_suits = row['SafeSuits']
            if safe_suits == 4:
                categories.append("4 Safe Suits (Complete)")
            else:
                # SafeSuits == 3. Find the unsafe suit configuration.
                hand_str = row['Hand']
                suits = parse_hand(hand_str)
                skat_str = str(row.get('Skat', '[]')).strip()
                skat_suits = parse_hand(skat_str)
                
                unsafe_config = "Unknown"
                for s in ['C', 'S', 'H', 'D']:
                    skat_ranks = skat_suits.get(s, [])
                    safe, _ = is_suit_safe(suits[s], skat_ranks)
                    if not safe:
                        # Create a readable string of the held ranks in this unsafesuit
                        # Map ranks back to chars
                        # NULL_RANKS = {'7': 0, '8': 1, '9': 2, 'T': 3, 'J': 4, 'Q': 5, 'K': 6, 'A': 7}
                        # Invert map
                        rank_map = {v: k for k, v in NULL_RANKS.items()}
                        held_chars = [rank_map[r] for r in suits[s]]
                        unsafe_config = "3 Safe + [" + "".join(held_chars) + "]"
                        break
                categories.append(unsafe_config)
                
        # Count and Sort
        from collections import Counter
        counts = Counter(categories)
        
        f.write("Top 20 Most Frequent Configurations:\n")
        f.write(f"{'Count':<10} {'Freq %':<10} {'Configuration'}\n")
        f.write("-" * 50 + "\n")
        
        total = len(playable_df)
        for cat, count in counts.most_common(20):
            freq = (count / total) * 100
            f.write(f"{count:<10} {freq:<10.1f} {cat}\n")
        f.write("\n")

        # Frequency Analysis of Playable Null Games with 2 Safe Suits
        f.write("--- Frequency Analysis: Playable Null Hands (SafeSuits == 2, Win > 66%) ---\n")
        
        playable2_df = res_df[(res_df['SafeSuits'] == 2) & (res_df['ProbNull'] > 0.66)]
        f.write(f"Total Playable Hands (2 Safe Suits): {len(playable2_df)}\n\n")
        
        categories2 = []
        
        for idx, row in playable2_df.iterrows():
            hand_str = row['Hand']
            suits = parse_hand(hand_str)
            skat_str = str(row.get('Skat', '[]')).strip()
            skat_suits = parse_hand(skat_str)
            
            unsafe_configs = []
            
            # Map ranks back to chars
            rank_map = {v: k for k, v in NULL_RANKS.items()}
            
            for s in ['C', 'S', 'H', 'D']:
                skat_ranks = skat_suits.get(s, [])
                safe, _ = is_suit_safe(suits[s], skat_ranks)
                if not safe:
                    held_chars = [rank_map[r] for r in suits[s]]
                    config = "[" + "".join(held_chars) + "]"
                    unsafe_configs.append(config)
            
            # Should have exactly 2 unsafe configs
            if len(unsafe_configs) == 2:
                # Sort them so order doesn't matter ([8] + [9] is same as [9] + [8])
                unsafe_configs.sort()
                cat = "2 Safe + " + unsafe_configs[0] + " + " + unsafe_configs[1]
                categories2.append(cat)
            else:
                categories2.append(f"Error ({len(unsafe_configs)} unsafe)")
                
        # Count and Sort
        counts2 = Counter(categories2)
        
        f.write("Top 20 Most Frequent Configurations (2 Safe Suits):\n")
        f.write(f"{'Count':<10} {'Freq %':<10} {'Configuration'}\n")
        f.write("-" * 60 + "\n")
        
        total2 = len(playable2_df)
        if total2 > 0:
            for cat, count in counts2.most_common(20):
                freq = (count / total2) * 100
                f.write(f"{count:<10} {freq:<10.1f} {cat}\n")
        else:
            f.write("No playable hands found with exactly 2 Safe Suits.\n")
        f.write("\n")
            
    print(f"Report written to {report_path}")

    # Plotting
    try:
        import matplotlib.pyplot as plt
        import seaborn as sns
    except ImportError:
        print("Skipping plots (matplotlib/seaborn not installed)")
        return res_df

    plt.figure(figsize=(24, 6))
    
    # Plot 1: WinProb vs Safe Suits
    plt.subplot(1, 4, 1)
    sns.boxplot(x='SafeSuits', y='ProbNull', data=res_df, palette="viridis")
    plt.title('Win Prob vs # Safe Suits')
    
    # Plot 2: WinProb vs Unsafe Gaps
    # Bin large gaps
    res_df['GapsCapped'] = res_df['TotalGaps'].apply(lambda x: x if x <= 6 else 7)
    
    plt.subplot(1, 4, 2)
    sns.boxplot(x='GapsCapped', y='ProbNull', data=res_df, palette="magma_r")
    plt.title('Win Prob vs # Gaps')
    
    # Plot 3: WinProb vs Blank 8s (All)
    plt.subplot(1, 4, 3)
    sns.boxplot(x='Blank8s', y='ProbNull', data=res_df, palette="coolwarm")
    plt.title('Win Prob vs # Blank 8s (All)')

    # Plot 4: WinProb vs Blank 8s (3 Safe Suits Only)
    plt.subplot(1, 4, 4)
    if not safe3_df.empty:
        sns.boxplot(x='Blank8s', y='ProbNull', data=safe3_df, palette="coolwarm")
        plt.title('Win Prob vs # Blank 8s (3 Safe Suits Only)')
    
    plt.tight_layout()

    plot_path = os.path.join(os.path.dirname(input_file), "null_safety_analysis.png")
    plt.savefig(plot_path)
    print(f"Plot saved to {plot_path}")
    
    return res_df

if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument('--input', type=str, required=True)
    args = parser.parse_args()
    
    analyze_null_safety(args.input)
