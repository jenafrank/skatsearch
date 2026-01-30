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
except:
    df = pd.read_csv(latest_file, on_bad_lines='skip')

# Cleanup
df.columns = df.columns.str.strip()
if 'BestGame' in df.columns:
    df['BestGame'] = df['BestGame'].astype(str).str.strip()

# Filter Suit Games
suit_games = ['Clubs', 'Spades', 'Hearts', 'Diamonds']
df = df[df['BestGame'].isin(suit_games)].copy()
print("Suit Games:", len(df))

# 3. Calculate Signature Metrics
# Pre
df['PreJacks'] = pd.to_numeric(df['CntJ'], errors='coerce')
df['PreSafeFulls'] = pd.to_numeric(df['Aces'] + df['Att10'], errors='coerce')
df['PreTotalTrumps'] = df['PreJacks'] + pd.to_numeric(df['MxLen'], errors='coerce')

# Post
df['PostJacks'] = pd.to_numeric(df['PCntJ'], errors='coerce')
df['PostSafeFulls'] = pd.to_numeric(df['PAces'] + df['PAtt10'], errors='coerce')
df['PostTotalTrumps'] = df['PostJacks'] + pd.to_numeric(df['PMxLen'], errors='coerce')

# Ensure probability matches Pre-Discard (MaxProb)
df['WinProb'] = pd.to_numeric(df['MaxProb'], errors='coerce')

# 4. Group by Signature
# Signature = (TotalTrumps, Jacks, SafeFulls)

# Pre-Discard Stats
pre_grouped = df.groupby(['PreTotalTrumps', 'PreJacks', 'PreSafeFulls']).agg(
    PreMeanProb=('WinProb', 'mean'),
    PreCount=('WinProb', 'count')
).reset_index()

pre_grouped.rename(columns={
    'PreTotalTrumps': 'TotalTrumps', 
    'PreJacks': 'Jacks', 
    'PreSafeFulls': 'SafeFulls'
}, inplace=True)

# Post-Discard Stats ("Stagnation Outcome Estimate")
# We calculate Mean(WinProb) for hands ENDING in signature S.
post_grouped = df.groupby(['PostTotalTrumps', 'PostJacks', 'PostSafeFulls']).agg(
    PostMeanProb=('WinProb', 'mean'), # Note: WinProb is still Pre-Prob, but averaged over hands LANDING here
    PostCount=('WinProb', 'count')
).reset_index()

post_grouped.rename(columns={
    'PostTotalTrumps': 'TotalTrumps', 
    'PostJacks': 'Jacks', 
    'PostSafeFulls': 'SafeFulls'
}, inplace=True)

# 5. Merge
merged = pd.merge(pre_grouped, post_grouped, on=['TotalTrumps', 'Jacks', 'SafeFulls'], how='inner')

# 6. Identify "Traps" (Pre >= 66% AND Post < 66%)
traps = merged[
    (merged['PreMeanProb'] >= 0.66) & 
    (merged['PostMeanProb'] < 0.66) &
    (merged['PreCount'] > 20) # Filter for significance
].copy()

traps['ProbLoss'] = traps['PreMeanProb'] - traps['PostMeanProb']
traps = traps.sort_values(by='ProbLoss', ascending=False)

print("\n=== TRAPPY SIGNATURES (Pre > 66%, Post < 66%) ===")
if len(traps) > 0:
    print(traps[['TotalTrumps', 'Jacks', 'SafeFulls', 'PreMeanProb', 'PostMeanProb', 'ProbLoss']].to_string(index=False))
else:
    print("No trappy signatures found with current filters.")

# 7. Visualize Shift
# Plot Pre vs Post for relevant signatures (Pre > 50%)
relevant = merged[merged['PreMeanProb'] > 0.5].copy()

