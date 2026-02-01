
import numpy as np
import matplotlib.pyplot as plt
import seaborn as sns
import os

# Set style
plt.style.use('seaborn-v0_8-darkgrid')
sns.set_context("notebook", font_scale=1.2)

def analyze_decision_boundary():
    # User Parameters
    # P_s: Win Probability with Skat
    # P_h: Win Probability Hand (without Skat)
    
    # Values (normalized to Base Game Value V=1)
    # Hand Game: Worth +33% -> V_h = 1.333
    # Skat Game: Worth 1.0  -> V_s = 1.0
    
    # Penalties
    # Hand Loss: Simple -> -V_h = -1.333
    # Skat Loss: Double -> -2 * V_s = -2.0
    
    V_h = 1.333
    V_s = 1.0
    
    # Expected Values
    # EV_h = P_h * V_h + (1 - P_h) * (-V_h)
    #      = V_h * (2 * P_h - 1)
    
    # EV_s = P_s * V_s + (1 - P_s) * (-2 * V_s)
    #      = V_s * (P_s - 2 + 2*P_s) 
    #      = V_s * (3 * P_s - 2)
    
    # Decision Boundary: EV_h > EV_s
    # V_h * (2*P_h - 1) > V_s * (3*P_s - 2)
    # 1.333 * (2*P_h - 1) > 1.0 * (3*P_s - 2)
    # 2.666 * P_h - 1.333 > 3 * P_s - 2
    # 2.666 * P_h > 3 * P_s - 0.667
    # P_h > (3/2.666) * P_s - (0.667/2.666)
    # P_h > 1.125 * P_s - 0.25
    
    # Create Grid
    p_skat = np.linspace(0, 1, 100)
    p_hand = np.linspace(0, 1, 100)
    X, Y = np.meshgrid(p_skat, p_hand)
    
    # Calculate EV Difference (EV_h - EV_s)
    EV_h = V_h * (2 * Y - 1)
    EV_s = V_s * (3 * X - 2)
    Z = EV_h - EV_s
    
    # Plot
    plt.figure(figsize=(10, 8))
    
    # Heatmap of Advantage
    levels = np.linspace(-2, 2, 21)
    contour = plt.contourf(X, Y, Z, levels=levels, cmap='RdBu', alpha=0.8, extend='both')
    cbar = plt.colorbar(contour)
    cbar.set_label('Advantage of Playing Hand (EV Units)')
    
    # Zero Line (Decision Boundary)
    plt.contour(X, Y, Z, levels=[0], colors='black', linewidths=3, linestyles='--')
    
    # Add Text Labels for Zones
    plt.text(0.2, 0.8, 'BETTER TO PLAY HAND', fontsize=16, fontweight='bold', color='darkblue', ha='center')
    plt.text(0.8, 0.2, 'BETTER TO PICK UP', fontsize=16, fontweight='bold', color='darkred', ha='center')
    
    # Add specific reference points
    # Case 1: Skat Win 100% (X=1.0) -> Boundary P_h > 0.875
    # Case 2: Skat Win 80% (X=0.8) -> Boundary P_h > 0.65
    # Case 3: Skat Win 60% (X=0.6) -> Boundary P_h > 0.425
    
    points = [
        (1.0, 0.875, "If Skat is 100%, Hand must be >88%"),
        (0.8, 0.65, "If Skat is 80%, Hand must be >65%"),
        (0.6, 0.425, "If Skat is 60%, Hand must be >43%")
    ]
    
    for x_val, y_val, text in points:
        plt.plot(x_val, y_val, 'ko', markersize=8)
        plt.annotate(text, (x_val, y_val), xytext=(x_val-0.25, y_val+0.05), 
                     arrowprops=dict(facecolor='black', shrink=0.05), fontsize=10, bbox=dict(boxstyle="round,pad=0.3", fc="white", ec="black", alpha=0.8))

    plt.title("Decision Matrix: Hand Game vs. Pickup Skat\n(Assumes: Hand Value +33%, Double Penalty for Lost Pickup)", fontsize=15)
    plt.xlabel("Probability of Winning with Skat (P_skat)", fontsize=12)
    plt.ylabel("Probability of Winning Hand Game (P_hand)", fontsize=12)
    plt.grid(True, linestyle=':', alpha=0.6)
    
    # Safe area
    plt.plot([0, 1], [0, 1], 'w:', linewidth=1, label="Equal Probability Line")
    
    output_dir = "research/plots"
    os.makedirs(output_dir, exist_ok=True)
    out_path = f"{output_dir}/decision_boundary_hand_vs_skat.png"
    plt.savefig(out_path, dpi=100, bbox_inches='tight')
    plt.close()
    print(f"Saved: {out_path}")
    
    # Interactive Plotly
    fig = px.imshow(Z, 
                    x=p_skat, 
                    y=p_hand, 
                    origin='lower',
                    labels=dict(x="P(Win Skat)", y="P(Win Hand)", color="Hand Advantage"),
                    color_continuous_scale='RdBu',
                    title="Hand vs Skat Decision Matrix")
    
    # Add line
    # P_h = 1.125 * P_s - 0.25
    x_line = np.linspace(0, 1, 100)
    y_line = 1.125 * x_line - 0.25
    
    fig.add_scatter(x=x_line, y=y_line, mode='lines', line=dict(color='black', width=3, dash='dash'), name='Decision Boundary')
    
    html_out = f"{output_dir}/decision_boundary_hand_vs_skat.html"
    fig.write_html(html_out)
    print(f"Saved Interactive: {html_out}")

import plotly.express as px

if __name__ == "__main__":
    analyze_decision_boundary()
