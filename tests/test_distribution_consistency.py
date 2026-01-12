import subprocess
import json
import random
import os
import sys
import re
import time

# Configuration
SKAT_BINARY = r"target\release\skat_aug23.exe"
CONTEXT_FILE = "temp_consistency_context.json"
LOG_FILE = "consistency_test.log"
NUM_GAMES = 20  # Number of random distributions to test

# Card Definitions
SUITS = ['C', 'S', 'H', 'D']
RANKS = ['J', 'A', 'T', 'K', 'Q', '9', '8', '7']
ALL_CARDS = [s + r for s in SUITS for r in RANKS]

def log(msg, to_console=True):
    if to_console:
        print(msg)
    with open(LOG_FILE, "a", encoding="utf-8") as f:
        f.write(msg + "\n")

def generate_random_context():
    """Generates a random Skat game context."""
    deck = ALL_CARDS[:]
    random.shuffle(deck)
    
    declarer_cards = deck[0:10]
    left_cards = deck[10:20]
    right_cards = deck[20:30]
    
    # Randomly choose game type and start player for variety
    game_type = random.choice(["Grand", "Suit", "Suit", "Null"])
    start_player = random.choice(["Declarer", "Left", "Right"])
    
    context = {
        "game_type": game_type,
        "start_player": start_player,
        "declarer_cards": ", ".join(declarer_cards),
        "left_cards": ", ".join(left_cards),
        "right_cards": ", ".join(right_cards),
        "mode": "Value" # Default required by some parsers, though overridden by CLI args
    }
    return context

def run_solver(context, optimum_mode=None):
    """
    Runs the skat solver with the given context and optional optimum-mode.
    Returns (value, output, duration) or (None, output, duration) if failure.
    """
    with open(CONTEXT_FILE, 'w') as f:
        json.dump(context, f)
        
    cmd = [SKAT_BINARY, "value-calc", "--context", CONTEXT_FILE]
    if optimum_mode:
        cmd.extend(["--optimum-mode", optimum_mode])
        
    try:
        # Run command
        t0 = time.time()
        result = subprocess.run(cmd, capture_output=True, text=True, check=True)
        dt = time.time() - t0
        
        # Parse output for value
        # Pattern usually: "Value: 85" or "Value: -30"
        # New Optimum Pattern: "Optimum Best Move: XX, Score: YYY, Value: 85"
        
        # Check for Optimum Output First
        match_opt = re.search(r"Value: (-?\d+)", result.stdout)
        if match_opt:
            return int(match_opt.group(1)), result.stdout, dt
        else:
            log(f"ERROR: Could not parse value from output for mode={optimum_mode}")
            log(f"STDOUT: {result.stdout}")
            return None, result.stdout, dt
            
    except subprocess.CalledProcessError as e:
        log(f"CRASH: Solver failed for mode={optimum_mode}")
        log(f"STDERR: {e.stderr}")
        return None, e.stderr, 0.0

def run_test():
    # Setup
    with open(LOG_FILE, "w", encoding="utf-8") as f:
        f.write(f"Starting Consistency Test at {time.strftime('%Y-%m-%d %H:%M:%S')}\n")
        f.write("="*60 + "\n")
    
    if not os.path.exists(SKAT_BINARY):
        log(f"FATAL: Binary not found at {SKAT_BINARY}")
        sys.exit(1)

    # Create debug directory
    debug_dir = "debug_jsons"
    if not os.path.exists(debug_dir):
        os.makedirs(debug_dir)

    failures = 0
    successes = 0

    for i in range(1, NUM_GAMES + 1):
        log(f"\nTest Case {i}/{NUM_GAMES}")
        ctx = generate_random_context()
        
        # Save Debug JSON
        debug_path = os.path.join(debug_dir, f"consistency_{i}.json")
        with open(debug_path, "w") as f:
            json.dump(ctx, f, indent=2)
            
        log(f"Debug File: {debug_path}")
        log(f"Game: {ctx['game_type']}, Start: {ctx['start_player']}")
        log(f"Cards Decl : {ctx['declarer_cards']}")
        log(f"Cards Left : {ctx['left_cards']}")
        log(f"Cards Right: {ctx['right_cards']}")
        
        # 1. Standard Mode
        val_std, out_std, dt_std = run_solver(ctx, None)
        if val_std is None:
            failures += 1
            continue
        log(f"Standard Mode : {val_std:<4} (Time: {dt_std:.4f}s)")
            
        # 2. Optimum Mode: best_value
        val_best, out_best, dt_best = run_solver(ctx, "best_value")
        if val_best is None:
            failures += 1
            continue
        log(f"BestValue Mode: {val_best:<4} (Time: {dt_best:.4f}s)")

        # 3. Optimum Mode: all_winning
        val_win, out_win, dt_win = run_solver(ctx, "all_winning")
        if val_win is None:
            failures += 1
            continue
        log(f"AllWinning Mod: {val_win:<4} (Time: {dt_win:.4f}s)")
            
        # --- Verification ---
        failed_this = False
        
        # Check 1: Standard vs Best Value (Should be Exact)
        if val_std != val_best:
            log(f"FAIL: Standard ({val_std}) != BestValue ({val_best})")
            failed_this = True
        else:
            log(f"OK: Standard == BestValue ({val_std})")
            
        # Check 2: Standard vs All Winning (Consistency)
        is_null = (ctx['game_type'] == "Null")
        
        if not is_null:
            std_is_win = (val_std >= 61)
            win_is_win = (val_win >= 61)
            
            if std_is_win != win_is_win:
                log(f"FAIL: Win/Loss Mismatch! Standard Value: {val_std} (Win={std_is_win}), AllWinning Value: {val_win} (Win={win_is_win})")
                failed_this = True
            else:
                 log(f"OK: Win/Loss Consistent. Std: {val_std}, WinMode: {val_win}")
        else:
            # Null Logic
            if val_std != val_win:
                 log(f"WARN: Null Game mismatch? Std: {val_std}, WinMode: {val_win}")
                 if val_std != val_win:
                     failed_this = True # Enforce strictness for now
            else:
                log(f"OK: Null Game Consistent ({val_std})")

        if failed_this:
            failures += 1
            log(f"FAILURE: See {debug_path}")
        else:
            successes += 1
            
    log("="*60)
    log(f"FINISHED. Total: {NUM_GAMES}, Success: {successes}, Failures: {failures}")
    
    # Cleanup
    if os.path.exists(CONTEXT_FILE):
        os.remove(CONTEXT_FILE)

    if failures > 0:
        sys.exit(1)
    else:
        sys.exit(0)

if __name__ == "__main__":
    run_test()