if len(relevant) > 0:
    plt.figure(figsize=(10, 10))
    sns.scatterplot(
        data=relevant, 
        x='PreMeanProb', 
        y='PostMeanProb', 
        hue='TotalTrumps', 
        size='PreCount', 
        sizes=(50, 500),
        palette='viridis',
        alpha=0.7
    )
    
    # Reference Line x=y
    plt.plot([0.5, 1.0], [0.5, 1.0], color='red', linestyle='--', label='No Loss')
    # Threshold Lines
    plt.axvline(x=0.66, color='green', linestyle=':', label='Pre-Win Threshold')
    plt.axhline(y=0.66, color='orange', linestyle=':', label='Post-Win Threshold')
    
    # Highlight Traps
    if len(traps) > 0:
        plt.scatter(
            traps['PreMeanProb'], 
            traps['PostMeanProb'], 
            color='red', 
            s=100, 
            marker='x', 
            label='Traps'
        )
        # Annotate top traps
        for i, row in traps.head(5).iterrows():
            plt.text(
                row['PreMeanProb'], 
                row['PostMeanProb'] - 0.02, 
                f"{int(row['TotalTrumps'])}T/{int(row['Jacks'])}J/{int(row['SafeFulls'])}F", 
                fontsize=8, 
                color='red',
                ha='center'
            )

    plt.title("Pre-Discard Expectation vs. Post-Discard Reality\n(Traps: Pre > 66% -> Post < 66%)", fontsize=16)
    plt.xlabel("Pre-Discard Mean WinProb (Expected)", fontsize=14)
    plt.ylabel("Post-Discard Mean WinProb (Realized by Stagnant Hands)", fontsize=14)
    plt.legend()
    
    
    # 4-Zone Color Palette for Scatter
    # Purple (<0.50), Red (0.50-0.65), Yellow (0.65-0.75), Green (>0.75)
    # Since hue is 'TotalTrumps', we should keep that but maybe color the bubbles by WinProb?
    # The user asked for bubble colors. However, this chart maps Pre vs Post.
    # Color represents "Total Trumps" in this chart. 
    # But maybe we should change the color to represent the DELTA or the PostProb?
    # The user request "Die Bubble-Grafiken" likely refers to the main 3 bubble charts.
    # The Stagnation chart is a different type (Scatter Pre vs Post). 
    # Proceeding to keep TotalTrumps as Hue but maybe add distinct colors for consistency if possible.
    # For now, let's stick to Viridis for this scientific chart unless explicitly requested.
    # Wait, "Die Bubble-Grafiken" probably implies all of them.
    # But for this chart, X/Y axes are Probabilities. 
    # Let's keep it scientifically clean but update the HTML interactive version to maybe use the color zones if applicable?
    # Actually, let's assume the user meant the "Biddability/Strength" charts.
    # I will leave this one as is for now to avoid confusion, or maybe just update the interactive tooltip.
    
    out_path = "research/plots/stagnation_traps.png"
    plt.savefig(out_path, dpi=100)
    plt.close()
    print(f"Saved: {out_path}")

    # --- Interactive Plotly Chart ---
    try:
        import plotly.express as px
        
        # Create Signature String for Hover
        relevant['Signature'] = relevant.apply(
            lambda r: f"{int(r['TotalTrumps'])}T / {int(r['Jacks'])}J / {int(r['SafeFulls'])}F", axis=1
        )
        relevant['Prob Loss'] = (relevant['PreMeanProb'] - relevant['PostMeanProb']).apply(lambda x: f"{x:.1%}")
        relevant['Pre Win%'] = relevant['PreMeanProb'].apply(lambda x: f"{x:.1%}")
        relevant['Post Win%'] = relevant['PostMeanProb'].apply(lambda x: f"{x:.1%}")

        fig = px.scatter(
            relevant, 
            x='PreMeanProb', 
            y='PostMeanProb', 
            color='TotalTrumps',
            size='PreCount',
            hover_name='Signature',
            hover_data={
                'PreMeanProb': False,
                'PostMeanProb': False,
                'TotalTrumps': False,
                'PreCount': True,
                'Pre Win%': True,
                'Post Win%': True,
                'Prob Loss': True
            },
            title="Pre-Discard Expectation vs. Post-Discard Reality<br><sup>Traps: High Pre-Win% but Low Post-Win% (if Skat doesn't improve)</sup>",
            labels={
                "PreMeanProb": "Pre-Discard Mean WinProb (Expected)", 
                "PostMeanProb": "Post-Discard Mean WinProb (Realized)",
                "TotalTrumps": "Total Trumps (Color Code)"
            },
            range_x=[0.5, 1.0],
            range_y=[0.2, 1.0],
            size_max=50,
            template="plotly_white"
        )
        
        # Add Reference Line x=y
        fig.add_shape(type="line", x0=0.5, y0=0.5, x1=1, y1=1, line=dict(color="Red", width=2, dash="dash"))
        
        # Add Thresholds
        fig.add_shape(type="line", x0=0.66, y0=0.2, x1=0.66, y1=1, line=dict(color="Green", width=1, dash="dot"))
        fig.add_shape(type="line", x0=0.5, y0=0.66, x1=1, y1=0.66, line=dict(color="Orange", width=1, dash="dot"))

        # Highlight Traps (Separate trace)
        if len(traps) > 0:
             # Add annotations for top traps
            for i, row in traps.head(5).iterrows():
                fig.add_annotation(
                    x=row['PreMeanProb'],
                    y=row['PostMeanProb'],
                    text=f"{int(row['TotalTrumps'])}T/{int(row['Jacks'])}J",
                    showarrow=True,
                    arrowhead=1
                )

        html_output_path = "research/plots/stagnation_traps.html"
        fig.write_html(html_output_path)
        print(f"Saved Interactive: {html_output_path}")

    except ImportError:
        pass
    except Exception as e:
        print(f"Error creating interactive chart: {e}")
