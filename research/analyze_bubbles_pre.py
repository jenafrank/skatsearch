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
        # ... logic continues ...
            subset = df[df['PotentialTrumps'] >= 7].copy()
            title_suffix = "7+ Potential Trumps"
            filename = "bubble_pre_trumps_7_plus.png"
        else:
            subset = df[df['PotentialTrumps'] == trumps].copy()
            title_suffix = f"{trumps} Potential Trumps"
            filename = f"bubble_pre_trumps_{trumps}.png"
            
        if len(subset) == 0:
            print(f"No data for {trumps} Trumps")
            continue
            
        # Group by Signature
        grouped = subset.groupby(['CntJ', 'SafeFulls']).agg(
            Count=('MaxProb', 'count'),
            MeanWinRate=('MaxProb', 'mean')
        ).reset_index()
        
        if len(grouped) == 0:
            continue

        # Create Bubble Chart
        plt.figure(figsize=(10, 8))
        
        # Bubble sizes
        sizes = grouped['Count'] 
        # Normalize size
        sizes_norm = (sizes / sizes.max()) * 2000 + 100
        
        scatter = plt.scatter(
            x=grouped['CntJ'], 
            y=grouped['SafeFulls'], 
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
            if row['Count'] > 50: 
                plt.text(
                    row['CntJ'], 
                    row['SafeFulls'], 
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
        plt.yticks(range(int(grouped['SafeFulls'].max()) + 2))
        
        plt.title(f"Pre-Discard Biddability: {title_suffix}\n(Win Prob based on Jacks vs. Standing Fulls)", fontsize=16)
        plt.xlabel("Number of Jacks", fontsize=14)
        plt.ylabel("Standing Fulls (Aces + Tens with Ace)", fontsize=14)
        
        # Add annotation
        plt.figtext(0.5, 0.01, f"Category: {title_suffix} (Max Suit Length + Jacks). Size = Frequency.", ha="center", fontsize=10, style='italic')
        
        output_path = os.path.join(output_dir, filename)
        plt.savefig(output_path, dpi=100, bbox_inches='tight')
        print(f"Saved: {output_path}")
        plt.close()

        # --- Interactve Plotly Chart ---
        try:
            import plotly.express as px
            
            # Format hover data
            grouped['Win Rate'] = grouped['MeanWinRate'].apply(lambda x: f"{x:.1%}")
            
            fig = px.scatter(
                grouped, 
                x='CntJ', 
                y='SafeFulls', 
                size='Count', 
                color='MeanWinRate',
                hover_name='Win Rate',
                hover_data={
                    'CntJ': True, 
                    'SafeFulls': True, 
                    'Count': True, 
                    'MeanWinRate': False, # Shown in hover_name
                    'Win Rate': False # Duplicate
                },
                title=f"Pre-Discard Biddability: {title_suffix}<br><sup>Win Prob based on Jacks vs. Standing Fulls. Size=Frequency.</sup>",
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
            
            # X-Axis Ticks (Integers)
            fig.update_xaxes(dtick=1)
            fig.update_yaxes(dtick=1)
            
            html_filename = filename.replace('.png', '.html')
            html_output_path = os.path.join(output_dir, html_filename)
            fig.write_html(html_output_path)
            print(f"Saved Interactive: {html_output_path}")
            
        except ImportError:
            print("Plotly not installed. Skipping interactive chart.")
        except Exception as e:
            print(f"Error creating interactive chart: {e}")

if __name__ == "__main__":
    analyze_bubbles_pre()
