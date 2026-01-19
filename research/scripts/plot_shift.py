import pandas as pd
import matplotlib.pyplot as plt
import numpy as np

import pandas as pd
import matplotlib.pyplot as plt
import numpy as np

def plot_shift():
    # Load data
    csv_file = "../data/suit_large_10k.csv"
    try:
        df = pd.read_csv(csv_file)
        print(f"Loaded {len(df)} rows from {csv_file}")
    except Exception as e:
        print(f"Error reading {csv_file}: {e}")
        # Try fallbacks or exit
        return

    # Filter Winners? Or keep all for probability?
    # User wants Probability visualization, so we keep all.

    def process_row(row, suffix=""):
        # Helper to extract data from a row for Pre or Post
        # Columns: JacksMask, TrumpCount, Aces, Tens, AttachedTens
        # Suffix is "" for Pre, "Post" for Post.
        
        jacks_col = f"{suffix}JacksMask" if suffix else "JacksMask"
        # aces_col = f"{suffix}Aces" ...
        
        # Check if column exists
        if jacks_col not in row:
            return None
            
        mask = str(row[jacks_col]).strip()
        num_jacks = 0 if mask in ["-", ""] else len(mask)
        
        def get_val(col_base):
            col = f"{suffix}{col_base}" if suffix else col_base
            return row[col] if col in row else 0

        aces = get_val("Aces")
        att_tens = get_val("AttachedTens")
        skat_fulls = get_val("SkatFulls")
        
        y = aces + att_tens + skat_fulls
        
        trump_col = f"{suffix}TrumpCount" if suffix else "TrumpCount"
        trump_count = row[trump_col] if trump_col in row else 0
        
        return {
            'NumJacks': num_jacks,
            'Y': y,
            'TrumpCount': trump_count,
            'WinProb': row['WinProb'] # WinProb is common for the game
        }

    # Iterate rows and build Pre/Post dataframes
    pre_list = []
    post_list = []
    
    for _, row in df.iterrows():
        # Pre
        p = process_row(row, "")
        if p: pre_list.append(p)
        
        # Post
        # Note: Pre-discard CSV had 'TrumpCount'. New CSV has 'TrumpCount' (Pre) and 'PostTrumpCount'.
        post = process_row(row, "Post")
        if post: post_list.append(post)
    
    pre = pd.DataFrame(pre_list)
    post = pd.DataFrame(post_list)

    def process_data_stats(df_in):
        if df_in.empty: return df_in
        # Group by coordinates, calculate Mean WinProb
        grouped = df_in.groupby(['NumJacks', 'Y']).agg({
            'WinProb': 'mean',
            'TrumpCount': 'first', # Dummy, we actually filter by TrumpCount outside
             # Wait, we need to preserve rows or group later?
             # The plotting function expects grouped data.
             # But we split by TrumpCount logic LATER.
             # So here we should probably NOT group yet unless we group by TrumpCount too.
        }).reset_index() # Grouping removes individual rows. 
        # But `process_data` inside `plot_shift` grouped by X,Y.
        return df_in # Return raw for filtering

    # Update logic functions...
    
    # Helper for Plotting
    def plot_on_ax(ax, data, title, centroid):
        if data.empty:
            ax.text(0.5, 0.5, "No Data", ha='center')
            return
            
        # Group stats here
        grouped = data.groupby(['NumJacks', 'Y']).agg({
            'WinProb': 'mean',
            'TrumpCount': 'count' # Use this for sizing? No, user wants Size ~ Prob.
        }).reset_index()
        grouped.rename(columns={'TrumpCount': 'Count', 'WinProb': 'AvgWinProb'}, inplace=True)
        
        # Scale factor
        sizes = grouped['AvgWinProb'] * 1500 
        
        scatter = ax.scatter(grouped['NumJacks'], grouped['Y'], s=sizes, alpha=0.5, c='green', edgecolors='black')
        
        # Plot Centroid
        if centroid != (0,0):
            ax.scatter([centroid[0]], [centroid[1]], color='red', s=150, marker='X', label='Schwerpunkt (Erw. Sieg)')
        
        # Labels: Show Win % inside
        for i, row in grouped.iterrows():
            if row['AvgWinProb'] > 0.05: 
                pct = int(row['AvgWinProb'] * 100)
                ax.annotate(f"{pct}%", (row['NumJacks'], row['Y']), 
                            xytext=(0,0), textcoords='offset points', ha='center', va='center', color='black', weight='bold', size=8)

        ax.set_title(title + f"\nSchwerpunkt: ({centroid[0]:.2f}, {centroid[1]:.2f})")
        ax.set_xlabel("Anzahl Buben")
        ax.set_ylabel("Asse + Stehende Volle")
        ax.grid(True, linestyle='--', alpha=0.7)
        ax.set_xticks(range(5)) 
        ax.set_yticks(range(0, 7)) 
        ax.legend(loc='lower left')

    def get_weighted_centroid(df_raw):
        # Weighted by WinProb individually
        total_prob = df_raw['WinProb'].sum()
        if total_prob == 0: return (0,0)
        
        cx = (df_raw['NumJacks'] * df_raw['WinProb']).sum() / total_prob
        cy = (df_raw['Y'] * df_raw['WinProb']).sum() / total_prob
        return (cx, cy)

    # Plotting Logic for a specific Trump Count
    def generate_plot_for_trumps(target_trumps):
        print(f"\nGeneriere Probability Plot für {target_trumps} Trümpfe...")
        
        # Filter raw data
        pre_subset = pre[pre['TrumpCount'] == target_trumps].copy()
        post_subset = post[post['TrumpCount'] == target_trumps].copy()
        
        print(f"  Samples: Pre={len(pre_subset)}, Post={len(post_subset)}")
        
        cx_pre, cy_pre = get_weighted_centroid(pre_subset)
        cx_post, cy_post = get_weighted_centroid(post_subset)
        
        fig, axes = plt.subplots(1, 2, figsize=(14, 6), sharex=True, sharey=True)
        
        plot_on_ax(axes[0], pre_subset, f"Pre-Discard Prob ({target_trumps} Trümpfe)", (cx_pre, cy_pre))
        plot_on_ax(axes[1], post_subset, f"Post-Discard Prob ({target_trumps} Trümpfe)", (cx_post, cy_post))
        
        filename = f"../plots/suit_10k_prob_{target_trumps}_trumps.png"
        plt.tight_layout()
        plt.savefig(filename)
        print(f"  Saved {filename}")
        plt.close(fig)

    # Generate for 4 (User Request)
    for t in [4]:
        generate_plot_for_trumps(t)

if __name__ == "__main__":
    plot_shift()

if __name__ == "__main__":
    plot_shift()
