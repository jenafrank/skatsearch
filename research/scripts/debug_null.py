
# Copied from analyze_null.py to debug
NULL_RANKS = {'7': 0, '8': 1, '9': 2, 'T': 3, 'J': 4, 'Q': 5, 'K': 6, 'A': 7}

def parse_hand(hand_str):
    hand_str = hand_str.strip("[]")
    cards = hand_str.split()
    suits = {'C': [], 'S': [], 'H': [], 'D': []}
    
    for card in cards:
        if not card: continue
        suit = card[0]
        rank_char = card[1]
        if rank_char in NULL_RANKS:
            suits[suit].append(NULL_RANKS[rank_char])
            
    for s in suits:
        suits[s].sort()
        
    return suits

def is_suit_safe(held_ranks):
    held_set = set(held_ranks)
    all_ranks = set(range(8))
    opponent_ranks = sorted(list(all_ranks - held_set))
    
    num_checks = min(len(held_ranks), len(opponent_ranks))
    unsafe_gaps = 0
    
    print(f"Held: {held_ranks}")
    print(f"Opponent: {opponent_ranks}")
    
    for i in range(num_checks):
        safe_check = held_ranks[i] < opponent_ranks[i]
        print(f"Check {i}: Held {held_ranks[i]} < Opp {opponent_ranks[i]} ? {safe_check}")
        if held_ranks[i] > opponent_ranks[i]: # Wait, logic was > ? No, if H > O, then unsafe.
            # But what if H == O? Impossible (sets disjoint).
            # So H > O means Unsafe.
            unsafe_gaps += 1
            
    return (unsafe_gaps == 0), unsafe_gaps

if __name__ == "__main__":
    hand_str = "[C9 HQ HJ HT H8 H7 DQ DT D9 D7]"
    print(f"Testing Hand: {hand_str}")
    suits = parse_hand(hand_str)
    
    print("Clubs Analysis:")
    safe, gaps = is_suit_safe(suits['C'])
    print(f"Safe: {safe}, Gaps: {gaps}")
