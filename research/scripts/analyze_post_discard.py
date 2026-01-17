import pandas as pd
import sys
import matplotlib.pyplot as plt

def parse_jacks_mask(mask_str):
    if isinstance(mask_str, int):
        return mask_str
    
    val = 0
    if 'C' in mask_str: val |= 8
    if 'S' in mask_str: val |= 4
    if 'H' in mask_str: val |= 2
    if 'D' in mask_str: val |= 1
    return val

def analyze(file_path):
    print(f"Analyzing {file_path}...")
    try:
        df = pd.read_csv(file_path)
    except Exception as e:
        print(f"Error reading CSV: {e}")
        return

    # Filter for winners > 50%
    winners = df[df['WinProb'] > 0.5]
    
    print(f"Total Hands: {len(df)}")
    print(f"Winning Hands (>50%): {len(winners)} ({len(winners)/len(df)*100:.1f}%)")
    
    # Analyze Skat Fulls impact
    print("\n--- Impact of 'Full Cards' (Aces/Tens) in Skat ---")
    
    # Group by SkatFulls
    grouped = df.groupby('SkatFulls')['WinProb'].agg(['count', 'mean', 'std'])
    print(grouped)
    
    # Analyze interaction between Hand Tens and Skat Fulls for winning hands
    print("\n--- Winners: Hand Tens vs Skat Fulls ---")
    winners_grouped = winners.groupby(['Tens', 'SkatFulls'])['WinProb'].count().unstack().fillna(0)
    print(winners_grouped)
    
    # Calculate Stats
    print("\nProcessing card distributions...")
    
    # Helper to parse cards
    def get_hand_stats(cards_str):
        cards = cards_str.split(' ')
        jacks = 0
        suits = {'C': 0, 'S': 0, 'H': 0, 'D': 0}
        
        for c in cards:
            if 'J' in c:
                jacks += 1
            else:
                s = c[0] # First char is suit (C, S, H, D)
                if s in suits:
                    suits[s] += 1
        
        max_suit = max(suits.values()) if suits else 0
        return jacks, max_suit

    # Apply to dataframe
    stats = df['Cards'].apply(get_hand_stats)
    df['JacksCount'] = stats.apply(lambda x: x[0])
    df['MaxSuitLen'] = stats.apply(lambda x: x[1])
    
    # 1. Jacks Count Analysis
    print("\n--- Win Rate by Number of Jacks ---")
    print(df.groupby('JacksCount')['WinProb'].agg(['count', 'mean']))

    # 2. Aces Count Analysis
    print("\n--- Win Rate by Number of Aces ---")
    print(df.groupby('Aces')['WinProb'].agg(['count', 'mean']))

    # 3. Longest Suit Analysis
    print("\n--- Win Rate by Longest Side Suit ---")
    print(df.groupby('MaxSuitLen')['WinProb'].agg(['count', 'mean']))

    # 4. Combined Analysis (The "Recipe")
    print("\n--- The Recipe: Jacks + Aces + Long Suit ---")
    # Grouping by bins to see patterns
    grouped = df.groupby(['JacksCount', 'Aces', 'MaxSuitLen'])['WinProb'].agg(['count', 'mean'])
    # Filter for relevant combinations (count > 50 to be statistically mostly valid)
    print(grouped[grouped['count'] > 20].sort_values('mean', ascending=False).head(20))
    
    # Specifically answer: 1 Jack + ? Aces + ? Suit
    print("\n--- Deep Dive: 1 Jack Hands ---")
    j1 = df[df['JacksCount'] == 1]
    print(j1.groupby(['Aces', 'MaxSuitLen'])['WinProb'].agg(['count', 'mean']))

    # Specifically answer: 2 Jack Hands
    print("\n--- Deep Dive: 2 Jack Hands ---")
    j2 = df[df['JacksCount'] == 2]
    # Filter for relevant combinations
    stats_j2 = j2.groupby(['Aces', 'MaxSuitLen'])['WinProb'].agg(['count', 'mean'])
    print(stats_j2[stats_j2['count'] > 10]) # Filter low sample sizes

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python analyze_post_discard.py <csv_file>")
    else:
        analyze(sys.argv[1])
