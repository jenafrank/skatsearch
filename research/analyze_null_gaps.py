
import pandas as pd
import collections

# Load Data
csv_path = 'research/data/null_sim_48h_combined.csv'
try:
    df = pd.read_csv(csv_path)
except Exception as e:
    print(f"Error loading CSV: {e}")
    exit()

# Filter for Opponent Leads
df_filtered = df[df['StartPlayer'].isin(['Left', 'Right'])]

print(f"Total Hands: {len(df)}")
print(f"Opponent Leads: {len(df_filtered)}")

# Suit Mapping
# User uses "7-9-U-K" (Unter=J) and "7-9-Q" (Queen=Ober).
# CSV uses: 7, 8, 9, T, J, Q, K, A
# We will output: 7, 8, 9, 10, J, Q, K, A (using '10' instead of 'T')

rank_map = {
    '7': '7', '8': '8', '9': '9', 'T': '10', 
    'J': 'J', 'Q': 'Q', 'K': 'K', 'A': 'A'
}
rank_order = {'7':0, '8':1, '9':2, '10':3, 'J':4, 'Q':5, 'K':6, 'A':7}

def parse_hand_suits(hand_str):
    cards = hand_str.split()
    suits = collections.defaultdict(list)
    for card in cards:
        suit = card[0]
        rank_char = card[1:] # e.g. 'T', '9', 'K'
        
        # Map Rank
        if rank_char in rank_map:
            rank = rank_map[rank_char]
        else:
            rank = rank_char # Fallback
            
        suits[suit].append(rank)
    
    # Sort ranks for each suit
    suit_patterns = []
    for s in ['C', 'S', 'H', 'D']: # Canonical order doesn't matter for grouping, but good for processing
        if s in suits:
            # Sort by rank index
            ranks = sorted(suits[s], key=lambda r: rank_order.get(r, 99))
            pattern = "-".join(ranks)
            suit_patterns.append(pattern)
        else:
            suit_patterns.append("Void") # Void suit
            
    return suit_patterns

# Aggregation
stats = collections.defaultdict(lambda: {'wins': 0, 'total': 0})

for idx, row in df_filtered.iterrows():
    won = 1 if row['Won'] else 0
    patterns = parse_hand_suits(row['Hand'])
    
    for p in patterns:
        stats[p]['total'] += 1
        stats[p]['wins'] += won

# Convert to DataFrame for sorting
results = []
for p, data in stats.items():
    win_rate = (data['wins'] / data['total']) * 100
    results.append({
        'Pattern': p,
        'Count': data['total'],
        'Wins': data['wins'],
        'WinRate': win_rate
    })

results_df = pd.DataFrame(results)

# Sort by Pattern for consistent looking up? Or by Count?
# User wants to know "healthy win probability (~60%+)".
# Let's sort by Win Rate descending, but filter for statistical significance (e.g. at least 50 occurrences?)

min_samples = 50
results_df_sig = results_df[results_df['Count'] >= min_samples].sort_values(by='WinRate', ascending=False)

print("\n--- Top Safe Holdings (>60% Win Rate, Min 50 samples) ---")
print(results_df_sig[results_df_sig['WinRate'] >= 60.0].head(20).to_string(index=False))

print("\n--- Risky Holdings (<60% Win Rate, Min 50 samples) ---")
print(results_df_sig[results_df_sig['WinRate'] < 60.0].sort_values(by='WinRate').head(20).to_string(index=False))

# specific gaps
gaps_of_interest = ["8", "9", "7-10", "7-9-Q", "7-9-K", "7-9-J", "7-8", "7-9", "8-10", "7-J", "7-Q", "7-K", "7-A"]
print("\n--- Specific First-Look Gaps ---")
for g in gaps_of_interest:
    row = results_df[results_df['Pattern'] == g]
    if not row.empty:
        print(row.to_string(index=False, header=False))
    else:
        print(f"{g}: No data")

# Save detailed report
results_df.sort_values(by=['Count', 'WinRate'], ascending=False).to_csv('research/null_gaps_analysis.csv', index=False)
print("\nFull analysis saved to 'research/null_gaps_analysis.csv'")
