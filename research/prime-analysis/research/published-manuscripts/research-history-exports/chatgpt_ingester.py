import os
import json
import re
from datetime import datetime

EXPORT_DIR = "/Volumes/X10 Pro/everything/Research/ChatGPT-History-Export"
OUTPUT_DIR = os.path.join(EXPORT_DIR, "Formatted-Conversations")

def slugify(text):
    text = text.lower()
    text = re.sub(r'[^\w\s-]', '', text)
    text = re.sub(r'[\s_-]+', '-', text)
    return text.strip('-')

def get_conversation_turns(mapping):
    # Reconstruct chronological message list from tree mapping
    # Root nodes usually have no parent
    nodes = {}
    root_node_id = None
    
    for node_id, node_info in mapping.items():
        msg = node_info.get("message")
        parent = node_info.get("parent")
        
        if not parent:
            root_node_id = node_id
            
        nodes[node_id] = {
            "id": node_id,
            "parent": parent,
            "children": node_info.get("children", []),
            "message": msg
        }
        
    # Reconstruct conversation linear path
    # If there are branches, follow the path that reaches the deepest/current node
    # For simplicity, we can do a standard DFS or track parents backwards from the leaf nodes
    # Let's find leaf nodes (nodes with no children)
    leaves = [node_id for node_id, info in nodes.items() if not info["children"]]
    if not leaves:
        return []
        
    # Pick the longest path or just follow the first leaf backwards
    longest_path = []
    for leaf in leaves:
        path = []
        curr = leaf
        while curr:
            path.append(curr)
            curr = nodes[curr]["parent"]
        path.reverse()
        if len(path) > len(longest_path):
            longest_path = path
            
    # Extract messages in chronological order
    turns = []
    for node_id in longest_path:
        node = nodes[node_id]
        msg = node["message"]
        if msg:
            author = msg.get("author", {})
            role = author.get("role")
            
            # Extract content text
            content = msg.get("content", {})
            parts = content.get("parts", [])
            text = content.get("text", "")
            
            # Combine parts
            combined_text = ""
            if parts:
                for part in parts:
                    if isinstance(part, str):
                        combined_text += part + "\n"
                    elif isinstance(part, dict) and "text" in part:
                        combined_text += part["text"] + "\n"
            elif text:
                combined_text = text
                
            if combined_text.strip():
                create_time = msg.get("create_time")
                date_str = ""
                if create_time:
                    try:
                        date_str = datetime.fromtimestamp(create_time).strftime('%Y-%m-%d %H:%M:%S')
                    except:
                        pass
                        
                turns.append({
                    "role": role,
                    "text": combined_text.strip(),
                    "date": date_str
                })
                
    return turns

def ingest_export():
    print(f"Creating output directory: {OUTPUT_DIR}")
    os.makedirs(OUTPUT_DIR, exist_ok=True)
    
    # Locate all conversations-*.json files
    files = [f for f in os.listdir(EXPORT_DIR) if f.startswith("conversations-") and f.endswith(".json")]
    
    total_parsed = 0
    
    for json_file in sorted(files):
        filepath = os.path.join(EXPORT_DIR, json_file)
        print(f"Processing: {json_file}...")
        
        try:
            with open(filepath, 'r') as f:
                conversations = json.load(f)
        except Exception as e:
            print(f"Error loading {json_file}: {e}")
            continue
            
        for conv in conversations:
            title = conv.get("title") or "Untitled Conversation"
            conv_id = conv.get("id")
            create_time = conv.get("create_time")
            
            date_prefix = "unknown-date"
            if create_time:
                try:
                    date_prefix = datetime.fromtimestamp(create_time).strftime('%Y-%m-%d')
                except:
                    pass
                    
            mapping = conv.get("mapping", {})
            turns = get_conversation_turns(mapping)
            
            if not turns:
                continue
                
            # Write markdown file
            safe_title = slugify(title)[:50]
            filename = f"{date_prefix}_{safe_title}_{conv_id[:8]}.md"
            outfile = os.path.join(OUTPUT_DIR, filename)
            
            try:
                with open(outfile, 'w') as out:
                    out.write(f"# {title}\n\n")
                    out.write(f"- **Conversation ID:** `{conv_id}`\n")
                    out.write(f"- **Date:** {date_prefix}\n\n")
                    out.write("---\n\n")
                    
                    for turn in turns:
                        role = turn["role"].capitalize()
                        date_str = f" ({turn['date']})" if turn["date"] else ""
                        out.write(f"### {role}{date_str}\n\n{turn['text']}\n\n")
                        
                total_parsed += 1
            except Exception as e:
                print(f"Error writing conversation {conv_id}: {e}")
                
    print(f"\nSuccessfully parsed and formatted {total_parsed} conversations!")
    print(f"Markdown files saved to: {OUTPUT_DIR}")

if __name__ == "__main__":
    ingest_export()
