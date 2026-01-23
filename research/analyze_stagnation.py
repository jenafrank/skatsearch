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

# 3. Calculate Metrics
cols = ['CntJ', 'Aces', 'Att10', 'MxLen', 'PCntJ', 'PAces', 'PAtt10', 'PMxLen', 'MaxProb', 'WinProb']
for c in cols:
    df[c] = pd.to_numeric(df[c], errors='coerce')

df['PreSafeFulls'] = df['Aces'] + df['Att10']
df['PreTotalTrumps'] = df['CntJ'] + df['MxLen']

df['PostSafeFulls'] = df['PAces'] + df['PAtt10']
df['PostTotalTrumps'] = df['PCntJ'] + df['PMxLen']

# 4. Identify Stagnation
# Stagnation = No increase in Jacks, No increase in Trumps, No increase in Fulls
# Actually, strict stagnation: Metrics are IDENTICAL (or worse).
# If Trumps increased, we improved. If Fulls increased, we improved.
# We want cases where: PostTrumps <= PreTrumps AND PostFulls <= PreFulls.

df['Stagnant'] = (df['PostTotalTrumps'] <= df['PreTotalTrumps']) & (df['PostSafeFulls'] <= df['PreSafeFulls'])

stagnant_df = df[df['Stagnant']].copy()
print("Stagnant Hands:", len(stagnant_df))
print(f"Percentage Stagnant: {len(stagnant_df)/len(df):.1%}")

