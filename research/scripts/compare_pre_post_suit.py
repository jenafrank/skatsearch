import pandas as pd

def analyze_comparison():
    print("Loading data...")
    # Load Pre-Discard (Hand + Potential)
    # The 'Cards' column here is the INITIAL 10-card hand. The 'SkatCards' column (if present) is irrelevant for signature here usually, 
    # but the simulation calculates win probability assuming optimal skat pickup.
    # So 'JacksMask' and 'TrumpCount' here refer to the INITIAL hand? 
    # Let's verify: In 'analyze_suit_with_pickup', 'sig_initial' is from 'my_hand' (10 cards). 
    # So YES, Pre-Discard signature is the 10-card hand.
    
    pre = pd.read_csv("suit_pre_discard_500.csv")
    post = pd.read_csv("suit_post_discard_500.csv")
    
    # Filter winners
    pre_wins = pre[pre['WinProb'] >= 0.75]
    post_wins = post[post['WinProb'] >= 0.75]
    
    print(f"Pre-Discard Winners: {len(pre_wins)}")
    print(f"Post-Discard Winners: {len(post_wins)}")
    
    # Group by Signature
    def get_means(df):
        return df.groupby(['JacksMask', 'TrumpCount']).agg({
            'Aces': 'mean',
            'Tens': 'mean',
            'WinProb': 'mean',
            'Cards': 'count'
        }).rename(columns={'Cards': 'Count'})

    pre_stats = get_means(pre_wins)
    post_stats = get_means(post_wins)
    
    # Combine/Join
    # We want to see: For a given Jack/Trump config, what Side strength is needed PRE vs POST?
    
    # Outer interaction of indices
    all_indices = pre_stats.index.union(post_stats.index)
    
    print("\n=== Comparison: Needed Strength (Win > 75%) ===")
    print("Pre = Initial Hand (10 Cards), Post = Final Hand (10 Cards)")
    print("Values are Average Aces | Average Tens")
    print(f"{'Konstellation':<20} | {'Pre-Discard (Avg)':<20} | {'Post-Discard (Avg)':<20}")
    print("-" * 70)
    
    # Sort indices manually: Jacks (CJ, CSH, etc) then Trumps descending
    # Actually just simple sort might work ok, or we iterate known constellations.
    # Let's just iterate the sorted index.
    
    for idx in sorted(all_indices, key=lambda x: (len(x[0]), x[1]), reverse=True): 
        # Sort logic: Length of mask (more jacks) desc, then TrumpCount desc
        # Actually better sort: Convert JacksMask to value?
        # Let's just iterate naturally.
        pass

    # Better: Use pandas join
    joined = pd.concat([pre_stats, post_stats], axis=1, keys=['Pre', 'Post'])
    
    # Iterate and print nice table
    # joined index is (JacksMask, TrumpCount)
    
    # Custom Sorter for Jacks
    jack_order = {
        'CJ': 1, 'SJ': 2, 'HJ': 3, 'DJ': 4,
        'CS': 5, 'CH': 6, 'CD': 7, 
        'CSH': 8, 'CSD': 9, 'CHD': 10,
        'CSHD': 11
    }
    # We can try to sort by this if we reset index.
    
    joined = joined.reset_index()
    # Handle missing values
    joined = joined.fillna(0)  
    
    # Printing
    pd.set_option('display.max_rows', None)
    # columns are (Pre/Post, Aces/Tens/WinProb/Count)
    
    # Write to file
    with open("compare_out.txt", "w", encoding="utf-8") as f:
        f.write("\n=== Comparison: Needed Strength (Win > 75%) ===\n")
        f.write("Pre = Initial Hand (10 Cards), Post = Final Hand (10 Cards)\n")
        f.write("Values are Average Aces | Average Tens\n")
        f.write(f"{'Jacks':<10} {'Trumps':<6} | {'Pre Aces':<8} {'Pre Tens':<8} | {'Post Aces':<9} {'Post Tens':<9} | {'Pre Count':<9} {'Post Count':<9}\n")
        f.write("-" * 85 + "\n")
        
        for _, row in joined.iterrows():
            jacks = row[('JacksMask', '')]
            trumps = row[('TrumpCount', '')]
            
            pre_aces = row[('Pre', 'Aces')]
            pre_tens = row[('Pre', 'Tens')]
            pre_count = row[('Pre', 'Count')]
            
            post_aces = row[('Post', 'Aces')]
            post_tens = row[('Post', 'Tens')]
            post_count = row[('Post', 'Count')]
            
            # Only print if we have data (count > 0)
            if pre_count > 0 or post_count > 0:
                p_ace_str = f"{pre_aces:.1f}" if pre_count > 0 else "-"
                p_ten_str = f"{pre_tens:.1f}" if pre_count > 0 else "-"
                
                po_ace_str = f"{post_aces:.1f}" if post_count > 0 else "-"
                po_ten_str = f"{post_tens:.1f}" if post_count > 0 else "-"
                
                pc_str = f"{int(pre_count)}" if pre_count > 0 else "0"
                poc_str = f"{int(post_count)}" if post_count > 0 else "0"

                f.write(f"{jacks:<10} {trumps:<6} | {p_ace_str:<8} {p_ten_str:<8} | {po_ace_str:<9} {po_ten_str:<9} | {pc_str:<9} {poc_str:<9}\n")
        
    print("Comparison written to compare_out.txt")

if __name__ == "__main__":
    analyze_comparison()
