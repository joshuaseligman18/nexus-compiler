use std::{collections::HashMap};

use log::{info, debug};
use petgraph::{graph::{NodeIndex, Graph}, dot::{Dot, Config}};

use wasm_bindgen::prelude::*;

use crate::nexus::cst_node::{CstNode, NonTerminals, CstNodeTypes};

// Code from https://github.com/rustwasm/wasm-bindgen/blob/main/examples/import_js/crate/src/lib.rs
// Have to import the treeRenderer js module
#[wasm_bindgen(module = "/treeRenderer.js")]
extern "C" {
    // Import the createCst function from js so we can call it from the Rust code
    #[wasm_bindgen(js_name = "createCst")]
    fn create_cst_rendering(dotSrc: &str);
}

#[derive (Debug)]
pub struct Cst {
    // A graph with a string as the node content and no edge weights
    pub graph: Graph<CstNode, ()>,

    // The root of the tree
    root: Option<usize>,

    // The current node we are at
    current: Option<usize>,

    // A hashmap to keep track of parents
    parents: HashMap<usize, Option<usize>>
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
    pub fn add_node(&mut self, kind: CstNodeTypes, label: CstNode) {
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
        if self.current.is_some() {
            let cur_parent: &Option<usize> = self.parents.get(&self.current.unwrap()).unwrap();
            // Set the current node to be the old current's parent
            if cur_parent.is_none() {
                self.current = None;
            } else {
                self.current = Some(cur_parent.unwrap());
            }
        }
    }

    // Function that creates 
    pub fn create_image(&self) {
        // Convert the graph into a dot format
        let graph_dot: Dot<&Graph<CstNode, ()>> = Dot::with_config(&self.graph, &[Config::EdgeNoLabel]);
    
        info!("{:?}", graph_dot);
    
        // Call the JS to create the graph on the webpage using d3.js
        create_cst_rendering(format!("{:?}", graph_dot).as_str());
    }

    // Resets the CST and clears everything in it
    pub fn reset(&mut self) {
        self.graph.clear();
        self.parents.clear();
        self.current = None;
        self.root = None;
    }
}