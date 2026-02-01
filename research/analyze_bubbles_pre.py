import pandas as pd
import matplotlib.pyplot as plt
import seaborn as sns
import os
import sys
import glob

# Set plot style for premium look
plt.style.use('seaborn-v0_8-darkgrid')
sns.set_context("notebook", font_scale=1.2)

def analyze_bubbles_pre():
    # 1. Find latest CSV
    list_of_files = glob.glob('research/data/general_pre_stats_final_50k_*.csv') 
    if not list_of_files:
        print("No CSV files found in research/data/")
        return
    latest_file = max(list_of_files, key=os.path.getctime)
    print(f"Processing: {latest_file}")

    try:
        df = pd.read_csv(latest_file)
    except Exception as e:
        print(f"Error reading CSV: {e}")
        return

    # Strip whitespace from columns
    df.columns = df.columns.str.strip()
    
    # 2. Define Metrics
    # X-Axis: CntJ (Jacks Count)
    # Y-Axis: SafeFulls (Aces + Att10)
    # Category: MxLen (4, 5, 6)
    
    # Ensure numeric
    cols_to_numeric = ['CntJ', 'Aces', 'Att10', 'MxLen', 'MaxProb']
    for col in cols_to_numeric:
        if col in df.columns:
            df[col] = pd.to_numeric(df[col], errors='coerce')
        else:
            print(f"Missing column: {col}")
            return
            
    # Calculate Safe Fulls (Stehende Volle)
    # Note: Att10 is "Attached Tens" (Tens with Ace). Aces are always "Standing".
    df['SafeFulls'] = df['Aces'] + df['Att10']
    
    # Calculate Potential Total Trumps
    # MxLen (Suit Length without Jacks) + CntJ (Jacks)
    df['PotentialTrumps'] = df['MxLen'] + df['CntJ']
    
    # Filter Categories: Total Trumps
    # We want charts for TotalTrumps = 4, 5, 6, 7+
    categories = [4, 5, 6, 7] 
    
    output_dir = "research/plots"
    os.makedirs(output_dir, exist_ok=True)
    
    for trumps in categories:
        if trumps == 7:
            subset = df[df['PotentialTrumps'] >= 7].copy()
            title_suffix = "7+ Potential Trumps"
            filename_base = "bubble_pre_trumps_7_plus"
        else:
            subset = df[df['PotentialTrumps'] == trumps].copy()
            title_suffix = f"{trumps} Potential Trumps"
            filename_base = f"bubble_pre_trumps_{trumps}"
            
        if len(subset) == 0:
            print(f"No data for {trumps} Trumps")
            continue
            
        # Group by Signature
        grouped = subset.groupby(['CntJ', 'SafeFulls']).agg(
            Count=('MaxProb', 'count'),
            MeanWinRate=('MaxProb', 'mean')
        ).reset_index()
        
        # DEBUG: Print count for 1 Jack, 0 SafeFulls
        if trumps == 4:
             debug_row = grouped[(grouped['CntJ'] == 1) & (grouped['SafeFulls'] == 0)]
             if not debug_row.empty:
                 print(f"DEBUG PRE (4 Trumps, 1J, 0Safe): n={debug_row['Count'].values[0]}")
        
        if len(grouped) == 0:
            continue

        # Create Bubble Chart
        plt.figure(figsize=(10, 8))
        
        # Bubble sizes
        sizes = grouped['Count'] 
        # Normalize size
        sizes_norm = (sizes / sizes.max()) * 2000 + 100
        
        # Strict Discontinuity with 4 Bins
        import matplotlib.colors as mcolors
        
        # Colors: Purple (0-50), Red (50-65), Yellow (65-75), Green (75-100)
        colors = ['#9C27B0', '#D32F2F', '#FFD700', '#388E3C'] # Purple, Red, Gold, Green
        bounds = [0.0, 0.50, 0.65, 0.75, 1.0]
        
        cmap = mcolors.ListedColormap(colors)
        norm = mcolors.BoundaryNorm(bounds, cmap.N)
        
        scatter = plt.scatter(
            x=grouped['CntJ'], 
            y=grouped['SafeFulls'], 
            s=sizes_norm, 
            c=grouped['MeanWinRate'], 
            cmap=cmap, 
            norm=norm,
            alpha=0.9, 
            edgecolors='black',
            linewidth=0.5
        )
        
        # Add labels to ALL bubbles
        for i, row in grouped.iterrows():
            # Choose text color based on background luminance roughly
            # Purple/Red/Green -> White text. Yellow -> Black text.
            win_rate = row['MeanWinRate']
            text_color = 'black' if (0.65 <= win_rate < 0.75) else 'white'
            
            label = f"{win_rate:.0%}"
            # Add count for bubbles that are at least 20% of the max frequency in this plot
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

        # Colorbar
        cbar = plt.colorbar(scatter, ticks=[0.25, 0.575, 0.7, 0.875])
        cbar.ax.set_yticklabels(['<50% (Purple)', '50-65% (Red)', '65-75% (Yel)', '>75% (Green)']) 
        cbar.set_label('Win Probability Category')
        
        # Grid and Ticks - STANDARDIZED V7
        plt.xticks(range(5)) # Jacks 0-4
        plt.yticks(range(9)) # SafeFulls 0-8
        
        plt.title(f"Pre-Discard Biddability: {title_suffix}", fontsize=16)
        plt.xlabel("Number of Jacks", fontsize=14)
        plt.ylabel("Safe Fulls (Aces + Tens with Ace)", fontsize=14)
        
        # Add annotation
        plt.figtext(0.5, 0.01, "Size represents frequency. Color represents Win Probability.", ha="center", fontsize=10, style='italic')
        plt.grid(True, linestyle='--', alpha=0.7)
        plt.xlim(-0.5, 4.5)
        plt.ylim(-0.5, 8.5) # Standardized to max possible (4 Aces + 4 Tens)

        out_path = f"{output_dir}/{filename_base}_v7.png"
        plt.savefig(out_path, dpi=100, bbox_inches='tight')
        plt.close()
        print(f"Saved: {out_path}")

        # --- Interactve Plotly Chart ---
        try:
            import plotly.express as px
            import numpy as np
            
            # Format hover data
            grouped['Win Rate'] = grouped['MeanWinRate'].apply(lambda x: f"{x:.1%}")
            
            # Discrete Colorscale for Plotly
            # We map the range 0-1 to specific colors
            custom_colorscale = [
                [0.0, '#9C27B0'],  # Purple
                [0.499, '#9C27B0'],
                [0.50, '#D32F2F'],  # Red
                [0.649, '#D32F2F'],
                [0.65, '#FFD700'],  # Yellow
                [0.749, '#FFD700'],
                [0.75, '#388E3C'],  # Green
                [1.0, '#388E3C']
            ]

            fig = px.scatter(
                grouped, 
                x='CntJ', 
                y='SafeFulls', 
                size='Count', 
                color='MeanWinRate',
                text='Win Rate', # Add text to bubbles
                hover_name='Win Rate',
                hover_data={
                    'CntJ': True, 
                    'SafeFulls': True, 
                    'Count': True, 
                    'MeanWinRate': False, 
                    'Win Rate': False 
                },
                title=f"Pre-Discard Biddability: {title_suffix}<br><sup>Purple <50% | Red 50-65% | Yellow 65-75% | Green >75%</sup>",
                color_continuous_scale=custom_colorscale,
                range_color=[0, 1],
                size_max=60
            )
            
            fig.update_layout(
                xaxis_title="Number of Jacks",
                yaxis_title="Standing Fulls (Aces + Tens with Ace)",
                coloraxis_colorbar_title="Win Probability",
                template="plotly_white"
            )
            
            # X-Axis Ticks (Integers)
            fig.update_xaxes(dtick=1)
            fig.update_yaxes(dtick=1)
            
            html_out = f"{output_dir}/{filename_base}_v7.html"
            fig.write_html(html_out)
            print(f"Saved Interactive: {html_out}")
            
        except ImportError:
            print("Plotly not installed. Skipping interactive chart.")
        except Exception as e:
            print(f"Error creating interactive chart: {e}")

if __name__ == "__main__":
    analyze_bubbles_pre()
