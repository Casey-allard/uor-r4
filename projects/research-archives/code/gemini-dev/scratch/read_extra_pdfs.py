import os
import sys

def extract_pdf_to_text(pdf_path, txt_path):
    print(f"Extracting {pdf_path}...")
    try:
        import pypdf
        reader = pypdf.PdfReader(pdf_path)
        text = ""
        for i, page in enumerate(reader.pages):
            text += f"=== PAGE {i+1} ===\n"
            text += page.extract_text() or ""
            text += "\n"
        with open(txt_path, "w", encoding="utf-8") as f:
            f.write(text)
        print(f"Saved {len(text)} characters to {txt_path}")
    except Exception as e:
        print(f"Failed to extract {pdf_path}: {e}")

if __name__ == "__main__":
    pdf_dir = "/Users/adminamn/gemini-dev/extra_reading"
    txt_dir = "/Users/adminamn/gemini-dev/scratch/extra_text"
    os.makedirs(txt_dir, exist_ok=True)
    
    for fn in os.listdir(pdf_dir):
        if fn.lower().endswith(".pdf"):
            pdf_path = os.path.join(pdf_dir, fn)
            txt_name = fn.replace(" ", "_").replace(".pdf", ".txt")
            txt_path = os.path.join(txt_dir, txt_name)
            extract_pdf_to_text(pdf_path, txt_path)
