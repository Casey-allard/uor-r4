import random
import os

def generate_universal_corpus(output_path="universal_corpus.txt", target_count=50000):
    # 1. Broad Cross-Domain Semantic Repository
    # Categorized pools to ensure high-entropy conceptual cross-linking
    nodes = {
        "metaphysics": ["objective reality", "epistemological truth", "the Ship of Theseus paradigm", "temporal causality", "solipsistic isolation"],
        "physics": ["entropy", "quantum superposition", "the event horizon", "thermodynamic equilibrium", "gravitational singularity"],
        "cognition": ["linguistic syntax", "neuroplastic adaptation", "the subconscious shadow", "meta-cognitive awareness", "symbolic abstraction"],
        "natural_systems": ["mycelial networks", "tectonic subduction", "the biosphere", "cellular mitosis", "apex predation"],
        "architecture": ["structural load-bearing vectors", "gothic vaulting", "the panopticon design", "modular scalability", "spatial thresholds"]
    }

    actions = [
        {"verb": "fundamentally destabilizes", "passive": "is destabilized by", "causal": "thereby forcing a collapse of"},
        {"verb": "mirrors the underlying geometry of", "passive": "is perfectly mirrored by", "causal": "consequently validating the integrity of"},
        {"verb": "serves as an evolutionary precursor to", "passive": "emerges directly from", "causal": "accelerating the development of"},
        {"verb": "reconfigures the topological boundaries of", "passive": "is constrained within the boundaries of", "causal": "redefining our measurement of"},
        {"verb": "acts as a catalytic conduit for", "passive": "is accelerated through the conduit of", "causal": "instantly altering the equilibrium of"},
        {"verb": "contradicts the intrinsic logic of", "passive": "is systematically invalidated by", "causal": "rendering impossible the preservation of"}
    ]

    # 2. Complex, Multi-Clause Syntactic Templates
    # These frames force the router to handle long-range dependencies, inverted syntax, and recursive conditions
    structures = [
        "Insofar as {node1} {action1} {node2}, it necessarily follows that {node3} {action2} {node4}, {causal} the entire system framework.",
        "Although superficial observations suggest that {node1} {action1} {node2}, a deeper analysis reveals how {node3} {action2} {node4}.",
        "By examining the precise mechanism through which {node1} {passive} {node2}, we can deduce why {node3} ultimately {action2} {node4}.",
        "Whenever {node1} {action1} {node2}, an invariant mathematical symmetry requires that {node3} {passive} {node4}.",
        "Supposing that {node3} {action2} {node4} precisely because {node1} {passive} {node2}, the resulting paradigm shifts our understanding of both domains.",
        "Lacking any external stabilization, the reality wherein {node1} {action1} {node2} will inevitably collide with the axiom that {node3} {action2} {node4}.",
        "If it is true that {node1} {passive} {node2}, then any subsequent assertion that {node3} {action1} {node4} must be treated as a logical contradiction."
    ]

    # Extract flat lists for quick, varied random sampling
    all_categories = list(nodes.keys())
    
    corpus = set() # Using a set guarantees 100% unique string profiles
    attempts = 0
    max_attempts = target_count * 10 # Safety break to prevent infinite loops

    print("Generating cross-domain hypersphere context...")

    while len(corpus) < target_count and attempts < max_attempts:
        attempts += 1
        
        # Select 4 completely random, unique concepts across our categories
        # This forces cross-domain pollination (e.g., mixing physics with metaphysics)
        selected_nodes = []
        while len(selected_nodes) < 4:
            cat = random.choice(all_categories)
            node = random.choice(nodes[cat])
            if node not in selected_nodes:
                selected_nodes.append(node)
                
        n1, n2, n3, n4 = selected_nodes
        
        # Pick two distinct verbs/relationships
        act1 = random.choice(actions)
        act2 = random.choice(actions)
        while act1 == act2:
            act2 = random.choice(actions)
            
        # Select a complex grammar frame
        template = random.choice(structures)
        
        # Interpolate variables into the complex sentence structure
        sentence = template.format(
            node1=n1,
            action1=act1["verb"],
            passive=act1["passive"],
            node2=n2,
            node3=n3,
            action2=act2["verb"],
            node4=n4,
            causal=act1["causal"]
        )
        
        # Capitalize the first letter safely
        sentence = sentence[0].upper() + sentence[1:]
        corpus.add(sentence)

    # 3. Stream to Disk via High-Velocity Buffered Write
    with open(output_path, "w", encoding="utf-8") as f:
        for sentence in corpus:
            f.write(sentence + "\n")

    print(f"\n[Success] Generated {len(corpus)} high-entropy, cross-linked sentences.")
    print(f"Destination Vector File: {os.path.abspath(output_path)}")

if __name__ == "__main__":
    generate_universal_corpus()
