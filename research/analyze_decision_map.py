
import pandas as pd
import matplotlib.pyplot as plt
import seaborn as sns
import os
import sys
import glob
import matplotlib.colors as mcolors
import plotly.express as px
import numpy as np

# Set plot style for premium look
plt.style.use('seaborn-v0_8-darkgrid')
sns.set_context("notebook", font_scale=1.2)

def analyze_decision_map():
    # --- 1. Load Pre-Discard Data (Pickup Probability) ---
    # Find latest general_pre_stats
    search_pattern = "research/data/general_pre_stats_final_50k_*.csv"
    files = glob.glob(search_pattern)
    if not files:
        print("No Pre-Discard data found.")
        return
    pre_file = max(files, key=os.path.getctime)
    print(f"Loading Pre-Discard Data (Pickup): {pre_file}")
    
    try:
        df_pre = pd.read_csv(pre_file)
        df_pre.columns = df_pre.columns.str.strip()
    except Exception as e:
        print(f"Error reading Pre CSV: {e}")
        return

    # Metrics for Pre
    # 'MaxProb' is generally the win rate if you play the best game after pickup
    if 'MaxProb' in df_pre.columns:
        pickup_col = 'MaxProb'
    elif 'WinProb' in df_pre.columns:
        pickup_col = 'WinProb' # Often the same
    else:
        print("No WinProb/MaxProb in Pre Data")
        return

    df_pre['SafeFulls'] = df_pre['Aces'] + df_pre['Att10']
    df_pre['Trumps'] = df_pre['MxLen'] + df_pre['CntJ']
    
    # Aggregating Pre Data
    # We need the Frequency count from here as it's more representative
    pre_grouped = df_pre.groupby(['Trumps', 'CntJ', 'SafeFulls']).agg(
        ProbPickup=(pickup_col, 'mean'),
        Count=('CntJ', 'count')
    ).reset_index()

    # --- 2. Load Hand Data (Suit Hand / Grand Hand) ---
    hand_file = 'research/data/hand_best_game_cluster.csv'
    if not os.path.exists(hand_file):
        print("No Hand Data found.")
        return
    print(f"Loading Hand Data: {hand_file}")
    
    try:
        df_hand = pd.read_csv(hand_file)
        df_hand.columns = df_hand.columns.str.strip()
    except Exception as e:
        print(f"Error reading Hand CSV: {e}")
        return

    # Metrics for Hand
    suit_cols = ['ProbClubs', 'ProbSpades', 'ProbHearts', 'ProbDiamonds']
    # Ensure they exist
    for c in suit_cols + ['ProbGrand', 'Aces', 'Att10', 'MxLen', 'CntJ']:
        df_hand[c] = pd.to_numeric(df_hand[c], errors='coerce')
        
    df_hand['ProbSuitHand'] = df_hand[suit_cols].max(axis=1)
    df_hand['ProbGrandHand'] = df_hand['ProbGrand']
    df_hand['SafeFulls'] = df_hand['Aces'] + df_hand['Att10']
    df_hand['Trumps'] = df_hand['MxLen'] + df_hand['CntJ']
    
    # Aggregating Hand Data
    # Note: Hand data is cluster-based, so counts are artificial. We only care about Probs.
    hand_grouped = df_hand.groupby(['Trumps', 'CntJ', 'SafeFulls']).agg(
        ProbSuitHand=('ProbSuitHand', 'mean'),
        ProbGrandHand=('ProbGrandHand', 'mean')
    ).reset_index()

    # --- 3. Merge ---
    merged = pd.merge(pre_grouped, hand_grouped, on=['Trumps', 'CntJ', 'SafeFulls'], how='inner')
    
    # --- 4. Logic & Plotting ---
    categories = [4, 5, 6, 7]
    output_dir = "research/plots"
    os.makedirs(output_dir, exist_ok=True)
    
    # Decision Colors
    # Grand Hand: Blue
    # Suit Hand: Orange
    # Pickup: Green
    decision_colors = {
        'Grand Hand': '#1E88E5',   # Blue
        'Suit Hand': '#FB8C00',    # Orange
        'Pickup Skat': '#43A047'   # Green
    }
    
    for trumps in categories:
        # User requested strictly 7 trumps, ignoring 8+
        subset = merged[merged['Trumps'] == trumps].copy()
        
        if trumps == 7:
            title_suffix = "7 Trumps"
            filename_base = "bubble_decision_map_trumps_7_v10"
        else:
            title_suffix = f"{trumps} Trumps"
            filename_base = f"bubble_decision_map_trumps_{trumps}_v10"
            
        if len(subset) == 0:
            continue
            
        plt.figure(figsize=(12, 10))
        
        # Lists for Scatter
        scatter_x = []
        scatter_y = []
        scatter_s = []
        scatter_c = []
        
        # Determine Decision based on Expected Value (EV)
        # Points:
        # Grand Hand: Win=+1.33, Loss=-1.33 => EV = 1.33*P - 1.33*(1-P) = 2.66P - 1.33 (User Request)
        # Suit Hand:  Win=+1, Loss=-1  => EV = 1*P - 1*(1-P) = 2P - 1
        # Pickup Skat: Win=+1, Loss=-2 => EV = 1*P - 2*(1-P) = 3P - 2
        
        def get_decision(row):
            p_g = row['ProbGrandHand']
            p_s = row['ProbSuitHand']
            p_p = row['ProbPickup']
            
            ev_g = 2.66 * p_g - 1.33
            ev_s = 2 * p_s - 1
            ev_p = 3 * p_p - 2
            
            # Must have positive expectation to play
            if max(ev_g, ev_s, ev_p) <= 0:
                return None
                
            # Choose Strategy with Max EV
            best_ev = max(ev_g, ev_s, ev_p)
            
            if best_ev == ev_g: return 'Grand Hand'
            if best_ev == ev_s: return 'Suit Hand'
            return 'Pickup Skat'

        subset['Decision'] = subset.apply(get_decision, axis=1)
        
        # Filter out None (Negative EV)
        subset = subset.dropna(subset=['Decision'])
        
        if len(subset) == 0:
            print(f"No positive EV hands for {title_suffix}")
            continue

        # Normalizing Size
        max_count = subset['Count'].max()
        if max_count == 0: max_count = 1
        
        # Plotting
        for i, row in subset.iterrows():
            decision = row['Decision']
            color = decision_colors[decision]
            
            # Size scaling
            size = (row['Count'] / max_count) * 2500 + 300
            
            plt.scatter(
                row['CntJ'], 
                row['SafeFulls'], 
                s=size, 
                color=color, 
                alpha=0.6, 
                edgecolors='w',
                linewidth=2
            )
            
            # Text Inside: Show Probabilities
            p_p_val = int(row['ProbPickup'] * 100)
            p_s_val = int(row['ProbSuitHand'] * 100)
            p_g_val = int(row['ProbGrandHand'] * 100)
            
            lines = []
            # Order by priority/relevance
            if decision == 'Grand Hand':
                lines.append(f"G: {p_g_val}%")
                lines.append(f"S: {p_s_val}%")
                lines.append(f"P: {p_p_val}%")
            elif decision == 'Suit Hand':
                lines.append(f"S: {p_s_val}%")
                lines.append(f"G: {p_g_val}%")
                lines.append(f"P: {p_p_val}%")
            else:
                lines.append(f"P: {p_p_val}%")
                lines.append(f"S: {p_s_val}%")
                lines.append(f"G: {p_g_val}%")
                
            label_text = "\n".join(lines)
            
            plt.text(
                row['CntJ'], 
                row['SafeFulls'], 
                label_text, 
                ha='center', 
                va='center', 
                fontsize=9, 
                color='black', 
                fontweight='bold',
                linespacing=1.2
            )

        # Legend
        from matplotlib.patches import Patch
        legend_elements = [
            Patch(facecolor='#1E88E5', edgecolor='white', label='Grand Hand (Max EV)'),
            Patch(facecolor='#FB8C00', edgecolor='white', label='Suit Hand (Max EV)'),
            Patch(facecolor='#43A047', edgecolor='white', label='Pickup Skat (Max EV)')
        ]
        plt.legend(handles=legend_elements, loc='upper right', title="Optimal Strategy", frameon=True)

        # Axes / Grid & Formula Annotation
        plt.title(f"Decision Map (Expected Value): {title_suffix}", fontsize=18, pad=20)
        
        # Add Formula Annotation
        formula_text = (
            "EV Formula:\n"
            "G_Hand = 2.66*P - 1.33\n"
            "S_Hand = 2*P - 1\n"
            "Pickup = 3*P - 2"
        )
        plt.figtext(0.15, 0.82, formula_text, fontsize=10, 
                   bbox=dict(facecolor='white', alpha=0.8, edgecolor='gray'))

        plt.xlabel("Jack Count (CntJ)", fontsize=14)
        plt.ylabel("Side Strength (Safe Fulls)", fontsize=14)
        
        plt.xticks(range(5)) 
        plt.yticks(range(9))
        plt.grid(True, linestyle='--', alpha=0.6)
        plt.ylim(-0.5, 8.5)
        plt.xlim(-0.5, 4.5)
        
        # Save
        out_path = f"{output_dir}/{filename_base}.png"
        plt.savefig(out_path, dpi=100, bbox_inches='tight')
        plt.close()
        print(f"Saved: {out_path}")
        
        # --- Interactive Plotly ---
        # Map Decision to Colors
        color_map = {
            'Grand Hand': '#1E88E5',
            'Suit Hand': '#FB8C00',
            'Pickup Skat': '#43A047' 
        }
        
        fig = px.scatter(
            subset,
            x='CntJ',
            y='SafeFulls',
            size='Count',
            color='Decision',
            color_discrete_map=color_map,
            hover_name='Decision',
            hover_data={
                'CntJ': True, 
                'SafeFulls': True, 
                'ProbGrandHand': ':.1%',
                'ProbSuitHand': ':.1%',
                'ProbPickup': ':.1%',
                'Count': True,
                'Decision': False
            },
            size_max=60,
            title=f"Decision Map (EV): {title_suffix}",
            template="plotly_white"
        )
        
        # Label with Winner %
        def get_label(r):
            if r['Decision'] == 'Grand Hand': return f"G:{r['ProbGrandHand']:.0%}"
            if r['Decision'] == 'Suit Hand': return f"S:{r['ProbSuitHand']:.0%}"
            return f"P:{r['ProbPickup']:.0%}"
            
        subset['Label'] = subset.apply(get_label, axis=1)
        
        fig.update_traces(text=subset['Label'], textposition='middle center', textfont=dict(color='black'))
        
        # Add annotation to interactive chart?
        fig.add_annotation(
            text="EV: GH(2.66P-1.33), SH(2P-1), PS(3P-2)",
            xref="paper", yref="paper",
            x=0.01, y=0.99, showarrow=False,
            bgcolor="white", bordercolor="gray"
        )

        fig.update_layout(
            xaxis_title="Jack Count",
            yaxis_title="Side Strength (Safe Fulls)",
            xaxis=dict(range=[-0.5, 4.5], tickmode='linear', tick0=0, dtick=1),
            yaxis=dict(range=[-0.5, 8.5], tickmode='linear', tick0=0, dtick=1),
            legend_title="Optimal Strategy"
        )
        
        html_out = f"{output_dir}/{filename_base}.html"
        fig.write_html(html_out)
        print(f"Saved Interactive: {html_out}")

if __name__ == "__main__":
    analyze_decision_map()
