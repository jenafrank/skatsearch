import pandas as pd
import os

def dump():
    pre_file = "research/data/general_pre_stats_final_50k_36611593.csv"
    import glob
    files = glob.glob("research/data/general_pre_stats_final_50k_*.csv")
    pre_file = max(files, key=os.path.getctime)
    df_pre = pd.read_csv(pre_file)
    df_pre.columns = df_pre.columns.str.strip()
    pickup_col = 'MaxProb' if 'MaxProb' in df_pre.columns else 'WinProb'
    df_pre['SafeFulls'] = df_pre['Aces'] + df_pre['Att10']
    df_pre['Trumps'] = df_pre['MxLen'] + df_pre['CntJ']
    pre_grouped = df_pre.groupby(['Trumps', 'CntJ', 'SafeFulls']).agg(ProbPickup=(pickup_col, 'mean')).reset_index()

    hand_file = 'research/data/hand_best_game_cluster.csv'
    df_hand = pd.read_csv(hand_file)
    df_hand.columns = df_hand.columns.str.strip()
    for c in ['ProbClubs', 'ProbSpades', 'ProbHearts', 'ProbDiamonds', 'ProbGrand', 'Aces', 'Att10', 'MxLen', 'CntJ']:
        df_hand[c] = pd.to_numeric(df_hand[c], errors='coerce')
    df_hand['ProbSuitHand'] = df_hand[['ProbClubs', 'ProbSpades', 'ProbHearts', 'ProbDiamonds']].max(axis=1)
    df_hand['ProbGrandHand'] = df_hand['ProbGrand']
    df_hand['SafeFulls'] = df_hand['Aces'] + df_hand['Att10']
    df_hand['Trumps'] = df_hand['MxLen'] + df_hand['CntJ']
    hand_grouped = df_hand.groupby(['Trumps', 'CntJ', 'SafeFulls']).agg(
        ProbSuitHand=('ProbSuitHand', 'mean'), ProbGrandHand=('ProbGrandHand', 'mean')).reset_index()

    merged = pd.merge(pre_grouped, hand_grouped, on=['Trumps', 'CntJ', 'SafeFulls'], how='inner')

    def get_decision(row):
        ev_g = 2.66 * row['ProbGrandHand'] - 1.33
        ev_s = 2 * row['ProbSuitHand'] - 1
        ev_p = 3 * row['ProbPickup'] - 2
        if max(ev_g, ev_s, ev_p) <= 0: return 'Fold'
        best = max(ev_g, ev_s, ev_p)
        if best == ev_g: return 'Grand'
        if best == ev_s: return 'Suit'
        return 'Pickup'

    merged['Decision'] = merged.apply(get_decision, axis=1)
    for trumps in range(4, 9):
        sub = merged[(merged['Trumps'] == trumps) & (merged['Decision'].isin(['Grand', 'Suit', 'Pickup']))]
        if len(sub) == 0: continue
        print(f"\n--- Trumps: {trumps} ---")
        for d in ['Grand', 'Suit', 'Pickup']:
            d_sub = sub[sub['Decision'] == d]
            if len(d_sub) > 0:
                print(f"Decision {d}:")
                for _, r in d_sub.iterrows():
                    print(f"  CntJ: {int(r['CntJ'])}, SafeFulls: {int(r['SafeFulls'])}")

if __name__ == "__main__":
    dump()
