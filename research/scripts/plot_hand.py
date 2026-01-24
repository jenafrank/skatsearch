import pandas as pd
import matplotlib.pyplot as plt
import numpy as np
import os

import pandas as pd
import matplotlib.pyplot as plt
import numpy as np
import os

def plot_hand_analysis(input_file=None):
    script_dir = os.path.dirname(os.path.abspath(__file__))
    if input_file:
        data_file = input_file
    else:
        data_file = os.path.join(script_dir, "../data/hand_best_game_50k.csv")
    output_dir = os.path.join(script_dir, "../plots")
    
    if not os.path.exists(data_file):
        print(f"Data file not found: {data_file}")
        return

    if not os.path.exists(output_dir):
        os.makedirs(output_dir)

    print(f"Loading data from {data_file}...")
    try:
        df = pd.read_csv(data_file)
    except Exception as e:
        print(f"Error reading CSV: {e}")
        return
    
    # Clean whitespace in headers
    df.columns = df.columns.str.strip()
    
    # Filter for Suit Games (best game is not Null/Grand/Unknown)
    suit_games = ['Clubs', 'Spades', 'Hearts', 'Diamonds']
    df_suit = df[df['BestGame'].str.strip().isin(suit_games)].copy()
    
    print(f"Suit Games: {len(df_suit)} / {len(df)}")

    # Calculate Metrics for Suit Games
    # CntJ is in CSV.
    # MxLen is Max Suit Length (which for the Best Suit Game is likely the Trump Suit Length).
    # TotalTrumpLength = CntJ + MxLen.
    
    df_suit['CntJ'] = pd.to_numeric(df_suit['CntJ'], errors='coerce').fillna(0)
    df_suit['MxLen'] = pd.to_numeric(df_suit['MxLen'], errors='coerce').fillna(0)
    df_suit['TotalTrumpLength'] = df_suit['CntJ'] + df_suit['MxLen']
    
    # Y Axis: Standing Fulls = Aces + AttachedTens
    # CSV has Aces, Att10
    df_suit['Aces'] = pd.to_numeric(df_suit['Aces'], errors='coerce').fillna(0)
    df_suit['Att10'] = pd.to_numeric(df_suit['Att10'], errors='coerce').fillna(0)
    df_suit['Y'] = df_suit['Aces'] + df_suit['Att10']
    
    df_suit['WinProb'] = pd.to_numeric(df_suit['WinProb'], errors='coerce').fillna(0)
    
    # Slices: Total Trump Length 4, 5, 6, 7, 8
    slices = [4, 5, 6, 7, 8]
    
    for sl in slices:
        subset = df_suit[df_suit['TotalTrumpLength'] == sl]
        if subset.empty:
            print(f"No data for Total Trump Length {sl}")
            continue
            
        print(f"Plotting Suit Slice: Total Trump Length {sl} (N={len(subset)})")
        
        # Aggregating
        grouped = subset.groupby(['CntJ', 'Y']).agg({
            'WinProb': 'mean',
            'Hand': 'count'
        }).reset_index()
        
        # Bubble Sizes
        sizes = grouped['WinProb'] * 1000
        
        # Centroid
        total_prob = subset['WinProb'].sum()
        if total_prob > 0:
            cx = (subset['CntJ'] * subset['WinProb']).sum() / total_prob
            cy = (subset['Y'] * subset['WinProb']).sum() / total_prob
        else:
            cx, cy = 0, 0
            
        plt.figure(figsize=(10, 8))
        
        # Scatter with color map based on WinProb? Or just fixed color?
        plt.scatter(grouped['CntJ'], grouped['Y'], s=sizes, alpha=0.6, c='purple', edgecolors='black', label='Hand Strength')
        
        if total_prob > 0:
            plt.scatter([cx], [cy], color='red', s=200, marker='X', label=f'Schwerpunkt ({cx:.2f}, {cy:.2f})')
        
        for _, row in grouped.iterrows():
            if row['WinProb'] > 0.01:
                pct = int(row['WinProb'] * 100)
                plt.annotate(f"{pct}%", (row['CntJ'], row['Y']), 
                             ha='center', va='center', color='white', weight='bold', size=9)

        plt.title(f"Best Hand Game (Suit) Strength (Trumpflänge {sl})\nN={len(subset)} - Schwerpunkt: Jacks={cx:.2f}, Fulls={cy:.2f}")
        plt.xlabel("Anzahl Buben")
        plt.ylabel("Stehende Volle (Asse + 10)")
        plt.grid(True, linestyle='--', alpha=0.5)
        plt.legend(loc='upper left')
        plt.xticks(range(0, 5)) 
        plt.yticks(range(0, 7))
        plt.ylim(-0.5, 6.5)
        
        out_file = f"{output_dir}/hand_best_suit_len_{sl}.png"
        plt.savefig(out_file)
        print(f"  Saved {out_file}")
        plt.close()

    # -------------------------------------------------------------------------
    # Analyze Grand Games
    # -------------------------------------------------------------------------
    df_grand = df[df['BestGame'].str.strip() == 'Grand'].copy()
    print(f"Grand Games: {len(df_grand)} / {len(df)}")
    
    if df_grand.empty:
        print("No Grand games found.")
        return

    # Calculate Metrics for Grand
    df_grand['CntJ'] = pd.to_numeric(df_grand['CntJ'], errors='coerce').fillna(0)
    df_grand['MxLen'] = pd.to_numeric(df_grand['MxLen'], errors='coerce').fillna(0)
    # For Grand, we usually slice by 'Max Side Suit Length' (MxLen)
    # as Jacks (CntJ) is the X-axis.
    
    df_grand['Aces'] = pd.to_numeric(df_grand['Aces'], errors='coerce').fillna(0)
    df_grand['Att10'] = pd.to_numeric(df_grand['Att10'], errors='coerce').fillna(0)
    df_grand['Y'] = df_grand['Aces'] + df_grand['Att10']
    
    df_grand['WinProb'] = pd.to_numeric(df_grand['WinProb'], errors='coerce').fillna(0)
    
    # Slices for Side Suit Length in Grand: 3, 4, 5, 6 (maybe 7?)
    # A side suit of length 6+ is huge in Grand.
    slices_grand = [3, 4, 5, 6]
    
    for sl in slices_grand:
        subset = df_grand[df_grand['MxLen'] == sl]
        if subset.empty:
            print(f"No Grand data for Max Side Suit Length {sl}")
            continue
            
        print(f"Plotting Grand Slice: Max Side Suit Length {sl} (N={len(subset)})")
        
        grouped = subset.groupby(['CntJ', 'Y']).agg({
            'WinProb': 'mean',
            'Hand': 'count'
        }).reset_index()
        
        sizes = grouped['WinProb'] * 1000
        
        total_prob = subset['WinProb'].sum()
        if total_prob > 0:
            cx = (subset['CntJ'] * subset['WinProb']).sum() / total_prob
            cy = (subset['Y'] * subset['WinProb']).sum() / total_prob
        else:
            cx, cy = 0, 0
            
        plt.figure(figsize=(10, 8))
        
        plt.scatter(grouped['CntJ'], grouped['Y'], s=sizes, alpha=0.6, c='gold', edgecolors='black', label='Hand Strength')
        
        if total_prob > 0:
            plt.scatter([cx], [cy], color='red', s=200, marker='X', label=f'Schwerpunkt ({cx:.2f}, {cy:.2f})')
        
        for _, row in grouped.iterrows():
            if row['WinProb'] > 0.01:
                pct = int(row['WinProb'] * 100)
                plt.annotate(f"{pct}%", (row['CntJ'], row['Y']), 
                             ha='center', va='center', color='black', weight='bold', size=9)

        plt.title(f"Best Hand Game (Grand) Stärke (Nebenfärbenlänge {sl})\nN={len(subset)} - Schwerpunkt: Jacks={cx:.2f}, Fulls={cy:.2f}")
        plt.xlabel("Anzahl Buben")
        plt.ylabel("Stehende Volle (Asse + 10)")
        plt.grid(True, linestyle='--', alpha=0.5)
        plt.legend(loc='upper left')
        plt.xticks(range(0, 5)) 
        plt.yticks(range(0, 7))
        plt.ylim(-0.5, 6.5)
        
        out_file = f"{output_dir}/hand_best_grand_len_{sl}.png"
        plt.savefig(out_file)
        print(f"  Saved {out_file}")
        plt.close()

import argparse

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description='Plot Hand Analysis')
    parser.add_argument('--input', type=str, default=None, help='Input CSV file path')
    args = parser.parse_args()
    
    plot_hand_analysis(input_file=args.input)


