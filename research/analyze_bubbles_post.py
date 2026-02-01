import pandas as pd
import matplotlib.pyplot as plt
import seaborn as sns
import glob
import os

# Set style
plt.style.use('seaborn-v0_8-darkgrid')
sns.set_context("notebook", font_scale=1.2)

# 1. Find latest CSV
list_of_files = glob.glob('research/data/general_pre_stats_final_50k_*.csv')
if not list_of_files:
    print("No CSV files found.")
    exit()
latest_file = max(list_of_files, key=os.path.getctime)
print(f"Processing: {latest_file}")

# 2. Load Data
try:
    df = pd.read_csv(latest_file)
except pd.errors.ParserError:
    df = pd.read_csv(latest_file, on_bad_lines='skip')

# Strip whitespace from columns
df.columns = df.columns.str.strip()

# Strip whitespace from BestGame values
if 'BestGame' in df.columns:
    df['BestGame'] = df['BestGame'].astype(str).str.strip()

# 3. NO Filtering by BestGame - We want Suit Potential for ALL hands
    # suit_games = ['Clubs', 'Spades', 'Hearts', 'Diamonds']
    # df = df[df['BestGame'].isin(suit_games)].copy()
    print("Processing ALL hands for Suit Potential:", len(df))
    
    # 4. Calculate Features
    # Ensure numeric for Suit Probs
    suit_cols = ['ProbClubs', 'ProbSpades', 'ProbHearts', 'ProbDiamonds']
    cols_to_numeric = ['PMxLen', 'PCntJ', 'PAces', 'PAtt10'] + suit_cols
    
    for col in cols_to_numeric:
        if col in df.columns:
            df[col] = pd.to_numeric(df[col], errors='coerce')
            
    # Calculate Max Suit Probability for each hand (Best Suit Game)
    df['WinProb'] = df[suit_cols].max(axis=1)

    df['TotalTrumps'] = df['PMxLen'] + df['PCntJ']
    df['PSafeFulls'] = df['PAces'] + df['PAtt10']
    
    print("TotalTrumps counts:")
    print(df['TotalTrumps'].value_counts().sort_index())
    
    # 5. Define Categories
    categories = [4, 5, 6, 7] # 4, 5, 6, 7+ 
    
    output_dir = "research/plots"
os.makedirs(output_dir, exist_ok=True)

for trumps in categories:
    if trumps == 7:
        subset = df[df['TotalTrumps'] >= 7].copy()
        title_suffix = "7+ Total Trumps (Post)"
        filename = "bubble_post_trumps_7_plus_v7.png"
    else:
        subset = df[df['TotalTrumps'] == trumps].copy()
        title_suffix = f"{trumps} Total Trumps (Post)"
        filename = f"bubble_post_trumps_{trumps}_v7.png"
        
    if len(subset) == 0:
        print(f"No data for Post-Discard: {title_suffix}")
        continue
        
    # Group by Signature
    # Group by Signature
    grouped = subset.groupby(['PCntJ', 'PSafeFulls']).agg(
        Count=('WinProb', 'count'),
        MeanWinRate=('WinProb', 'mean')
    ).reset_index()
    
    if len(grouped) == 0:
        continue

    # Create Bubble Chart
    plt.figure(figsize=(10, 8))
    
    # Bubble sizes
    sizes = grouped['Count'] 
    
    # Check if sizes is empty or sum is 0
    if sizes.empty or sizes.sum() == 0:
        continue

    # Normalize size
    max_count = sizes.max()
    if max_count == 0: max_count = 1
    
    sizes_norm = (sizes / max_count) * 2000 + 100
    
    
    # Strict Discontinuity with 4 Bins
    import matplotlib.colors as mcolors
    
    # Colors: Purple (0-50), Red (50-65), Yellow (65-75), Green (75-100)
    colors = ['#9C27B0', '#D32F2F', '#FFD700', '#388E3C'] # Purple, Red, Gold, Green
    bounds = [0.0, 0.50, 0.65, 0.75, 1.0]
    
    cmap = mcolors.ListedColormap(colors)
    norm = mcolors.BoundaryNorm(bounds, cmap.N)

    scatter = plt.scatter(
        x=grouped['PCntJ'], 
        y=grouped['PSafeFulls'], 
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
        # Choose text color based on background luminance
        win_rate = row['MeanWinRate']
        text_color = 'black' if (0.65 <= win_rate < 0.75) else 'white'
        
        label = f"{win_rate:.0%}"
        if row['Count'] > grouped['Count'].max() * 0.2:
            label += f"\n(n={int(row['Count'])})"
        
        plt.text(
            row['PCntJ'], 
            row['PSafeFulls'], 
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
    
    # Grid and Ticks
    plt.xticks(range(5)) # Jacks 0-4
    plt.yticks(range(int(grouped['PSafeFulls'].max()) + 2))
    
    plt.title(f"Post-Discard Strength: {title_suffix}\n(Purple <50 | Red <65 | Yellow <75 | Green >75)", fontsize=16)
    plt.xlabel("Number of Jacks", fontsize=14)
    plt.ylabel("Standing Fulls (Aces + Tens with Ace)", fontsize=14)
    
    # Add annotation
    plt.figtext(0.5, 0.01, "Size represents frequency. Color represents Win Probability (Green > 66%, Red < 33%)", ha="center", fontsize=10, style='italic')
    plt.figtext(0.5, 0.01, "Size represents frequency (normalized). Color is Win Rate.", ha="center", fontsize=10, style='italic')
    plt.grid(True, linestyle='--', alpha=0.7)
    plt.xlim(-0.5, 4.5)
    plt.ylim(-0.5, 8.5)
    
    output_path = os.path.join(output_dir, filename)
    plt.savefig(output_path, dpi=100, bbox_inches='tight')
    plt.close()
    print(f"Saved: {output_path}")

    # --- Interactve Plotly Chart ---
    try:
        import plotly.express as px
        
        # Format hover data
        grouped['Win Rate'] = grouped['MeanWinRate'].apply(lambda x: f"{x:.1%}")
        
        # Discrete Colorscale for Plotly
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
            x='PCntJ', 
            y='PSafeFulls', 
            size='Count', 
            color='MeanWinRate',
            text='Win Rate', 
            hover_name='Win Rate',
            hover_data={
                'PCntJ': True, 
                'PSafeFulls': True, 
                'Count': True, 
                'MeanWinRate': False, 
                'Win Rate': False 
            },
            title=f"Post-Discard Strength: {title_suffix}<br><sup>Purple <50% | Red 50-65% | Yellow 65-75% | Green >75%</sup>",
            color_continuous_scale=custom_colorscale,
            range_color=[0, 1],
            size_max=60
        )
        
        fig.update_layout(
            xaxis_title="Number of Jacks",
            yaxis_title="Standing Fulls (Aces + Tens with Ace)",
            coloraxis_colorbar_title="Win Probability",
            template="plotly_white",
            xaxis=dict(range=[-0.5, 4.5], tickmode='linear', tick0=0, dtick=1),
            yaxis=dict(range=[-0.5, 8.5], tickmode='linear', tick0=0, dtick=1)
        )
        
        html_filename = filename.replace('.png', '.html')
        html_output_path = os.path.join(output_dir, html_filename)
        fig.write_html(html_output_path)
        print(f"Saved Interactive: {html_output_path}")
        
    except ImportError:
        pass
    except Exception as e:
        print(f"Error creating interactive chart: {e}")
    print(f"Saved: {output_path}")
    plt.close()

print("Done.")
