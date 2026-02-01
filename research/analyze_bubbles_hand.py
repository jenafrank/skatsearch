
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

def analyze_bubbles_hand():
    # 1. Find the specific Hand Game CSV or latest cluster CSV
    # The user provided: research/data/hand_best_game_cluster.csv
    target_file = 'research/data/hand_best_game_cluster.csv'
    
    if not os.path.exists(target_file):
        print(f"File not found: {target_file}")
        return

    print(f"Processing: {target_file}")

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
    cols_to_numeric = ['CntJ', 'Aces', 'Att10', 'MxLen', 'WinProb']
    for col in cols_to_numeric:
        if col in df.columns:
            df[col] = pd.to_numeric(df[col], errors='coerce')
        else:
            print(f"Missing column: {col}")
            return
            
    # Calculate Safe Fulls (Stehende Volle) used in Pre-Discard
    # SafeFulls = Aces + Att10 (Standing Tens)
    df['SafeFulls'] = df['Aces'] + df['Att10']
    
    # Calculate Potential Total Trumps
    # MxLen (Suit Length generally excludes Jacks in Skat notation? Let's check logic)
    # In Pre-Analysis: 'Count' was used.
    # Here: 'MxLen' + 'CntJ'.
    df['TotalTrumps'] = df['MxLen'] + df['CntJ']
    
    # 3. Filter Categories: Total Trumps
    categories = [4, 5, 6, 7] 
    
    output_dir = "research/plots"
    os.makedirs(output_dir, exist_ok=True)
    
    # Define Discrete Color Map (Purple, Red, Yellow, Green)
    colors = ['#9C27B0', '#D32F2F', '#FFD700', '#388E3C'] 
    bounds = [0.0, 0.50, 0.65, 0.75, 1.0]
    cmap = mcolors.ListedColormap(colors)
    norm = mcolors.BoundaryNorm(bounds, cmap.N)

    for trumps in categories:
        if trumps == 7:
            subset = df[df['TotalTrumps'] >= 7].copy()
            title_suffix = "7+ Trumps (Hand Game)"
            filename_base = "bubble_hand_trumps_7_plus"
        else:
            subset = df[df['TotalTrumps'] == trumps].copy()
            title_suffix = f"{trumps} Trumps (Hand Game)"
            filename_base = f"bubble_hand_trumps_{trumps}"
            
        if len(subset) == 0:
            print(f"No data for {trumps} Trumps")
            continue
            
        # Group by Signature
        grouped = subset.groupby(['CntJ', 'SafeFulls']).agg(
            Count=('WinProb', 'count'),
            MeanWinRate=('WinProb', 'mean')
        ).reset_index()
        
        # DEBUG: Print count for 1 Jack, 0 SafeFulls
        if trumps == 4:
             debug_row = grouped[(grouped['CntJ'] == 1) & (grouped['SafeFulls'] == 0)]
             if not debug_row.empty:
                 print(f"DEBUG HAND (4 Trumps, 1J, 0Safe): n={debug_row['Count'].values[0]}")
        
        if len(grouped) == 0:
            continue

        # --- Matplotlib Chart ---
        plt.figure(figsize=(10, 8))
        
        sizes = grouped['Count'] 
        # Normalize size for display
        sizes_norm = (sizes / sizes.max()) * 2000 + 100
        
        # Colors based on Win Rate (Updated V6 Scheme)
        # < 40%: Loss (Purple)
        # 40-50%: Risky (Red)
        # 50-75%: Playable/Biddable (Yellow)
        # > 75%: Win (Green)
        def get_color(p):
            if p < 0.40: return '#9C27B0' # Purple
            if p < 0.50: return '#D32F2F' # Red
            if p < 0.75: return '#FFD700' # Gold
            return '#388E3C' # Green

        colors = grouped['MeanWinRate'].apply(get_color)
        
        # Scatter with size based on frequency (Count)
        # Normalize size for display
        sizes = grouped['Count']
        sizes_norm = (sizes / sizes.max()) * 2000 + 100
        
        scatter = plt.scatter(
            x=grouped['CntJ'], 
            y=grouped['SafeFulls'], 
            s=sizes_norm, 
            c=colors, 
            alpha=0.9, 
            edgecolors='black',
            linewidth=0.5
        )
        
        # Add labels to ALL bubbles
        for i, row in grouped.iterrows():
            win_rate = row['MeanWinRate']
            # Contrast text color
            text_color = 'black' if (0.50 <= win_rate < 0.75) else 'white'
            
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

        # Build Colorbar (Fake it with Legend patches to match discrete categories)
        from matplotlib.patches import Patch
        legend_elements = [
            Patch(facecolor='#9C27B0', edgecolor='black', label='<40% (Loss)'),
            Patch(facecolor='#D32F2F', edgecolor='black', label='40-50% (Risky)'),
            Patch(facecolor='#FFD700', edgecolor='black', label='50-75% (Playable)'),
            Patch(facecolor='#388E3C', edgecolor='black', label='>75% (Win)')
        ]
        plt.legend(handles=legend_elements, loc='upper right')

        # Titles and Labels
        plt.title(f"Hand Strength Analysis: {title_suffix}\n(Purple <40 | Red <50 | Yellow <75 | Green >75)", fontsize=16, pad=20)
        plt.xlabel("Jack Count (CntJ)", fontsize=14)
        plt.ylabel("Side Strength (Aces + Standing Tens)", fontsize=14)
        
        # Grid/Ticks
        plt.xticks(range(0, 5)) 
        plt.yticks(range(0, 9))
        plt.grid(True, linestyle='--', alpha=0.7)
        plt.ylim(-0.5, 8.5)
        plt.xlim(-0.5, 4.5)

        out_path = f"{output_dir}/{filename_base}_v7.png"
        plt.savefig(out_path, dpi=100, bbox_inches='tight')
        plt.close()
        print(f"Saved: {out_path}")

        # --- Interactive Plotly Chart ---
        
        # Add color names for Plotly
        def get_color_category(p):
            if p < 0.40: return "Lose (<40%)"
            if p < 0.50: return "Risky (40-50%)"
            if p < 0.75: return "Playable (50-75%)"
            return "Win (>75%)"
            
        grouped['Category'] = grouped['MeanWinRate'].apply(get_color_category)
        grouped['Win %'] = grouped['MeanWinRate'].apply(lambda x: f"{x:.1%}")
        
        # Map categories to colors
        color_map = {
            "Lose (<40%)": "#9C27B0",
            "Risky (40-50%)": "#D32F2F",
            "Playable (50-75%)": "#FFD700",
            "Win (>75%)": "#388E3C"
        }

        fig = px.scatter(
            grouped,
            x='CntJ',
            y='SafeFulls',
            size='Count',
            color='Category',
            color_discrete_map=color_map,
            hover_name='Category',
            hover_data={'CntJ': True, 'SafeFulls': True, 'Win %': True, 'Count': True, 'Category': False},
            size_max=60,
            title=f"Hand Game Analysis: {title_suffix}",
            template="plotly_white"
        )
        
        # Add text labels to Plotly bubbles
        fig.update_traces(text=grouped['Win %'], textposition='middle center', textfont=dict(color='white')) 
        # Note: Plotly text color is tricky with variable backgrounds, setting white for now or auto
        # To make it perfect we'd need a column for text color, but Plotly 'textfont' takes single value or array.
        # Let's try to imply text color array.
        text_colors = ['black' if (0.50 <= r['MeanWinRate'] < 0.75) else 'white' for i, r in grouped.iterrows()]
        fig.update_traces(textfont=dict(color=text_colors))

        fig.update_layout(
            xaxis_title="Jack Count",
            yaxis_title="Side Strength (Aces + Standing Tens)",
            xaxis=dict(range=[-0.5, 4.5], tickmode='linear', tick0=0, dtick=1),
            yaxis=dict(range=[-0.5, 8.5], tickmode='linear', tick0=0, dtick=1),
            legend_title="Win Probability"
        )
        
        html_out = f"{output_dir}/{filename_base}_v7.html"
        fig.write_html(html_out)
        print(f"Saved Interactive: {html_out}")

if __name__ == "__main__":
    analyze_bubbles_hand()
