import matplotlib.pyplot as plt
from matplotlib.backends.backend_pdf import PdfPages
import matplotlib.image as mpimg
import os

def create_pdf():
    plot_dir = "../plots"
    output_pdf = "../skat_analysis_plots.pdf"
    
    # Order of plots
    files = [
        "suit_10k_prob_4_trumps.png",
        "suit_10k_prob_5_trumps.png",
        "suit_10k_prob_6_trumps.png",
        "suit_10k_prob_7_trumps.png",
        "grand_prob_comparison.png"
    ]
    
    titles = [
        "Farbspiel: 4 Trümpfe (Pre vs Post)",
        "Farbspiel: 5 Trümpfe (Pre vs Post)",
        "Farbspiel: 6 Trümpfe (Pre vs Post)",
        "Farbspiel: 7 Trümpfe (Pre vs Post)",
        "Grand: Pre vs Post Vergleich"
    ]
    
    print(f"Generating PDF: {output_pdf}")
    
    with PdfPages(output_pdf) as pdf:
        for filename, title in zip(files, titles):
            path = os.path.join(plot_dir, filename)
            if not os.path.exists(path):
                print(f"Warning: {path} not found. Skipping.")
                continue
                
            print(f"Adding {filename}...")
            
            # Create a figure for the page
            # Landscape-ish
            fig = plt.figure(figsize=(11.69, 8.27)) # A4 Landscape
            
            # Title for the Page
            fig.suptitle(title, fontsize=16, fontweight='bold', y=0.95)
            
            # Load and show image
            img = mpimg.imread(path)
            ax = fig.add_axes([0.05, 0.05, 0.9, 0.85]) # Left, Bottom, Width, Height
            ax.axis('off')
            ax.imshow(img)
            
            pdf.savefig(fig)
            plt.close(fig)
            
    print("Done!")

if __name__ == "__main__":
    create_pdf()
