import pandas as pd
import re

def parse_cards(card_str):
    if not isinstance(card_str, str):
        return set()
    return set(re.findall(r'[CDHS][JKQA987T]', card_str))

# Define constants
JACKS = {'CJ', 'SJ', 'HJ', 'DJ'}
SUIT_MAP = {'C': 0, 'S': 1, 'H': 2, 'D': 3}
# Assume Clubs is Trump for the simulation (as per main.rs defaults)
TRUMP_SUIT_CHAR = 'C' 

def get_hand_stats(row):
    # Use PlayedCards if available, otherwise fallback (which will be inaccurate for now)
    if 'PlayedCards' in row:
        played_cards = parse_cards(row['PlayedCards'])
    else:
        # Fallback to deduction (only works if we implement SkatFulls deduction - which we skipped)
        initial_cards = parse_cards(row['Cards'])
        skat_cards = parse_cards(row.get('SkatCards', ''))
        played_cards = initial_cards - skat_cards # This is wrong approx but strict for now
    
    # Count Trumps and Side Suits
    trumps = 0
    # Side suits counts: S, H, D
    side_counts = {'S': 0, 'H': 0, 'D': 0}
    
    for card in played_cards:
        rank = card[1]
        suit = card[0]
        
        if rank == 'J' or suit == TRUMP_SUIT_CHAR:
            trumps += 1
        else:
            side_counts[suit] += 1
            
    # Count Suits (including Trump as a "Suit" category if > 0)
    # Actually, usually "3-Farben-Spiel" means "Trumps + 2 Side Suits".
    # Or "4-Farben-Spiel" means "Trumps + 3 Side Suits".
    # We will count "Active Side Suits".
    active_side_suits = sum(1 for count in side_counts.values() if count > 0)
    
    # Total "colors" involved (Trump + Side Suits)
    total_colors = (1 if trumps > 0 else 0) + active_side_suits
    
    return pd.Series({
        'PlayedTrumps': trumps,
        'ActiveSideSuits': active_side_suits,
        'TotalColors': total_colors,
        'SideAces': row['Aces'], 
        'SideTens': row['Tens']
    })

def analyze(filename):
    print(f"Analyzing {filename}...")
    try:
        df = pd.read_csv(filename)
    except Exception as e:
        print(f"Could not read {filename}: {e}")
        return

    # Filter for Wins > 75%
    winners = df[df['WinProb'] >= 0.75].copy()
    print(f"Found {len(winners)} winning hands (>= 75%) out of {len(df)}")
    
    if len(winners) == 0:
        return

    # Enrich data
    # We apply parsing to get accurate Side Suit counts from the actual Played Hand
    stats = winners.apply(get_hand_stats, axis=1)
    winners = pd.concat([winners, stats], axis=1)
    
    # Q1: Overview Table
    # Group by JacksMask and TrumpCount (using the csv TrumpCount which is signature based)
    # Check consistency: df['TrumpCount'] vs calculated 'PlayedTrumps'. Should be identical.
    
    # Simple Grouping
    # We summarize needed side strength.
    summary = winners.groupby(['JacksMask', 'TrumpCount']).agg({
        'Aces': 'mean',
        'Tens': 'mean',
        'WinProb': 'mean',
        'ActiveSideSuits': 'mean',
        'TotalColors': 'mean',
        'Cards': 'count' # Using Cards column just to count rows
    }).rename(columns={'Cards': 'Count'})
    
    print("\n=== Overview Table (Win > 75%) ===")
    print("Columns: Avg Aces, Avg Tens, WinProb, Avg Active Side Suits")
    pd.set_option('display.max_rows', None)
    pd.set_option('display.width', 1000)
    print(summary)
    
    # Q2: Suit Distribution
    print("\n=== Suit Distribution Analysis (Win > 75%) ===")
    dist = winners['ActiveSideSuits'].value_counts().sort_index()
    print("Number of Active Side Suits (Non-Trump):")
    for suits, count in dist.items():
        pct = (count / len(winners)) * 100
        print(f"{suits} Side Suits: {count} hands ({pct:.1f}%)")
        
    print("\nInterpretation:")
    print("0 Side Suits = Trump Only (Mono)")
    print("1 Side Suit  = Trump + 1 Suit (2-Farben-Spiel)")
    print("2 Side Suits = Trump + 2 Suits (3-Farben-Spiel)")
    print("3 Side Suits = Trump + 3 Suits (4-Farben-Spiel)")

    # Q3: Detailed check on "Away-Pressed" (Discarded) colors
    # Did we reduce the suit count?
    # We compare Initial Active Side Suits vs Played Active Side Suits
    
    def get_initial_suit_count(row):
        initial_cards = parse_cards(row['Cards'])
        side_counts = {'S': 0, 'H': 0, 'D': 0}
        for card in initial_cards:
            if card[1] != 'J' and card[0] != TRUMP_SUIT_CHAR:
                side_counts[card[0]] += 1
        return sum(1 for count in side_counts.values() if count > 0)

    winners['InitialSideSuits'] = winners.apply(get_initial_suit_count, axis=1)
    winners['SuitReduction'] = winners['InitialSideSuits'] - winners['ActiveSideSuits']
    
    print("\n=== Suit Reduction Analysis ===")
    reduction_dist = winners['SuitReduction'].value_counts().sort_index()
    print("Suit Count Reduction (How many suits eliminated):")
    for red, count in reduction_dist.items():
        pct = (count / len(winners)) * 100
        print(f"Reduced by {red}: {count} hands ({pct:.1f}%)")

import sys

if __name__ == "__main__":
    if len(sys.argv) > 1:
        filename = sys.argv[1]
    else:
        filename = "suit_detailed_run.csv"
    
    analyze(filename)
