import sys
import os

def extract_text(pdf_path, txt_path):
    print(f"Reading {pdf_path}")
    try:
        import pypdf
        reader = pypdf.PdfReader(pdf_path)
        text = ""
        for i, page in enumerate(reader.pages):
            text += f"--- PAGE {i+1} ---\n"
            text += page.extract_text() + "\n"
        with open(txt_path, "w") as f:
            f.write(text)
        print(f"Saved to {txt_path}")
    except Exception as e:
        print(f"Error reading {pdf_path}: {e}")

if __name__ == "__main__":
    folder = "/Users/adminamn/gemini-dev/AI-Research/deepermath"
    files = [
        ("QIMC.pdf", "qimc_extracted.txt"),
        ("Topological Phase Transport and Thermodynamic Stability in Prime Aligned Multi-Agent Routing Manifolds.pdf", "topo_extracted.txt"),
        ("Prime Candidate Preprint (m182589953).pdf", "prime_candidate_extracted.txt"),
        ("Canonical Explorations in Geometric Routing-1.pdf", "canonical_extracted.txt")
    ]
    for fn, out in files:
        extract_text(os.path.join(folder, fn), os.path.join("/Users/adminamn/gemini-dev/scratch", out))
