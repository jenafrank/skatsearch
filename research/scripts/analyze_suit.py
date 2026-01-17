import pandas as pd
import sys

def analyze_suit_csv(files):
    for filename in files:
        print(f"--- Analyzing {filename} ---")
        try:
            df = pd.read_csv(filename)
            
            # Basic Win Probability by Trump Count
            print("\nWin Probability by Trump Count:")
            print(df.groupby('TrumpCount')['WinProb'].mean())
            print("Counts:")
            print(df['TrumpCount'].value_counts().sort_index())

            # By Jacks Mask
            print("\nWin Probability by Jacks:")
            print(df.groupby('JacksMask')['WinProb'].mean())

            # By Side Aces (Aces column)
            print("\nWin Probability by Side Aces:")
            print(df.groupby('Aces')['WinProb'].mean())
            
            # By Side Tens
            print("\nWin Probability by Side Tens:")
            print(df.groupby('Tens')['WinProb'].mean())

             # Combined Trumps + Aces
            df['TrumpsPlusAces'] = df['TrumpCount'] + df['Aces']
            print("\nWin Probability by Trumps + Side Aces:")
            print(df.groupby('TrumpsPlusAces')['WinProb'].mean())

            if 'SkatPoints' in df.columns:
                print("\nAverage Skat Points:")
                print(df['SkatPoints'].mean())
                print("\nWin Probability by Skat Points:")
                print(df.groupby('SkatPoints')['WinProb'].mean())

        except Exception as e:
            print(f"Error processing {filename}: {e}")

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python analyze_suit.py <csv_file1> [csv_file2 ...]")
    else:
        analyze_suit_csv(sys.argv[1:])
