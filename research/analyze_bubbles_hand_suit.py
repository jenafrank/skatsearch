
import pandas as pd
import matplotlib.pyplot as plt
import seaborn as sns
import os
import sys
import glob
import matplotlib.colors as mcolors
import plotly.express as px

# Set plot style for premium look
plt.style.use('seaborn-v0_8-darkgrid')
sns.set_context("notebook", font_scale=1.2)

def analyze_bubbles_hand_suit():
    # 1. Find the specific Hand Game CSV or latest cluster CSV
    # The user provided: research/data/hand_best_game_cluster.csv
    target_file = 'research/data/hand_best_game_cluster.csv'
    
    if not os.path.exists(target_file):
        print(f"File not found: {target_file}")
        return

    print(f"Processing for Suit Hand: {target_file}")

    try:
        df = pd.read_csv(target_file)
    except Exception as e:
        print(f"Error reading CSV: {e}")
        return

    # Strip whitespace from columns
    df.columns = df.columns.str.strip()
    
    # 2. Define Metrics
    # X-Axis: CntJ (Jacks Count)
    # Y-Axis: SafeFulls (Aces + Att10)
    # Category: TotalTrumps (CntJ + MxLen)
    
    # Ensure numeric
    suit_cols = ['ProbClubs', 'ProbSpades', 'ProbHearts', 'ProbDiamonds']
    cols_to_numeric = ['CntJ', 'Aces', 'Att10', 'MxLen'] + suit_cols
    for col in cols_to_numeric:
        if col in df.columns:
            df[col] = pd.to_numeric(df[col], errors='coerce')
        else:
            print(f"Missing column: {col}")
            # Fallback if specific suits are missing?
            # Assuming they exist based on previous checks.
            pass
            
    # Calculate Max Suit Probability (Best Suit Game)
    df['SuitHandProb'] = df[suit_cols].max(axis=1)
    
    # Use SuitHandProb as WinProb
    df['WinProb'] = df['SuitHandProb']

    # Calculate Safe Fulls (Stehende Volle) used in Pre-Discard
    # SafeFulls = Aces + Att10 (Standing Tens)
    df['SafeFulls'] = df['Aces'] + df['Att10']
    
    # Calculate Potential Total Trumps
    df['TotalTrumps'] = df['MxLen'] + df['CntJ']
    
    # 3. Filter Categories: Total Trumps
    categories = [4, 5, 6, 7] 
    
    output_dir = "research/plots"
    os.makedirs(output_dir, exist_ok=True)
    
    # Define Discrete Color Map (Purple, Red, Yellow, Green)
    # Biddable threshold has been shifting. 
    # For Hand games, we used:
    # <40 (Loss), <50 (Risky), <75 (Playable/Biddable), >75 (Win)
    
    for trumps in categories:
        if trumps == 7:
            subset = df[df['TotalTrumps'] >= 7].copy()
            title_suffix = "7+ Trumps (Suit Hand)"
            filename_base = "bubble_hand_suit_trumps_7_plus"
        else:
            subset = df[df['TotalTrumps'] == trumps].copy()
            title_suffix = f"{trumps} Trumps (Suit Hand)"
            filename_base = f"bubble_hand_suit_trumps_{trumps}"
            
        if len(subset) == 0:
            print(f"No data for {title_suffix}")
            continue
            
        # Group by Signature
        grouped = subset.groupby(['CntJ', 'SafeFulls']).agg(
            Count=('WinProb', 'count'),
            MeanWinRate=('WinProb', 'mean')
        ).reset_index()
        
        if len(grouped) == 0:
            continue

        # Create Bubble Chart
        plt.figure(figsize=(10, 8))
        
        # Bubble sizes
        sizes = grouped['Count'] 
        if sizes.empty: continue
        
        # Normalize size
        max_count = sizes.max()
        if max_count == 0: max_count = 1
        sizes_norm = (sizes / max_count) * 2000 + 100
        
        # Colors: <40 (Purple), 40-50 (Red), 50-75 (Yellow), >75 (Green)
        colors = ['#9C27B0', '#D32F2F', '#FFD700', '#388E3C'] 
        bounds = [0.0, 0.40, 0.50, 0.75, 1.0]
        cmap = mcolors.ListedColormap(colors)
        norm = mcolors.BoundaryNorm(bounds, cmap.N)
            
        scatter = plt.scatter(
            grouped['CntJ'], 
            grouped['SafeFulls'], 
            s=sizes_norm, 
            c=grouped['MeanWinRate'], 
            cmap=cmap,
            norm=norm,
            alpha=0.7, 
            edgecolors='w', 
            linewidth=2
        )
        
        # Add labels
        for i, row in grouped.iterrows():
            win_rate = row['MeanWinRate']
            # Text Color - ALWAYS BLACK and BOLD
            text_color = 'black'
             
            label = f"{win_rate:.0%}"
            if row['Count'] > grouped['Count'].max() * 0.2:
                label += f"\n(n={int(row['Count'])})"

            plt.text(
                row['CntJ'], 
                row['SafeFulls'], 
                label, 
                ha='center', 
                va='center', 
                fontsize=10, 
                color=text_color,
                fontweight='bold'
            )

        # Colorbar (Right side, stacked style like Pre-Discard)
        # Ticks centered in bins: [0.2, 0.45, 0.625, 0.875] roughly
        cbar = plt.colorbar(scatter, ticks=[0.2, 0.45, 0.625, 0.875])
        cbar.ax.set_yticklabels(['<40% (Loss)', '40-50% (Risk)', '50-75% (Play)', '>75% (Win)']) 
        cbar.set_label('Win Probability Category')

        # Titles and Labels
        plt.title(f"Suit Hand Strength: {title_suffix}", fontsize=16, pad=20)
        plt.xlabel("Jack Count (CntJ)", fontsize=14)
        plt.ylabel("Side Strength (Aces + Standing Tens)", fontsize=14)
        
        # Standardized Grids/Ticks/Limits (v7+)
        plt.xticks(range(5)) 
        plt.yticks(range(9))
        plt.grid(True, linestyle='--', alpha=0.7)
        plt.ylim(-0.5, 8.5)
        plt.xlim(-0.5, 4.5)

        out_path = f"{output_dir}/{filename_base}_v11.png"
        plt.savefig(out_path, dpi=100, bbox_inches='tight')
        plt.close()
        print(f"Saved: {out_path}")

        # --- Interactive Plotly ---
        # Map Color to Win Rate for Color Scale consistency
        grouped['Win %'] = grouped['MeanWinRate'].apply(lambda x: f"{x:.1%}")
        grouped['AvgWinProb'] = grouped['MeanWinRate'] # For hover data
        grouped['Samples'] = grouped['Count'] # For hover data

        fig = px.scatter(
            grouped, 
            x='CntJ', 
            y='SafeFulls', 
            size='Count', 
            color='MeanWinRate', # Use raw mean for continuous scale
            color_continuous_scale=[
                (0.00, '#9C27B0'),   # Purple 0%
                (0.40, '#9C27B0'),   # Purple 40%
                (0.40001, '#D32F2F'), # Red 40%
                (0.50, '#D32F2F'),   # Red 50%
                (0.50001, '#FFD700'), # Yellow 50%
                (0.75, '#FFD700'),   # Yellow 75%
                (0.75001, '#388E3C'), # Green 75%
                (1.00, '#388E3C')    # Green 100%
            ],
            range_color=[0, 1],
            hover_name='CntJ',
            hover_data={'CntJ': True, 'SafeFulls': True, 'Win %': ':.1%', 'Count': True, 'AvgWinProb': ':.1%', 'Samples': True},
            size_max=60,
            title=f"Suit Hand Strength: {title_suffix}",
            template="plotly_white"
        )
        
        # Add text labels for Win %
        fig.update_traces(text=grouped['Win %'], textposition='middle center', textfont=dict(color='black'))

        fig.update_layout(
            xaxis_title="Jack Count",
            yaxis_title="Side Strength (Aces + Standing Tens)",
            xaxis=dict(range=[-0.5, 4.5], tickmode='linear', tick0=0, dtick=1),
            yaxis=dict(range=[-0.5, 8.5], tickmode='linear', tick0=0, dtick=1),
            legend_title="Win Probability"
        )
        
        html_out = f"{output_dir}/{filename_base}_v11.html"
        fig.write_html(html_out)
        print(f"Saved Interactive: {html_out}")

if __name__ == "__main__":
    analyze_bubbles_hand_suit()
