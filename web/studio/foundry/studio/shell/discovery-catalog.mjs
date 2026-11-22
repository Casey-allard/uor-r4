// Generated from FoundrySpecification.discoveryCatalogJson. Do not edit.
export const discoveryCatalog = {
  "profile": "foundry/discovery-catalog/1",
  "revision": 1,
  "maximumCharacters": 512,
  "maximumWords": 32,
  "ignore": [
    "a",
    "an",
    "the",
    "app",
    "application",
    "create",
    "build",
    "make",
    "me",
    "please",
    "with",
    "and"
  ],
  "words": {
    "gallery": 101,
    "photo": 101,
    "photos": 101,
    "image": 101,
    "images": 101,
    "garden": 201,
    "resonant": 201,
    "resonance": 201,
    "audio": 202,
    "sound": 202,
    "motion": 203,
    "animated": 203,
    "visual": 100
  },
  "relations": [
    {
      "fromConcept": 100,
      "toConcept": 101,
      "weight": 500
    },
    {
      "fromConcept": 100,
      "toConcept": 201,
      "weight": 500
    }
  ],
  "policy": {
    "cpu": 3,
    "memory": 3,
    "grants": [],
    "targets": [
      1
    ],
    "evidence": []
  },
  "catalog": [
    {
      "key": 10,
      "version": 1,
      "cpu": 1,
      "memory": 1,
      "vocabulary": [
        900
      ],
      "outputs": [
        7
      ],
      "state": [],
      "events": [],
      "errors": [],
      "effects": [],
      "conflicts": [],
      "targets": [
        1
      ],
      "accessibility": [],
      "quality": [],
      "needs": [],
      "inputs": []
    },
    {
      "key": 11,
      "version": 1,
      "cpu": 1,
      "memory": 1,
      "vocabulary": [
        101
      ],
      "outputs": [],
      "state": [],
      "events": [],
      "errors": [],
      "effects": [],
      "conflicts": [],
      "targets": [
        1
      ],
      "accessibility": [],
      "quality": [],
      "needs": [
        {
          "key": 10,
          "version": 1
        }
      ],
      "inputs": []
    },
    {
      "key": 12,
      "version": 1,
      "cpu": 1,
      "memory": 1,
      "vocabulary": [
        201,
        202,
        203
      ],
      "outputs": [],
      "state": [],
      "events": [],
      "errors": [],
      "effects": [],
      "conflicts": [],
      "targets": [
        1
      ],
      "accessibility": [],
      "quality": [],
      "needs": [
        {
          "key": 10,
          "version": 1
        }
      ],
      "inputs": []
    }
  ],
  "applications": [
    {
      "key": 11,
      "version": 1,
      "kind": "gallery",
      "title": "Quiet gallery",
      "description": "Two illustrated rooms with keyboard controls and saved state. No sound or animation."
    },
    {
      "key": 12,
      "version": 1,
      "kind": "garden",
      "title": "Resonant garden",
      "description": "Two garden states with coordinated color and optional motion and tones. Sound stays off until enabled; reduced-motion preference is respected."
    }
  ]
};
