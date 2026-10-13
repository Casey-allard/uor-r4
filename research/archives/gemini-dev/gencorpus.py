import random
import itertools

# Massively expanded entity dictionary
entities = {
    "physics": ["quantum particle", "magnetic field", "thermal energy", "gravitational mass", "photon emission", "kinetic velocity", "wave function", "thermodynamic system", "plasma state", "nuclear fusion"],
    "bio_chem": ["enzyme catalyst", "cellular membrane", "amino acid", "nucleic acid", "ribosome structure", "hemoglobin protein", "pathogenic bacteria", "neural synapse", "mitochondrial matrix", "metabolic pathway"],
    "macro_systems": ["economic market", "tectonic plate", "atmospheric pressure", "oceanic current", "ecological biome", "supply chain", "demographic population", "corporate hierarchy", "monetary currency", "urban infrastructure"],
    "tech_math": ["cryptographic key", "compiler pipeline", "relational database", "neural network", "distributed ledger", "geometric matrix", "vector space", "algorithmic sequence", "binary execution", "hardware bus"]
}

targets = {
    "physics": ["subatomic acceleration", "localized entropy", "quantum superposition", "thermal equilibrium", "electromagnetic resonance", "frictional drag", "spacetime curvature", "potential difference"],
    "bio_chem": ["cellular respiration", "protein synthesis", "genetic replication", "chemical equilibrium", "homeostatic balance", "synaptic transmission", "cellular apoptosis", "enzymatic hydrolysis"],
    "macro_systems": ["inflationary pressure", "seismic displacement", "cyclonic velocity", "biodiversity loss", "capital allocation", "resource scarcity", "demographic shift", "systemic volatility"],
    "tech_math": ["data throughput", "memory allocation", "computational overhead", "cryptographic entropy", "tensor transformation", "packet routing", "cache latency", "state space convergence"]
}

# Verbs that establish strict topological, causal, conditional, or directional transitions
operators = [
    "directly accelerates", "structurally collapses", "exponentially amplifies", "consistently suppresses",
    "geometrically constrains", "systematically converts", "instantly triggers", "permanently alters",
    "strictly minimizes", "dynamically stabilizes", "directly correlates with", "logically invalidates"
]

# Advanced structural clauses to break up simple Subject-Verb-Object monotony
templates = [
    "Whenever a {sub} {op} {obj}, the local system undergoes {mod}.",
    "A {sub} {op} {obj} specifically {mod}.",
    "Undergoing {mod} allows a {sub} to ensure it {op} {obj}.",
    "If a {sub} {op} {obj}, then the surrounding environment exhibits {mod}.",
    "The presence of a {sub} {op} {obj} without causing any {mod}.",
    "By demonstrating {mod}, a specialized {sub} successfully {op} {obj}."
]

modifiers = [
    "zero geometric variance", "maximum structural entropy", "linear metric dilation", 
    "localized manifold curvature", "exponential state space warping", "continuous boundary clamping",
    "rapid temporal phase shifts", "orthogonal coordinate translation", "highly predictable systemic stabilization"
]

def generate_infinite_corpus(target_count=10000):
    sentences = set()
    categories = list(entities.keys())
    
    print(f"Total theoretical uniqueness pool size: {len(entities)*10 * 8 * len(operators) * len(templates) * len(modifiers):,}")
    
    while len(sentences) < target_count:
        cat = random.choice(categories)
        sub = random.choice(entities[cat])
        obj = random.choice(targets[cat])
        op = random.choice(operators)
        mod = random.choice(modifiers)
        tmp = random.choice(templates)
        
        # Inject the tokens into a randomized structural template
        sentence = tmp.format(sub=sub, op=op, obj=obj, mod=mod)
        sentences.add(sentence)
        
    return list(sentences)

if __name__ == "__main__":
    # Generate a fresh, high-complexity batch
    batch_size = 10000
    print(f"Generating {batch_size} highly complex relational sentences...")
    corpus = generate_infinite_corpus(batch_size)
    
    with open("complex_world_model_corpus.txt", "w") as f:
        for sentence in corpus:
            f.write(sentence + "\n")
            
    print("File saved as 'complex_world_model_corpus.txt'. Run this batch and see if your training numbers shoot back up!")