# 5. Calculate Loss
# MaxProb is Pre-Discard (Expected). WinProb is Post-Discard (Actual).
# But wait: MaxProb column in CSV is the result of Pre-Discard Simulation?
# Or is it just the Max of the Probs array?
# In `analyze_general_pre_discard` in Rust:
# `final_max_prob` is passed as `MaxProb`. It comes from `best_variant`.
# `best_variant` comes from sorting `all_probs`.
# `all_probs` comes from `analyze_hand_with_pickup`.
# So `MaxProb` IS the "Expected Win Prob given average Skat".
# HOWEVER, `WinProb` column in CSV... let's check Rust.
# In `row_str` format:
# ..., WinProb, ProbGrand, ...
# Wait, `WinProb` in CSV header corresponds to `final_max_prob` too!
# Rust: `probs[best_variant]`.
# 
# Ah! The CSV row contains `final_max_prob` twice?
# Header: ..., WinProb, ProbGrand, ...
# Code: ..., final_max_prob, probs[0], ...
#
# Crucial realization: usage of `analyze_general_pre_discard` loops 50k times.
# Inside the loop: `analyze_hand_with_pickup` is called.
# It returns `all_probs` (Pre-Discard probabilities).
# AND it returns `optimal_discard` (for the best variant).
#
# BUT! `analyze_hand_with_pickup` estimates the probability by averaging over Skats.
# It does NOT return the probability of the *specific single sample* simulation.
# PIMC calculates "Given this hand, what is my win rate across all unknown distributions?".
#
# The user wants: "If I pick up specific cards (Skat) and they DON'T help, how much does my prob drop?"
# The CSV *might not have* the "Post-Discard Prob given the SPECIFIC Skat cards found".
# Let's check `pimc/analysis.rs`.
# `analyze_general_pre_discard` calls `analyze_hand_with_pickup`.
# This function samples random Skats (from deck) to calculate `prob`.
# The `skat` variable in `analyze_general_pre_discard` is the "True Skat" for that sample row.
# But `prob` is calculated using *random* skats (inference).
# 
# Does the CSV contain the "True Probability" for the "True Final Hand"?
# No directly calculated column for "Post-Prob of Final Hand".
# The `MaxProb` is the Pre-Prob.
#
# WAIT. `FinalHand` is `(hand | skat) ^ discard`.
# If I want the probability of `FinalHand`, I need to re-evaluate it?
# Or does the CSV contain it?
#
# In `main.rs`:
# `let post_sig = HandSignature::from_hand...`
# But no `post_prob` calculation logic in the printing part.
# The `WinProb` column is `final_max_prob` which is Pre-Discard Prob.
#
# PROBLEM: I don't have the "Post-Discard Win Probability for this specific Skat outcome" in the CSV.
# The CSV only contains the "Pre-Discard Win Probability" (averaged over possible Skats).
#
# If the user wants to know the "Loss", I need to simulate or estimate it.
# BUT, I can't do that from the CSV alone if the column isn't there.
#
# Let's verify columns again.
# `MaxProb` -> Pre-Discard.
# Is there a `PostProb`?
# Header: ... WinProb, ProbGrand ...
# Rust: `final_max_prob, probs[0]...`
# Both are Pre-Discard estimates.
#
# CONCLUSION: I cannot answer this from the CSV alone.
# I would need to run a simulation where I calculate:
# 1. Pre-Prob (Average Skat).
# 2. Reveal Skat.
# 3. Post-Prob (Specific Skat).
#
# Current CSV deals with "General Pre-Stats".
#
# User Question: "How much (...) one loses (...) if 3D signature (...) doesn't change?".
# Implications:
# If the signature doesn't change, the hand *quality* is roughly the same as before (minus the "hope" value).
#
# Alternative: I can use the `MaxProb` of the *Pre-Discard* hand as a proxy for the "Hand Quality".
# If I have a Pre-Discard hand with 6 Trumps, I have Prob X.
# If I have a Post-Discard hand with 6 Trumps, I have Prob Y.
#
# If my Skat was bad (Stagnation), my Post-Discard hand has 6 Trumps.
# My Pre-Discard hand (before pickup) had... wait.
# Pre-Discard Hand: 10 Cards. Metric: 1 Jack + 4 Suit (5 Trumps).
# Post-Discard Hand: 10 Cards. Metric: 1 Jack + 4 Suit (5 Trumps).
#
# If metrics are identical (5 -> 5), then my hand *metrics* didn't improve.
# But my *knowledge* changed.
#
# If I group all hands by their **Pre-Discard Metric** (e.g. 5 Trumps) -> Average WinProb = X.
# If I group all hands by their **Post-Discard Metric** (e.g. 5 Trumps) -> Average WinProb = Y.
#
# If `Stagnation` means Pre=5 and Post=5.
# Then the "Loss" is `Prob(Pre=5) - Prob(Post=5)`.
#
# We can calculate average `MaxProb` for Pre-Discard categories.
# And we can calculate average `MaxProb` for Post-Discard categories?
# NO. `MaxProb` in the CSV is ALWAYS the Pre-Discard probability for that row.
#
# I need to infer the "Post-Prob" from the **Table of Statistics** I just built.
#
# Idea:
# 1. Build a Lookup Table: `AverageWinRate(Jacks, Fulls, TLen)` based on PRE-DISCARD data?
#    No, Pre-Discard data includes the "Hope Factor".
#    The *Post-Discard* Average Win Rate (from my Bubble Charts!) represents the "Realized Value" of a hand with those stats.
#
# 2. So:
#    - `ExpectedValue` = `MaxProb` from the CSV row (Pre-Discard Prob).
#    - `RealizedValue` = Look up the `MeanWinRate` from the **Post-Discard** Bubble Chart data for the *Post-Discard Signature* of this row.
#
#    Wait, the Bubble Chart `MeanWinRate` comes from `MaxProb`... which is Pre-Discard Prob.
#    So `MeanWinRate` in Post-Discard Bubble Chart is averaging "Pre-Discard Probs" of hands that *ended up* with that Post-Signature.
#    This is circular and potentially wrong.
#
#    Verification:
#    In `analyze_bubbles_post.py`: `MeanWinRate=('MaxProb', 'mean')`.
#    `MaxProb` is `probs[best_variant]`.
#    If `analyze_general` calculates `probs` BEFORE pickup (Pre-Discard), then:
#    The CSV contains ONLY Pre-Discard Probabilities.
#    It does NOT contain the probability of the specific final hand.
#
#    So, finding "Loss" is impossible with *this* CSV directly row-by-row.
#
#    HOWEVER:
#    If I have a hand with `Pre=5 Trumps`. Its `MaxProb` (Pre) is say 50%.
#    If I pick up bad Skat and end up with `Post=5 Trumps`.
#    Is the `MaxProb` (Pre) still valid?
#    Technically, `MaxProb` WAS the correct probability at time t=0.
#    But the user wants the probability at time t=1 (Post-Discard).
#    Which I don't have.
#
#    UNLESS: I use the `Post-Discard` simulation data to estimate the value of the Post-Discard hand.
#    If I assume that `WinProb(Hand X)` is roughly consistent.
#    I can check: What is the average `MaxProb` of a **Pre-Discard** hand that *starts* with the same signature as my **Post-Discard** hand?
#    (e.g. comparing a 10-card start hand vs a 10-card final hand).
#    If "5 Trumps Pre" has 40% win rate.
#    And "5 Trumps Post" (stagnant) implies I play with 5 Trumps.
#    Does a "5 Trump Start Hand" (Pre) have the same strength as a "5 Trump Final Hand" (Post)?
#    Roughly yes? Both are 10 cards.
#    Difference: "Start Hand" gets to pick up Skat. "Final Hand" has already picked up (and discarded).
#    So a "Start Hand" is ALWAYS stronger than an equivalent "Final Hand" because it has Option Value.
#
#    So `Value(Post=5) ~= Value(Pre=5) - OptionValue`.
#    This doesn't help me find `Value(Post=5)`.
#
#    Wait. The CSV *does* contain `SkatPoints`.
#    But not the "Post-Prob".
#
#    Maybe I can't answer this precisely.
#    The user asks "How much *probability* does one lose".
#    I might need to say: "I need to run a simulation where I re-evaluate the Final Hand".
#
#    OR:
#    I look at the `analyze_general` code again.
#    Does it re-evaluate?
#    Lines 270-380 in main.rs (viewed earlier).
#    It writes `final_max_prob`.
#    And the closure receives `all_probs`.
#    `all_probs` comes from `analyze_hand_with_pickup` -> returns `best_prob` (Pre).
#    It does NOT run `analyze_hand` (no pickup) on the `final_hand`.
#
#    Damn. I am missing the "True Post-Discard Probability" column.
#
#    Workaround:
#    I can't calculate the Delta per row.
#    But I can explain this to the user and propose to **estimate** it using the "Pre-Discard Curve".
#    
#    Hypothesis:
#    If I have 5 Trumps (Pre), my Prob is `P_pre`.
#    If I have 5 Trumps (Post), what is my Prob?
#    Ideally `P_post`.
#    If I look at hands that *started* with 5 Trumps and *ended* with 7 Trumps (Good Skat). Their `P_pre` was high? No, `P_pre` is fixed for the start hand.
#
#    Actually, I can't answer "WinProb Loss" without the Post-Prob column.
#    
#    Let's check `analyze_bubbles_post.py` again.
#    It plots `MaxProb` vs `PostSignature`.
#    This plot shows: "For hands that ended up with 7 Trumps, the PRE-DISCARD probability was X%".
#    This effectively answers: "What kind of Pre-Discard Probability leads to a 7-Trump Post-Hand?"
#    It implies correlation.
#
#    But it doesn't tell us the *current* strength of the 7-Trump hand.
#    
#    WAIT!
#    If `MaxProb` is the "Best Estimate of Winning" given the hand...
#    And we simulated 50k hands.
#    
#    Maybe the user is asking about the *expectation gap*.
#
#    Let's notify the user about this limitation and ask if they settle for a qualitative answer or want a dedicated calculation steps (which requires new Rust code/re-run).
#    Or better: I can construct a proxy.
#    
#    The "Value" of a Post-Discard hand (without pickup rights) is approximately the `MaxProb` of a Pre-Discard hand *minus the Skat-Improvement-Bonus*.
#    
#    Actually, I have `analyze_suit` (Hand Game) simulation data?
#    Yes! `compare_pre_post_suit.py` (from previous session tasks) existed.
#    I have `suit_10k_...` files?
#    Did I run a "Hand Game" simulation?
#    If I have a "Hand Game" simulation (no pickup), then `MaxProb` == `PostProb`.
#    
#    Let's check if I have "Hand Game" data. 
#    The file is `general_pre_stats...`. This is "With Pickup".
#    
#    The user wants to know the "Drop".
#    I will write a script to check if I can find reasonable proxies using the Skat Value?
#    No.
#
#    Let's notify the user. "The current CSV only has Pre-Discard probabilities. To determine the Loss, I need to know the 'True Value' of the final hand. I can try to estimate this by comparing with 'Hand Game' stats if available, or I need to update the simulation."
#
#    Wait, I see `bubble_post_trumps_*.png`.
#    The Y-Axis is `Standing Fulls`. Color is `WinProb`.
#    If I assume that `WinProb` (Pre) is a good proxy for "Average Outcome".
#    And "Stagnation" means we are *below average*.
#
#    Let's pause.
#    If I look at my previous task list: "Run Suit Hand Simulation" (Task 23).
#    Check `research/data/` for Hand simulation CSVs?
#
#    If I have a CSV mapping `Signature -> WinProb (Hand Game)`, I can use that!
#    That would give me `Value(Post-Signature)`.
#
#    Let's check file list.

