
import matplotlib.pyplot as plt
import matplotlib.image as mpimg
from matplotlib.backends.backend_pdf import PdfPages
import os

def create_decision_summary_pdf():
    # A4 Landscape size in inches (approx 11.69 x 8.27)
    # 2x2 Grid
    
    output_pdf = "research/plots/decision_maps_summary_v11.pdf"
    img_files = [
        "research/plots/bubble_decision_map_trumps_4_v11.png",
        "research/plots/bubble_decision_map_trumps_5_v11.png",
        "research/plots/bubble_decision_map_trumps_6_v11.png",
        "research/plots/bubble_decision_map_trumps_7_v11.png"
    ]
    
    titles = [
        "4 Trumps (Weak)",
        "5 Trumps (Medium)",
        "6 Trumps (Strong)",
        "7 Trumps (Monster)"
    ]

    # Check existence
    for f in img_files:
        if not os.path.exists(f):
            print(f"Missing file: {f}")
            return

    # Create Figure
    # A4 Landscape
    fig, axes = plt.subplots(2, 2, figsize=(11.69, 8.27), dpi=300)
    axes = axes.flatten()
    
    for i, ax in enumerate(axes):
        if i < len(img_files):
            img = mpimg.imread(img_files[i])
            ax.imshow(img)
            ax.axis('off') # Hide axis
            # Add simple title if needed, but the plots already have titles.
            # ax.set_title(titles[i], fontsize=12)
        else:
            ax.axis('off')

    plt.tight_layout(pad=0.5)
    
    plt.savefig(output_pdf, dpi=300, orientation='landscape')
    plt.close()
    
    print(f"PDF Saved: {output_pdf}")

if __name__ == "__main__":
    create_decision_summary_pdf()
