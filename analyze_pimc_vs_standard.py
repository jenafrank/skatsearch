import subprocess
import json
import random
import os
import sys
import re
import time

LOG_FILE = "analysis_full.log"
SKAT_BINARY = r"target\release\skat_aug23.exe"
CONTEXT_FILE = "temp_analysis_context.json"

# Card Definitions
SUITS = ['C', 'S', 'H', 'D']
RANKS = ['J', 'A', 'T', 'K', 'Q', '9', '8', '7']
ALL_CARDS = [s + r for s in SUITS for r in RANKS]

def generate_random_game():
    """Generates a random Skat game context where Declarer > Def."""
    deck = ALL_CARDS[:]
    random.shuffle(deck)
    
    declarer_cards = deck[0:10]
    left_cards = deck[10:20]
    right_cards = deck[20:30]

    # Randomize game type (Suit or Grand)
    # Randomize game type (Suit or Grand)
    # User Request: "Assume clubs to be trump..."
    # Engine default for Game::Suit is Clubs.
    game_type = random.choice(["Grand", "Grand", "Suit", "Suit", "Suit"])
    
    # Create Context JSON
    context = {
        "game_type": game_type,
        "start_player": "Declarer",
        "declarer_cards": ", ".join(declarer_cards),
        "left_cards": ", ".join(left_cards),
        "right_cards": ", ".join(right_cards),
        "mode": "Value"
    }
    
    return context

def run_value_calc(context):
    """Runs value-calc to check if Declarer wins with Perfect Info."""
    with open(CONTEXT_FILE, 'w') as f:
        json.dump(context, f)
    
    cmd = [SKAT_BINARY, "value-calc", "--context", CONTEXT_FILE, "--optimum-mode", "best_value"]
    try:
        result = subprocess.run(cmd, capture_output=True, text=True, check=True)
        # Parse "Value: (\d+)"
        match = re.search(r"Value: (-?\d+)", result.stdout)
        if match:
            return int(match.group(1))
            
        # Fallback check optimum output style
        # Re-run without optimum mode to get Score.
        cmd_val = [SKAT_BINARY, "value-calc", "--context", CONTEXT_FILE] 
        result_val = subprocess.run(cmd_val, capture_output=True, text=True, check=True)
        match_val = re.search(r"Value: (-?\d+)", result_val.stdout)
        if match_val:
            return int(match_val.group(1))
            
    except subprocess.CalledProcessError as e:
        # Don't log here to avoid spamming the log with failed searches
        return -1
    return -1

def log(msg):
    # Print to console
    print(msg)
    # Append to file
    with open(LOG_FILE, "a", encoding="utf-8") as f:
        f.write(msg + "\n")

def run_playout_full(context, command):
    """Runs a playout command and returns (full_output, points)."""
    with open(CONTEXT_FILE, 'w') as f:
        json.dump(context, f)
    
    cmd = [SKAT_BINARY, command, "--context", CONTEXT_FILE]
    try:
        result = subprocess.run(cmd, capture_output=True, text=True, check=True)
        output = result.stdout.strip()
        points = 0
        
        # Parse Points
        match = re.search(r"Declarer Points: (\d+)", output)
        if match:
            points = int(match.group(1))
        else:
            matches = re.findall(r"Decl: (\d+), Team: (\d+)", output)
            if matches:
                points = int(matches[-1][0])
                
        return output, points
            
    except subprocess.CalledProcessError as e:
        log(f"Error running {command}: {e}")
        log(f"STDERR: {e.stderr}")
        return "", 0

def main():
    N = 10
    if len(sys.argv) > 1:
        N = int(sys.argv[1])

    # Clear log file
    with open(LOG_FILE, "w", encoding="utf-8") as f:
        f.write("")
    
    log(f"Starting Analysis PIMC vs Standard over {N} games...")
    log("-" * 60)

    # Create debug directory
    debug_dir = "debug_jsons"
    if not os.path.exists(debug_dir):
        os.makedirs(debug_dir)

    results = []
    
    count = 0
    attempts = 0
    while count < N:
        attempts += 1
        if attempts % 10 == 0:
            sys.stdout.write(".")
            sys.stdout.flush()
            
        ctx = generate_random_game()
        
        # 1. Check if winning game (Perfect Info > 60)
        # Suppress logging for value-calc during search
        val = run_value_calc(ctx)
        
        if val < 61:
            continue
            
        # Found a game
        print(f"\nFound winning game after {attempts} attempts!")
        attempts = 0
        
        count += 1
        
        # Save Debug JSON
        debug_file = os.path.join(debug_dir, f"game_{count}.json")
        with open(debug_file, "w") as f:
            json.dump(ctx, f, indent=2)
            
        gtype_str = ctx['game_type']
        if gtype_str == "Suit":
            gtype_str += " (Clubs)"
            
        log(f"\n=== GAME {count} ===")
        log(f"Debug File: {debug_file}")
        log(f"Type: {gtype_str}")
        log(f"Declarer: {ctx['start_player']}")
        log(f"Cards Decl:  {ctx['declarer_cards']}")
        log(f"Cards Left:  {ctx['left_cards']}")
        log(f"Cards Right: {ctx['right_cards']}")
        log(f"Perfect Info Value: {val}")

        # 2. Run Standard Playout
        log("\n--- Standard Playout (Perfect Info) ---")
        t0 = time.time()
        std_out, std_pts = run_playout_full(ctx, "standard-playout")
        dt = time.time() - t0
        log(std_out)
        log(f">> Standard Result: {std_pts} Points (Time: {dt:.4f}s)")
        
        # 3. Run PIMC Playout
        log("\n--- PIMC Playout (Imperfect Info) ---")
        t0 = time.time()
        pimc_out, pimc_pts = run_playout_full(ctx, "playout")
        dt = time.time() - t0
        log(pimc_out)
        log(f">> PIMC Result: {pimc_pts} Points (Time: {dt:.4f}s)")
        
        # 4. Compare
        diff = std_pts - pimc_pts
        results.append((std_pts, pimc_pts, diff))
        log(f"\n>> DIFFERENCE (Std - PIMC): {diff}")
        log("-" * 60)
        
    # Stats
    if N > 0:
        avg_std = sum(r[0] for r in results) / N
        avg_pimc = sum(r[1] for r in results) / N
        avg_diff = sum(r[2] for r in results) / N
        
        log("\n=== FINAL STATISTICS ===")
        log(f"Total Games:      {N}")
        log(f"Average Standard: {avg_std:.2f}")
        log(f"Average PIMC:     {avg_pimc:.2f}")
        log(f"Average Diff:     {avg_diff:.2f}")
        
        if avg_diff > 0:
            log("Perfect Information benefits Declarer.")
        elif avg_diff < 0:
            log("Imperfect Information results in higher score?? (Variance/Noise).")
        else:
            log("No significant difference.")

    # Cleanup
    if os.path.exists(CONTEXT_FILE):
        os.remove(CONTEXT_FILE)

if __name__ == "__main__":
    main()
