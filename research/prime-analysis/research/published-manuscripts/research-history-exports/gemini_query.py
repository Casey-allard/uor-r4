import os
import sys
import json
import re

# We use the new google-genai SDK
try:
    from google import genai
    from google.genai import types
except ImportError:
    print("Error: The 'google-genai' SDK is not installed or import failed.")
    print("Please install it using: pip install google-genai")
    sys.exit(1)

CORPUS_JSONL = "/Volumes/X10 Pro/everything/Research/ChatGPT-History-Export/chat_history_corpus.jsonl"

def search_local_corpus(query, top_n=3):
    """Find top relevant conversations in the local JSONL corpus."""
    if not os.path.exists(CORPUS_JSONL):
        print(f"Error: Corpus file not found at {CORPUS_JSONL}")
        print("Please run gemini_sync.py first to compile the corpus.")
        return []

    print("Searching local conversation corpus...")
    # Compile simple search keywords
    keywords = [kw.lower() for kw in re.findall(r'\w+', query) if len(kw) > 2]
    
    scored_conversations = []
    
    with open(CORPUS_JSONL, 'r', encoding='utf-8') as f:
        for line in f:
            try:
                record = json.loads(line)
                content_lower = record["content"].lower()
                title_lower = record["title"].lower()
                
                # Simple scoring: count keyword occurrences
                score = 0
                for kw in keywords:
                    # Give higher weight to matches in the title
                    score += title_lower.count(kw) * 5
                    score += content_lower.count(kw)
                    
                if score > 0:
                    scored_conversations.append((score, record))
            except Exception as e:
                continue
                
    # Sort by score descending
    scored_conversations.sort(key=lambda x: x[0], reverse=True)
    return [record for score, record in scored_conversations[:top_n]]

def query_gemini(user_query):
    # Retrieve top 3 relevant conversations
    relevant_chats = search_local_corpus(user_query, top_n=3)
    
    if not relevant_chats:
        print("\nNo highly matching conversations found in your local history.")
        print("Will query Gemini directly without local chat history context.\n")
        context = "No relevant personal chat history found."
    else:
        print(f"\nRetrieved {len(relevant_chats)} matching conversations from history:")
        for chat in relevant_chats:
            print(f"  - [{chat['date']}] {chat['title']} (ID: {chat['id'][:8]})")
        print()
        
        # Build context from relevant chats
        context_parts = []
        for idx, chat in enumerate(relevant_chats):
            context_parts.append(f"--- CONVERSATION {idx+1}: {chat['title']} ({chat['date']}) ---")
            context_parts.append(chat["content"])
            context_parts.append("\n" + "="*40 + "\n")
        context = "\n".join(context_parts)

    # Check for GEMINI_API_KEY
    if not os.environ.get("GEMINI_API_KEY"):
        print("Error: GEMINI_API_KEY environment variable not set.")
        print("Please set it in your terminal:")
        print("  export GEMINI_API_KEY=\"your_api_key_here\"")
        sys.exit(1)

    print("Initializing Gemini API Client...")
    try:
        # Initialize client (uses GEMINI_API_KEY env variable automatically)
        client = genai.Client()
        
        system_instruction = (
            "You are Casey's Personal Intelligence assistant. You have access to Casey's personal "
            "ChatGPT history corpus. Below is the relevant context from his past conversations. "
            "Use it to answer his question. Refer to past decisions, projects, or context if available. "
            "Be precise, technical, and match his tone."
        )
        
        prompt = (
            f"Here is the context of Casey's relevant past conversations:\n\n{context}\n\n"
            f"Casey's Question: {user_query}\n\n"
            f"Please answer based on the context above."
        )
        
        print("Querying Gemini (gemini-2.5-flash)...")
        response = client.models.generate_content(
            model='gemini-2.5-flash',
            contents=prompt,
            config=types.GenerateContentConfig(
                system_instruction=system_instruction,
                temperature=0.3
            )
        )
        
        print("\n=== Personal Intelligence Response ===")
        print(response.text)
        print("======================================\n")
        
    except Exception as e:
        print(f"API Error: {e}")

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python3 gemini_query.py \"your search query or question here\"")
        print("Example: python3 gemini_query.py \"what did we decide about the prime R4 router?\"")
        sys.exit(1)
        
    query = " ".join(sys.argv[1:])
    query_gemini(query)
