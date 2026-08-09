use std::collections::HashMap;
use uor_r4_core::semantic::{KappaLabel, WeightedRoute};
use uor_r4_router::geometry::{
    FacetCoordinates, GroundedSemantics, Operator, SemanticGeometry, TypedObject,
};

pub struct MockGeometry {
    pub space_cid: KappaLabel,
}

impl SemanticGeometry for MockGeometry {
    fn space_manifest(&self) -> KappaLabel {
        self.space_cid.clone()
    }

    fn ground(&self, object: &TypedObject) -> Option<GroundedSemantics> {
        if object.content.is_empty() {
            return None;
        }
        Some(GroundedSemantics {
            vsa_vector: vec![0.5; 1024],
            roles: vec!["subject".to_string()],
        })
    }

    fn encode(&self, _grounded: &GroundedSemantics) -> FacetCoordinates {
        let mut coordinates = HashMap::new();
        coordinates.insert("type".to_string(), vec![1, 2, 3]);
        coordinates.insert("entity".to_string(), vec![10, 20, 30, 40]);
        coordinates.insert("relation".to_string(), vec![5, 6]);
        FacetCoordinates { coordinates }
    }

    fn soft_route(&self, coordinates: &FacetCoordinates, max_routes: usize) -> Vec<WeightedRoute> {
        let mut routes = Vec::new();
        if let Some(path) = coordinates.coordinates.get("type") {
            routes.push(WeightedRoute {
                axis: 1, // type axis
                path: path.clone(),
                score: 0.9,
            });
        }
        if let Some(path) = coordinates.coordinates.get("entity") {
            routes.push(WeightedRoute {
                axis: 2, // entity axis
                path: path.clone(),
                score: 0.8,
            });
        }
        routes.into_iter().take(max_routes).collect()
    }

    fn apply_operator(
        &self,
        route: &WeightedRoute,
        operator: &Operator,
    ) -> Option<Vec<WeightedRoute>> {
        if operator.name == "identity" {
            Some(vec![route.clone()])
        } else {
            None
        }
    }
}

#[test]
fn test_mock_geometry_grounding_and_routing() {
    let geom = MockGeometry {
        space_cid: "blake3:mock_space".to_string(),
    };

    let obj = TypedObject {
        object_type: "document".to_string(),
        content: "Hello structured world".to_string(),
    };

    let grounded = geom.ground(&obj).unwrap();
    assert_eq!(grounded.roles, vec!["subject"]);

    let coords = geom.encode(&grounded);
    assert_eq!(coords.coordinates.get("type").unwrap(), &vec![1, 2, 3]);
    assert_eq!(
        coords.coordinates.get("entity").unwrap(),
        &vec![10, 20, 30, 40]
    );

    let routes = geom.soft_route(&coords, 5);
    assert_eq!(routes.len(), 2);
    assert_eq!(routes[0].axis, 1);
    assert_eq!(routes[0].path, vec![1, 2, 3]);
}

#[test]
fn test_selective_backoff() {
    // Simulate selective backoff: shorten the entity path while keeping relation intact
    let mut coords = FacetCoordinates {
        coordinates: HashMap::new(),
    };
    coords.coordinates.insert("type".to_string(), vec![1, 2, 3]);
    coords
        .coordinates
        .insert("entity".to_string(), vec![10, 20, 30, 40]);
    coords
        .coordinates
        .insert("relation".to_string(), vec![5, 6]);

    // Backoff entity axis only
    if let Some(entity_path) = coords.coordinates.get_mut("entity") {
        if entity_path.len() > 1 {
            entity_path.pop();
        }
    }

    assert_eq!(coords.coordinates.get("entity").unwrap(), &vec![10, 20, 30]);
    assert_eq!(coords.coordinates.get("relation").unwrap(), &vec![5, 6]);
}
