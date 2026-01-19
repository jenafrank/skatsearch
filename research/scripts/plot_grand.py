import pandas as pd
import matplotlib.pyplot as plt
import numpy as np

def parse_cards_max_suit(card_str):
    # Same logic as analyze_thresholds
    if not isinstance(card_str, str): return 0
    content = card_str.strip('[]')
    cards = content.split()
    suits = {'C': 0, 'S': 0, 'H': 0, 'D': 0}
    for c in cards:
        if 'J' not in c: # Exclude Jacks
            s_char = c[0]
            if s_char in suits:
                suits[s_char] += 1
    if not suits: return 0
    return max(suits.values())

def plot_grand():
    # Load data
    pre_file = "../data/grand_pickup_1000.csv"
    post_file = "../data/grand_post_discard_10k.csv"
    
    try:
        pre = pd.read_csv(pre_file)
        post = pd.read_csv(post_file)
        print(f"Loaded Pre: {len(pre)}, Post: {len(post)}")
    except Exception as e:
        print(f"Error loading files: {e}")
        return

    # Process Data
    def prepare_data(df):
        df['NumJacks'] = df['JacksMask'].apply(lambda x: 0 if str(x) in ['-', 'nan', ''] else len(str(x)))
        # Calculate Y (Fulls)
        if 'SkatFulls' in df.columns:
            df['Y'] = df['Aces'] + df['AttachedTens'] + df['SkatFulls']
        else:
            df['Y'] = df['Aces'] + df['AttachedTens']
        
        # Calculate Effective Length (Jacks + Max Side Suit)
        df['MaxSuit'] = df['Cards'].apply(parse_cards_max_suit)
        df['EffLength'] = df['NumJacks'] + df['MaxSuit']
        return df

    pre = prepare_data(pre)
    post = prepare_data(post)

    # Plotting Helper
    def plot_on_ax(ax, data, title, centroid, color):
        if data.empty:
            ax.text(0.5, 0.5, "No Data", ha='center')
            return
            
        grouped = data.groupby(['NumJacks', 'Y']).agg({
            'WinProb': 'mean',
            'Cards': 'count'
        }).reset_index()
        grouped.rename(columns={'Cards': 'Count'}, inplace=True)
        
        sizes = grouped['WinProb'] * 1500
        
        ax.scatter(grouped['NumJacks'], grouped['Y'], s=sizes, alpha=0.5, c=color, edgecolors='black')
        
        if centroid != (0,0):
            ax.scatter([centroid[0]], [centroid[1]], color='red', s=150, marker='X', label='Schwerpunkt')
            
        for i, row in grouped.iterrows():
            if row['WinProb'] > 0.05:
                # Calculate Win %
                pct = int(row['WinProb'] * 100)
                # Count used for something? No, just visual.
                ax.annotate(f"{pct}%", (row['NumJacks'], row['Y']), 
                            xytext=(0,0), textcoords='offset points', ha='center', va='center', color='black', weight='bold', size=8)

        ax.set_title(title + f"\nSchwerpunkt: ({centroid[0]:.2f}, {centroid[1]:.2f}) - N={len(data)}")
        ax.set_xlabel("Anzahl Buben")
        ax.set_ylabel("Asse + Stehende Volle")
        ax.grid(True, linestyle='--', alpha=0.7)
        ax.set_xticks(range(5))
        ax.set_yticks(range(0, 9))
        ax.legend(loc='lower left')

    def get_centroid(df):
        total_prob = df['WinProb'].sum()
        if total_prob == 0: return (0,0)
        cx = (df['NumJacks'] * df['WinProb']).sum() / total_prob
        cy = (df['Y'] * df['WinProb']).sum() / total_prob
        return (cx, cy)

    # Output Slices (3 to 7)
    targets = [3, 4, 5, 6, 7]
    
    for length in targets:
        print(f"Generating Plot for Effective Length {length}...")
        
        pre_subset = pre[pre['EffLength'] == length]
        post_subset = post[post['EffLength'] == length]
        
        # Skip empty plots? No, generate even if empty to show missing data clearly.
        
        cx_pre, cy_pre = get_centroid(pre_subset)
        cx_post, cy_post = get_centroid(post_subset)
        
        fig, axes = plt.subplots(1, 2, figsize=(14, 6), sharex=True, sharey=True)
        
        plot_on_ax(axes[0], pre_subset, f"Grand Pre-Discard (Len {length})", (cx_pre, cy_pre), 'blue')
        plot_on_ax(axes[1], post_subset, f"Grand Post-Discard (Len {length})", (cx_post, cy_post), 'orange')
        
        outfile = f"../plots/grand_prob_length_{length}.png"
        plt.tight_layout()
        plt.savefig(outfile)
        print(f"  Saved {outfile}")
        plt.close(fig)

if __name__ == "__main__":
    plot_grand()
