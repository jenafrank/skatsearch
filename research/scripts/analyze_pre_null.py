
import pandas as pd
import numpy as np
import argparse
import os
import matplotlib.pyplot as plt
import seaborn as sns

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

def analyze_pre_null(input_file):
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
        # Focus on InitHand analysis
        hand_str = str(row.get('InitHand', '')).strip()
        prob_null = float(row.get('ProbNull', 0.0))
        
        # Parse InitHand (10 cards)
        suits = parse_hand(hand_str)
        
        # Calculate Initial Gaps WITHOUT knowledge of Skat
        # We assume Skat cards are part of the 'unknown' pool (effectively opponent cards for safety check)
        initial_total_gaps = 0
        
        for s in ['C', 'S', 'H', 'D']:
            # skat_ranks=[] because we don't know them yet
            safe, gaps = is_suit_safe(suits[s], skat_ranks=[])
            initial_total_gaps += gaps
            
        results.append({
            'InitHand': hand_str,
            'Initial_TotalGaps': initial_total_gaps,
            'ProbNull': prob_null
        })
        
    res_df = pd.DataFrame(results)
    
    # Bucket Gaps
    # Buckets: 0, 1, 2, 3, 4, 5, 6+
    def bucket_gaps(g):
        if g == 0: return '0 (Perfect)'
        if g == 1: return '1'
        if g == 2: return '2'
        if g == 3: return '3'
        if g == 4: return '4'
        if g == 5: return '5'
        return '6+'
    
    res_df['GapBucket'] = res_df['Initial_TotalGaps'].apply(bucket_gaps)
    
    # Report Generation
    report_path = os.path.join(os.path.dirname(input_file), "pre_null_analysis.txt")
    with open(report_path, "w") as f:
        f.write("--- Pre-Discard Null Game Analysis ---\n")
        f.write(f"Source: {input_file}\n")
        f.write(f"Total Hands Analyzed: {len(res_df)}\n\n")
        
        f.write("Analysis Logic:\n")
        f.write("- Initial Gaps calculated on strictly the 10-card InitHand.\n")
        f.write("- Skat cards are treated as unknown (potentially opponent cards).\n")
        f.write("- Gaps = Number of held cards that fail coverage check.\n")
        f.write("- 0 Gaps: 2 free discards for Skat.\n")
        f.write("- 1 Gap: 1 forced discard, 1 free discard.\n")
        f.write("- 2 Gaps: 2 forced discards, must keep Skat.\n\n")
        
        # Group by Bucket
        stats = res_df.groupby('GapBucket')['ProbNull'].agg(['count', 'mean', 'std'])
        stats['Pct_Playable_90'] = res_df.groupby('GapBucket')['ProbNull'].apply(lambda x: (x > 0.9).mean() * 100)
        stats['Pct_Playable_66'] = res_df.groupby('GapBucket')['ProbNull'].apply(lambda x: (x > 0.66).mean() * 100)
        
        # Reorder index
        logical_order = ['0 (Perfect)', '1', '2', '3', '4', '5', '6+']
        # Filter for existing buckets in case some are empty
        logical_order = [x for x in logical_order if x in stats.index]
        stats = stats.reindex(logical_order)
        
        f.write("Stats by Initial Gap Bucket:\n")
        f.write(stats.to_string())
        f.write("\n\n")
        
    print(f"Report written to {report_path}")

    # Specific Gap Analysis
    # We want to know: For '1 Gap' hands, what are the held cards in the gap suit?
    # For '2 Gaps' hands, what are the configs?
    
    # Helper to stringify held ranks
    def hand_to_str(ranks):
        # NULL_RANKS = {'7': 0, '8': 1, '9': 2, 'T': 3, 'J': 4, 'Q': 5, 'K': 6, 'A': 7}
        rev_map = {v: k for k, v in NULL_RANKS.items()}
        return "".join([rev_map[r] for r in ranks])

    with open(report_path, "a") as f:
        for target_gap in [1, 2]:
            f.write(f"--- Detailed Analysis: Hands with Exactly {target_gap} Gap(s) ---\n")
            subset = res_df[res_df['Initial_TotalGaps'] == target_gap]
            
            if len(subset) == 0:
                f.write("No hands found.\n\n")
                continue
                
            # For each hand in this subset, identify the suit(s) with gaps and their configuration
            gap_configs = []
            
            for idx, row in subset.iterrows():
                hand_str = row['InitHand']
                suits = parse_hand(hand_str)
                prob = row['ProbNull']
                
                # Find the suit with gaps
                # Note: multiple suits could have gaps if target_gap > 1
                current_hand_configs = []
                
                for s in ['C', 'S', 'H', 'D']:
                    safe, gaps = is_suit_safe(suits[s], skat_ranks=[])
                    if gaps > 0:
                        # Append the config
                        current_hand_configs.append(hand_to_str(suits[s]))
                
                # Sort configs to normalize (e.g. C=[8], S=[7,T] vs S=[8], C=[7,T])
                current_hand_configs.sort()
                config_signature = " + ".join(current_hand_configs)
                
                gap_configs.append({
                    'Config': config_signature,
                    'ProbNull': prob
                })
            
            # Create DataFrame for aggregation
            config_df = pd.DataFrame(gap_configs)
            stats_cfg = config_df.groupby('Config')['ProbNull'].agg(['count', 'mean', 'std'])
            stats_cfg['Playable%'] = config_df.groupby('Config')['ProbNull'].apply(lambda x: (x > 0.66).mean() * 100)
            
            # Sort by Count desc
            stats_cfg = stats_cfg.sort_values('count', ascending=False)
            
            f.write(stats_cfg.to_string())
            f.write("\n\n")
    
    # Plotting
    try:
        plt.figure(figsize=(10, 6))
        sns.boxplot(x='GapBucket', y='ProbNull', data=res_df, order=logical_order, palette="coolwarm")
        plt.title('Win Probability vs Initial Null Gaps (Pre-Discard)')
        plt.xlabel('Initial Force-Gaps (10 cards)')
        plt.ylabel('Win Probability (after optimal discard)')
        
        plot_path = os.path.join(os.path.dirname(input_file), "pre_null_gaps_vs_prob.png")
        plt.savefig(plot_path)
        print(f"Plot saved to {plot_path}")
        
    except ImportError:
        print("Skipping plot (libraries missing)")

if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument('--input', type=str, required=True)
    args = parser.parse_args()
    
    analyze_pre_null(args.input)
