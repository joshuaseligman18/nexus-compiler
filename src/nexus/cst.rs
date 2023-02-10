use std::{collections::HashMap};

use petgraph::graph::{NodeIndex, Graph};

#[derive (Debug)]
pub struct Cst {
    // A graph with a string as the node content and no edge weights
    pub graph: Graph<String, ()>,

    // The root of the tree
    root: Option<usize>,

    // The current node we are at
    current: Option<usize>,

    // A hashmap to keep track of parents
    parents: HashMap<usize, Option<usize>>
}

#[derive (Debug, PartialEq)]
pub enum CstNodeTypes {
    Root,
    Branch,
    Leaf
}

impl Cst {
    // Constructor for a cst
    pub fn new() -> Self {
        return Cst {
            graph: Graph::new(),
            root: None,
            current: None,
            parents: HashMap::new()
        };
    }

    // Function to add a node to the CST
    pub fn add_node(&mut self, kind: CstNodeTypes, label: String) {
        // Create the node
        let new_node: NodeIndex = self.graph.add_node(label);

        // Check if the tree is empty
        if self.root.is_none() {
            // Create the root node
            self.root = Some(new_node.index());
            self.parents.insert(new_node.index(), None);
        } else {
            // Otherwise add the record of the new branch
            self.parents.insert(new_node.index(), Some(self.current.unwrap()));
            self.graph.add_edge(NodeIndex::from(self.current.unwrap() as u32), new_node, ());
        }

        // If it is not a leaf, then move down the tree
        if kind.ne(&CstNodeTypes::Leaf) {
            self.current = Some(new_node.index());
        }
    }

    // Function to move back up
    pub fn move_up(&mut self) {
        // Get the current parent
        let cur_parent = self.parents.get(&self.current.unwrap()).unwrap();
        // Set the current node to be the old current's parent
        if cur_parent.is_none() {
            self.current = None;
        } else {
            self.current = Some(cur_parent.unwrap());
        }
    }
}