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

# 3. Filter for Grand Games only
df = df[df['BestGame'] == 'Grand'].copy()
print("Filtered Grand Games:", len(df))

if len(df) == 0:
    print("No Grand games found.")
    exit()

# 4. Calculate Features
# Ensure numeric
cols_to_numeric = ['PMxLen', 'PCntJ', 'PAces', 'PAtt10', 'MaxProb']
for col in cols_to_numeric:
    df[col] = pd.to_numeric(df[col], errors='coerce')

# TotalTrumps proxy for Grand:
# "Effective Strength" = Jacks (PCntJ) + Longest Suit Length (PMxLen)
# Note: PMxLen in Grand is the longest side suit.
# This metric effectively measures "How many tricks can usually be made by establishing the long suit + Jacks".
df['TotalTrumps'] = df['PMxLen'] + df['PCntJ']
df['PSafeFulls'] = df['PAces'] + df['PAtt10']

print("TotalTrumps counts (Grand):")
print(df['TotalTrumps'].value_counts().sort_index())

# 5. Define Categories
# For Grand, we might see different distributions. Let's try 4, 5, 6, 7+ as well for comparability.
categories = [4, 5, 6, 7] 

output_dir = "research/plots"
os.makedirs(output_dir, exist_ok=True)

for trumps in categories:
    if trumps == 7:
        subset = df[df['TotalTrumps'] >= 7].copy()
        title_suffix = "7+ Effective Length (Jacks + Longest Suit)"
        filename = "bubble_grand_post_len_7_plus.png"
    else:
        subset = df[df['TotalTrumps'] == trumps].copy()
        title_suffix = f"{trumps} Effective Length (Jacks + Longest Suit)"
        filename = f"bubble_grand_post_len_{trumps}.png"
        
    if len(subset) == 0:
        print(f"No data for Grand Post-Discard: {title_suffix}")
        continue
        
    # Group by Signature
    grouped = subset.groupby(['PCntJ', 'PSafeFulls']).agg(
        Count=('MaxProb', 'count'),
        MeanWinRate=('MaxProb', 'mean')
    ).reset_index()
    
    if len(grouped) == 0:
        continue

    # Create Bubble Chart
    plt.figure(figsize=(10, 8))
    
    # Bubble sizes
    sizes = grouped['Count'] 
    
    if sizes.empty or sizes.sum() == 0:
        continue

    max_count = sizes.max()
    if max_count == 0: max_count = 1
    
    sizes_norm = (sizes / max_count) * 2000 + 100
    
    scatter = plt.scatter(
        x=grouped['PCntJ'], 
        y=grouped['PSafeFulls'], 
        s=sizes_norm, 
        c=grouped['MeanWinRate'], 
        cmap='RdYlGn', 
        alpha=0.7, 
        edgecolors='grey', 
        vmin=0.0, 
        vmax=1.0
    )
    
    # Add labels
    for i, row in grouped.iterrows():
        if row['Count'] > 20: # Lower threshold for Grand as data might be sparser
            plt.text(
                row['PCntJ'], 
                row['PSafeFulls'], 
                f"{row['MeanWinRate']:.0%}\n(n={row['Count']})", 
                ha='center', 
                va='center', 
                fontsize=9, 
                color='black',
                fontweight='bold'
            )

    # Colorbar
    cbar = plt.colorbar(scatter)
    cbar.set_label('Mean Max Win Probability')
    
    # Grid and Ticks
    plt.xticks(range(5)) # Jacks 0-4
    plt.yticks(range(int(grouped['PSafeFulls'].max()) + 2))
    
    plt.title(f"Grand Post-Discard: {title_suffix}\n(Win Prob based on Jacks vs. Post-Standing Fulls)", fontsize=16)
    plt.xlabel("Number of Jacks", fontsize=14)
    plt.ylabel("Standing Fulls (Aces + Tens with Ace)", fontsize=14)
    
    # Add annotation
    plt.figtext(0.5, 0.01, "Size represents frequency. Color represents Win Probability.", ha="center", fontsize=10, style='italic')
    
    output_path = os.path.join(output_dir, filename)
    plt.savefig(output_path, dpi=100, bbox_inches='tight')
    plt.close()
    print(f"Saved: {output_path}")

    # --- Interactve Plotly Chart ---
    try:
        import plotly.express as px
        
        # Format hover data
        grouped['Win Rate'] = grouped['MeanWinRate'].apply(lambda x: f"{x:.1%}")
        
        fig = px.scatter(
            grouped, 
            x='PCntJ', 
            y='PSafeFulls', 
            size='Count', 
            color='MeanWinRate',
            hover_name='Win Rate',
            hover_data={
                'PCntJ': True, 
                'PSafeFulls': True, 
                'Count': True, 
                'MeanWinRate': False, 
                'Win Rate': False 
            },
            title=f"Grand Post-Discard: {title_suffix}<br><sup>Win Prob based on Jacks vs. Standing Fulls. Size=Frequency.</sup>",
            color_continuous_scale='RdYlGn',
            range_color=[0, 1],
            size_max=60
        )
        
        fig.update_layout(
            xaxis_title="Number of Jacks",
            yaxis_title="Standing Fulls (Aces + Tens with Ace)",
            coloraxis_colorbar_title="Win Probability",
            template="plotly_white"
        )
        
        # Ticks
        fig.update_xaxes(dtick=1)
        fig.update_yaxes(dtick=1)
        
        html_filename = filename.replace('.png', '.html')
        html_output_path = os.path.join(output_dir, html_filename)
        fig.write_html(html_output_path)
        print(f"Saved Interactive: {html_output_path}")
        
    except ImportError:
        pass
    except Exception as e:
        print(f"Error creating interactive chart: {e}")

print("Done.")
