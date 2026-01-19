import matplotlib.pyplot as plt
from matplotlib.backends.backend_pdf import PdfPages
import matplotlib.image as mpimg
import os

def create_grand_slices_pdf():
    plot_dir = "../plots"
    output_pdf = "../grand_slices_plots.pdf"
    
    files = [
        "grand_prob_length_3.png",
        "grand_prob_length_4.png",
        "grand_prob_length_5.png",
        "grand_prob_length_6.png",
        "grand_prob_length_7.png"
    ]
    
    titles = [
        "Grand: '3-Trumpfer' (Jacks + Longest Suit = 3)",
        "Grand: '4-Trumpfer' (Jacks + Longest Suit = 4)",
        "Grand: '5-Trumpfer' (Jacks + Longest Suit = 5)",
        "Grand: '6-Trumpfer' (Jacks + Longest Suit = 6)",
        "Grand: '7-Trumpfer' (Jacks + Longest Suit = 7)"
    ]
    
    print(f"Generating PDF: {output_pdf}")
    
    with PdfPages(output_pdf) as pdf:
        for filename, title in zip(files, titles):
            path = os.path.join(plot_dir, filename)
            if not os.path.exists(path):
                print(f"Warning: {path} not found. Skipping.")
                continue
                
            print(f"Adding {filename}...")
            
            # Create a figure for the page (Portrait or Landscape)
            # Landscape is good for side-by-side plots
            fig = plt.figure(figsize=(11.69, 8.27)) # A4 Landscape
            
            fig.suptitle(title, fontsize=16, fontweight='bold', y=0.95)
            
            img = mpimg.imread(path)
            ax = fig.add_axes([0.05, 0.05, 0.9, 0.85]) 
            ax.axis('off')
            ax.imshow(img)
            
            pdf.savefig(fig)
            plt.close(fig)
            
    print("Done!")

if __name__ == "__main__":
    create_grand_slices_pdf()
