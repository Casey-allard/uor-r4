import os
import sys

print("Python version:", sys.version)

libraries = ["pypdf", "PyPDF2", "pdfplumber", "fitz", "pdfminer", "pypdfry"]
available = []
for lib in libraries:
    try:
        __import__(lib)
        available.append(lib)
    except ImportError:
        pass

print("Available PDF libraries:", available)

# Let's write a function to try and extract text using available libraries
def extract_text(pdf_path):
    if "pypdf" in available:
        import pypdf
        reader = pypdf.PdfReader(pdf_path)
        text = ""
        for page in reader.pages:
            text += page.extract_text() or ""
        return text
    elif "PyPDF2" in available:
        import PyPDF2
        reader = PyPDF2.PdfReader(pdf_path)
        text = ""
        for page in reader.pages:
            text += page.extract_text() or ""
        return text
    elif "fitz" in available: # PyMuPDF
        import fitz
        doc = fitz.open(pdf_path)
        text = ""
        for page in doc:
            text += page.get_text()
        return text
    elif "pdfplumber" in available:
        import pdfplumber
        with pdfplumber.open(pdf_path) as pdf:
            text = ""
            for page in pdf.pages:
                text += page.extract_text() or ""
            return text
    else:
        return "No library available"

extra_dir = "/Users/adminamn/gemini-dev/extra_reading"
if os.path.exists(extra_dir):
    files = [f for f in os.listdir(extra_dir) if f.endswith(".pdf")]
    print("PDF Files found:", files)
    for f in files:
        path = os.path.join(extra_dir, f)
        txt = extract_text(path)
        print(f"\n--- {f} (Length: {len(txt)}) ---")
        print(txt[:1000]) # print first 1000 characters
        
        # Write extracted text to a text file for easy reading
        txt_path = os.path.join(extra_dir, f.replace(".pdf", ".txt"))
        with open(txt_path, "w", encoding="utf-8") as out:
            out.write(txt)
        print(f"Saved extracted text to {txt_path}")
else:
    print("extra_reading directory not found")
