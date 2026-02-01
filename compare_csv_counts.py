import pandas as pd

def compare_counts(file1, file2):
    print(f"Comparing {file1} and {file2}")
    
    try:
        df1 = pd.read_csv(file1)
        df2 = pd.read_csv(file2)
    except Exception as e:
        print(f"Error reading files: {e}")
        return

    print("DF1 Columns:", df1.columns.tolist())
    print("DF2 Columns:", df2.columns.tolist())
    
    # Normalize columns
    df1.columns = df1.columns.str.strip()
    df2.columns = df2.columns.str.strip()
    
    # Calculate Metrics for DF1 (Pre)
    if 'Att10' not in df1.columns: df1['Att10'] = 0 # Fallback
    df1['SafeFulls'] = df1['Aces'] + df1['Att10']
    df1['TotalTrumps'] = df1['MxLen'] + df1['CntJ']
    
    # Calculate Metrics for DF2 (Hand)
    if 'Att10' not in df2.columns: df2['Att10'] = 0
    df2['SafeFulls'] = df2['Aces'] + df2['Att10']
    df2['TotalTrumps'] = df2['MxLen'] + df2['CntJ']
    
    # Filter for User's Query: 1 Jack, 0 SafeFulls, 4 Total Trumps
    criteria1 = (df1['CntJ'] == 1) & (df1['SafeFulls'] == 0) & (df1['TotalTrumps'] == 4)
    subset1 = df1[criteria1]
    
    criteria2 = (df2['CntJ'] == 1) & (df2['SafeFulls'] == 0) & (df2['TotalTrumps'] == 4)
    subset2 = df2[criteria2]
    
    print(f"User Query (1 Jack, 0 SafeFulls, 4 Trumps):")
    print(f"File 1 (Pre) Count: {len(subset1)}")
    print(f"File 2 (Hand) Count: {len(subset2)}")
    
    print("\nBestGame Distribution (File 1):")
    if 'BestGame' in df1.columns:
        print(df1['BestGame'].value_counts().head())
    else:
        print("BestGame column missing in File 1")

    print("\nBestGame Distribution (File 2):")
    if 'BestGame' in df2.columns:
        print(df2['BestGame'].value_counts().head())
    else:
        print("BestGame column missing in File 2")
        
    print("\nDuplicate Check:")
    print(f"File 1 Duplicates: {df1.duplicated(subset=['CntJ', 'Aces', 'Tens', 'MxLen']).sum()} (based on metrics)")
    # Note: Checking full row duplication is better but these columns are what define the bubbles.

if __name__ == "__main__":
    compare_counts(
        'research/data/general_pre_stats_final_50k_20260120-2156.csv', 
        'research/data/hand_best_game_cluster.csv'
    )
