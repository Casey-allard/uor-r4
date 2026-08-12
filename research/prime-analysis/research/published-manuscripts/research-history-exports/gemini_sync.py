import os
import json
import re

CONV_DIR = "/Volumes/X10 Pro/everything/Research/ChatGPT-History-Export/Formatted-Conversations"
OUTPUT_JSONL = "/Volumes/X10 Pro/everything/Research/ChatGPT-History-Export/chat_history_corpus.jsonl"
OUTPUT_MD = "/Volumes/X10 Pro/everything/Research/ChatGPT-History-Export/chat_history_corpus.md"

def compile_corpus():
    print("Scanning Formatted-Conversations directory...")
    if not os.path.exists(CONV_DIR):
        print(f"Error: Directory {CONV_DIR} does not exist!")
        return

    md_files = [f for f in os.listdir(CONV_DIR) if f.endswith(".md")]
    print(f"Found {len(md_files)} markdown files.")

    # Sort files chronologically by filename (starts with YYYY-MM-DD)
    md_files.sort()

    jsonl_records = []
    unified_md_content = []

    unified_md_content.append("# ChatGPT Conversation Corpus\n")
    unified_md_content.append(f"Compiled on: {import_date()}\n")
    unified_md_content.append(f"Total Conversations: {len(md_files)}\n\n")
    unified_md_content.append("This file contains the complete chronological record of all exported ChatGPT conversations.\n\n")
    unified_md_content.append("---\n\n")

    print("Compiling conversations...")
    for idx, filename in enumerate(md_files):
        filepath = os.path.join(CONV_DIR, filename)
        
        try:
            with open(filepath, 'r', encoding='utf-8') as f:
                content = f.read()
        except Exception as e:
            print(f"Error reading {filename}: {e}")
            continue

        # Extract metadata from content
        title_match = re.match(r"^#\s+(.+)$", content, re.MULTILINE)
        title = title_match.group(1).strip() if title_match else "Untitled Conversation"

        id_match = re.search(r"-\s+\*\*Conversation ID:\*\*\s+`([^`]+)`", content)
        conv_id = id_match.group(1).strip() if id_match else f"unknown-{idx}"

        date_match = re.search(r"-\s+\*\*Date:\*\*\s+(.+)$", content, re.MULTILINE)
        date_str = date_match.group(1).strip() if date_match else "unknown-date"

        # Record for JSONL
        record = {
            "id": conv_id,
            "title": title,
            "date": date_str,
            "filename": filename,
            "content": content
        }
        jsonl_records.append(record)

        # Content for Unified MD
        unified_md_content.append(content)
        unified_md_content.append("\n\n---\n\n")

    # Save JSONL
    print(f"Saving JSONL corpus to {OUTPUT_JSONL}...")
    try:
        with open(OUTPUT_JSONL, 'w', encoding='utf-8') as f:
            for rec in jsonl_records:
                f.write(json.dumps(rec, ensure_ascii=False) + "\n")
        print(f"JSONL corpus saved successfully! ({len(jsonl_records)} lines)")
    except Exception as e:
        print(f"Error writing JSONL corpus: {e}")

    # Save Unified MD
    print(f"Saving unified Markdown corpus to {OUTPUT_MD}...")
    try:
        with open(OUTPUT_MD, 'w', encoding='utf-8') as f:
            f.writelines(unified_md_content)
        print("Unified Markdown corpus saved successfully!")
    except Exception as e:
        print(f"Error writing unified Markdown corpus: {e}")

def import_date():
    from datetime import datetime
    return datetime.now().strftime('%Y-%m-%d')

if __name__ == "__main__":
    compile_corpus()
