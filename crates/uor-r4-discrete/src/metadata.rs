//! Ordered ternary binder metadata; no carrier/model arithmetic executes here.
use crate::{Error, Result};

pub const ABSENT: u32 = u32::MAX;
pub const MAX_CONTEXT: usize = 243;

#[derive(Clone, Debug)]
struct Node {
    children: Option<[u32; 3]>,
    atom: Option<u32>,
    level: usize,
}

#[derive(Clone, Debug)]
pub struct BinderPlan {
    context: usize,
    nodes: Vec<Node>,
    frontier: Vec<u32>,
    stages: usize,
}

fn invalid(message: impl Into<String>) -> Error {
    Error::Invalid(message.into())
}

impl BinderPlan {
    pub fn new(context: usize) -> Result<Self> {
        if !(1..=MAX_CONTEXT).contains(&context) {
            return Err(invalid("discrete binder context must be 1..=243"));
        }
        let mut nodes: Vec<Node> = Vec::new();
        let mut frontier = Vec::new();
        for atom in 0..context {
            frontier.push(nodes.len() as u32);
            nodes.push(Node {
                children: None,
                atom: Some(atom as u32),
                level: 0,
            });
            loop {
                let n = frontier.len();
                if n < 3 {
                    break;
                }
                let children = [frontier[n - 3], frontier[n - 2], frontier[n - 1]];
                let level = nodes[children[0] as usize].level;
                if children
                    .iter()
                    .any(|&child| nodes[child as usize].level != level)
                {
                    break;
                }
                let parent = nodes.len() as u32;
                nodes.push(Node {
                    children: Some(children),
                    atom: None,
                    level: level + 1,
                });
                frontier.truncate(n - 3);
                frontier.push(parent);
            }
        }
        let stages = 1 + nodes.iter().map(|node| node.level).max().unwrap_or(0);
        let plan = Self {
            context,
            nodes,
            frontier,
            stages,
        };
        plan.validate()?;
        Ok(plan)
    }

    pub fn context(&self) -> usize {
        self.context
    }
    pub fn nodes_count(&self) -> usize {
        self.nodes.len()
    }
    pub fn stages(&self) -> usize {
        self.stages
    }
    pub fn frontier(&self) -> &[u32] {
        &self.frontier
    }

    /// Real nodes only: children[3], then atom; absent fields are type markers.
    pub fn node_words(&self) -> Vec<u32> {
        self.nodes
            .iter()
            .flat_map(|node| {
                let children = node.children.unwrap_or([ABSENT; 3]);
                [
                    children[0],
                    children[1],
                    children[2],
                    node.atom.unwrap_or(ABSENT),
                ]
            })
            .collect()
    }

    pub fn validate(&self) -> Result<()> {
        if !(1..=MAX_CONTEXT).contains(&self.context)
            || self.nodes.is_empty()
            || self.frontier.is_empty()
            || self.frontier.len() > 16
        {
            return Err(invalid("invalid discrete binder inventory"));
        }
        let mut incoming = vec![0u8; self.nodes.len()];
        let mut next_atom = 0usize;
        for (index, node) in self.nodes.iter().enumerate() {
            match (node.children, node.atom) {
                (None, Some(atom)) if node.level == 0 && atom as usize == next_atom => {
                    next_atom += 1;
                }
                (Some(children), None) if node.level > 0 => {
                    for child in children {
                        let child = child as usize;
                        if child >= index
                            || self.nodes[child].level + 1 != node.level
                            || incoming[child] != 0
                        {
                            return Err(invalid(
                                "binder must retain three distinct same-level prior children",
                            ));
                        }
                        incoming[child] = 1;
                    }
                }
                _ => {
                    return Err(invalid(
                        "binder node must be one ordered atom or one complete triple",
                    ))
                }
            }
        }
        if next_atom != self.context
            || self.stages != 1 + self.nodes.iter().map(|node| node.level).max().unwrap_or(0)
        {
            return Err(invalid(
                "binder context or stage count differs from real nodes",
            ));
        }
        let mut roots = vec![false; self.nodes.len()];
        for &root in &self.frontier {
            let root = root as usize;
            if root >= self.nodes.len() || roots[root] || incoming[root] != 0 {
                return Err(invalid("invalid or repeated binder frontier root"));
            }
            roots[root] = true;
        }
        if roots
            .iter()
            .zip(&incoming)
            .any(|(&root, &parents)| root != (parents == 0))
        {
            return Err(invalid("binder frontier omits a retained root"));
        }
        // Topological/index validation only: each atom appears exactly once,
        // in its original order, including incomplete frontier tails.
        let mut stack: Vec<u32> = self.frontier.iter().rev().copied().collect();
        let mut expected_atom = 0u32;
        while let Some(index) = stack.pop() {
            let node = &self.nodes[index as usize];
            match (node.children, node.atom) {
                (Some(children), None) => stack.extend(children.into_iter().rev()),
                (None, Some(atom)) if atom == expected_atom => expected_atom += 1,
                _ => return Err(invalid("binder forest loses ordered atom identity")),
            }
        }
        if expected_atom as usize != self.context {
            return Err(invalid("binder forest does not cover every context atom"));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_admitted_context_preserves_ordered_atom_coverage() -> Result<()> {
        for context in 1..=MAX_CONTEXT {
            let plan = BinderPlan::new(context)?;
            plan.validate()?;
            assert_eq!(plan.node_words().len(), 4 * plan.nodes_count());
            assert!(plan.frontier().len() <= 16);
            assert!(plan.stages() <= 6);
        }
        assert!(BinderPlan::new(0).is_err());
        assert!(BinderPlan::new(MAX_CONTEXT + 1).is_err());
        Ok(())
    }

    #[test]
    fn triple_and_nine_atom_closure_retain_exact_child_indices() -> Result<()> {
        let plan = BinderPlan::new(3)?;
        assert_eq!(plan.frontier(), &[3]);
        assert_eq!(plan.stages(), 2);
        assert_eq!(&plan.node_words()[12..], &[0, 1, 2, ABSENT]);
        let plan = BinderPlan::new(9)?;
        assert_eq!(plan.frontier(), &[12]);
        assert_eq!(plan.stages(), 3);
        assert_eq!(&plan.node_words()[48..], &[3, 7, 11, ABSENT]);
        Ok(())
    }
}
