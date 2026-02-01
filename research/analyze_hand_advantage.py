
import pandas as pd
import matplotlib.pyplot as plt
import seaborn as sns
import os
import glob
import numpy as np

# Set style
plt.style.use('seaborn-v0_8-darkgrid')
sns.set_context("notebook", font_scale=1.2)

def analyze_hand_advantage():
    # 1. Load Hand Game Data
    hand_file = 'research/data/hand_best_game_cluster.csv'
    if not os.path.exists(hand_file):
        print("Hand file not found.")
        return
    
    print(f"Loading Hand Data: {hand_file}")
    df_hand_raw = pd.read_csv(hand_file)
    df_hand_raw.columns = df_hand_raw.columns.str.strip()
    
    # Calculate Signature Columns
    df_hand_raw['SafeFulls'] = df_hand_raw['Aces'] + df_hand_raw['Att10']
    df_hand_raw['TotalTrumps'] = df_hand_raw['MxLen'] + df_hand_raw['CntJ']
    
    # Aggregating Hand Data
    # We want Mean WinProb for each signature
    group_cols = ['TotalTrumps', 'CntJ', 'SafeFulls']
    df_hand = df_hand_raw.groupby(group_cols)['WinProb'].mean().reset_index()
    df_hand.rename(columns={'WinProb': 'Prob_Hand'}, inplace=True)
    
    # 2. Load Pickup (Pre-Discard) Data
    list_of_files = glob.glob('research/data/general_pre_stats_final_50k_*.csv')
    if not list_of_files:
        print("No Pre-Discard CSV found.")
        return
    pickup_file = max(list_of_files, key=os.path.getctime)
    
    print(f"Loading Pickup Data: {pickup_file}")
    df_pickup_raw = pd.read_csv(pickup_file)
    df_pickup_raw.columns = df_pickup_raw.columns.str.strip()
    
    # Calculate Signature Columns (Use Pre-Discard metrics)
    df_pickup_raw['SafeFulls'] = df_pickup_raw['Aces'] + df_pickup_raw['Att10']
    df_pickup_raw['TotalTrumps'] = df_pickup_raw['MxLen'] + df_pickup_raw['CntJ']
    
    # Aggregating Pickup Data
    # Use 'MaxProb' (Best game after pickup)
    df_pickup = df_pickup_raw.groupby(group_cols)['MaxProb'].mean().reset_index()
    df_pickup.rename(columns={'MaxProb': 'Prob_Pickup'}, inplace=True)
    
    # 3. Merge Datasets
    merged = pd.merge(df_hand, df_pickup, on=group_cols, how='inner')
    
    # 4. Apply "Golden Rule" / Decision Boundary
    # Condition: Prob_Hand > 1.125 * Prob_Pickup - 0.25
    # Let's calculate the "Hand Advantage Score"
    # Score > 0 means Play Hand.
    merged['Threshold'] = 1.125 * merged['Prob_Pickup'] - 0.25
    merged['Advantage'] = merged['Prob_Hand'] - merged['Threshold']
    
    # Filter for interesting zones (Total Trumps 4, 5, 6, 7)
    categories = [4, 5, 6, 7]
    
    output_dir = "research/plots"
    os.makedirs(output_dir, exist_ok=True)
    
    print("\n--- HAND GAME RECOMMENDATIONS ---")
    
    for trumps in categories:
        if trumps == 7:
            subset = merged[merged['TotalTrumps'] >= 7].copy()
            title_suffix = "7+ Trumps"
        else:
            subset = merged[merged['TotalTrumps'] == trumps].copy()
            title_suffix = f"{trumps} Trumps"
            
        if len(subset) == 0: continue
        
        # Identify "Play Hand" recommmendations
        recommendations = subset[subset['Advantage'] > 0.0].sort_values(by='Advantage', ascending=False)
        
        if not recommendations.empty:
            print(f"\n[{title_suffix}]: Play Hand in these cases:")
            for _, row in recommendations.iterrows():
                # Only show if meaningful sample impact (omitted here as we aggregated means, assuming sufficient N)
                # Filter noise: if Advantage is tiny (< 2%) maybe ignore?
                if row['Advantage'] > 0.01: 
                    print(f"  - {int(row['CntJ'])} Jacks, {int(row['SafeFulls'])} SafeFulls: "
                          f"Hand {row['Prob_Hand']:.1%} vs Pickup {row['Prob_Pickup']:.1%} "
                          f"(Advantage: +{row['Advantage']:.1%})")

        # Filter out "Hopeless" hands (Hand < 50% AND Pickup < 66%)
        # User request: "Lasse Bubbles ganz weg die weder Hand noch pre-discard gewinnen" (updated threshold)
        subset = subset[(subset['Prob_Hand'] >= 0.5) | (subset['Prob_Pickup'] >= 0.66)]
        
        if len(subset) == 0:
            print(f"No winning hands for {title_suffix}")
            continue

        # Visualization: Difference Chart
        plt.figure(figsize=(12, 10)) # Slightly larger for text
        import matplotlib.patheffects as PathEffects
        
        # Plot all points
        # Color: Red (Pickup) vs Blue (Hand)
        # Size: Magnitude of advantage/disadvantage
        
        subset['Color'] = subset['Advantage'].apply(lambda x: 'blue' if x > 0 else 'red')
        subset['AbsAdvantage'] = subset['Advantage'].abs()
        
        # Scale size
        sizes = subset['AbsAdvantage'] * 3000 + 300 # Larger bubbles for text
        
        plt.scatter(
            subset['CntJ'], 
            subset['SafeFulls'], 
            s=sizes, 
            c=subset['Color'], 
            alpha=0.7, 
            edgecolors='black',
            linewidth=1
        )
        
        # Labels
        for _, row in subset.iterrows():
            text_color = 'white'
             
            # Label format:
            # H: 90%
            # P: 80%
            label = f"H:{row['Prob_Hand']:.0%}\nP:{row['Prob_Pickup']:.0%}"
            
            plt.text(
                row['CntJ'], 
                row['SafeFulls'], 
                label, 
                ha='center', 
                va='center', 
                fontsize=11, 
                color=text_color, 
                fontweight='bold',
                path_effects=[PathEffects.withStroke(linewidth=2, foreground='black')]
            )
            
            # Optional: Add "PLAY" tag if highly advantageous
            if row['Advantage'] > 0.05:
                plt.text(
                    row['CntJ'], 
                    row['SafeFulls'] + 0.3, # Shift up 
                    "PLAY!", 
                    ha='center', 
                    fontsize=12, 
                    color='blue', 
                    fontweight='bold',
                    path_effects=[PathEffects.withStroke(linewidth=2, foreground='white')]
                )

        plt.title(f"Hand Decision Map: {title_suffix}\n(Filtered: Hand > 50% OR Pickup > 66%)", fontsize=16)
        plt.xlabel("Jack Count")
        plt.ylabel("Safe Fulls (Aces + Standing Tens)")
        plt.grid(True)
        plt.xticks(range(0, 5))
        plt.yticks(range(0, 9))
        
        # Legend (Manual)
        from matplotlib.lines import Line2D
        legend_elements = [
            Line2D([0], [0], marker='o', color='w', label='Play Hand', markerfacecolor='blue', markersize=15),
            Line2D([0], [0], marker='o', color='w', label='Pickup Skat', markerfacecolor='red', markersize=15)
        ]
        plt.legend(handles=legend_elements, loc='upper right')
        
        out_path = f"{output_dir}/decision_map_trumps_{trumps}_v6.png"
        plt.savefig(out_path)
        plt.close()
        print(f"Saved plot: {out_path}")

    # Generate HTML summary for interactive view
    import plotly.express as px
    merged['Recommendation'] = merged['Advantage'].apply(lambda x: "PLAY HAND" if x > 0 else "PICK UP")
    merged['Diff'] = merged['Advantage']
    
    fig = px.scatter(
        merged, 
        x='CntJ', 
        y='SafeFulls', 
        facet_col='TotalTrumps', 
        color='Diff',
        color_continuous_scale='RdBu',
        size='Prob_Hand', # Size by probability roughly
        hover_data=['Prob_Hand', 'Prob_Pickup', 'Advantage', 'Recommendation'],
        title="Hand vs Pickup Decision Landscape (Blue = Play Hand)"
    )
    fig.write_html(f"{output_dir}/decision_map_interactive.html")

if __name__ == "__main__":
    analyze_hand_advantage()
