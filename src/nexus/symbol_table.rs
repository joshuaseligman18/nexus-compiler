use std::collections::HashMap;

use log::*;

use petgraph::graph::{NodeIndex, Graph};

// Enum for determining the type of a variable in a symbol table
#[derive (Debug, PartialEq, Clone)]
pub enum Type {
    Int,
    String,
    Boolean
}

// Basic struct for what needs to be stored for every symbol table entry
// id is excluded here because it is the key in the hashmap
#[derive (Debug)]
pub struct SymbolTableEntry {
    pub symbol_type: Type,
    pub position: (usize, usize),
    pub scope: usize,
    pub is_initialized: bool,
    pub num_times_used: i32
}

#[derive (Debug)]
pub struct SymbolTable {
    // The graph for the symbol table
    graph: Graph<HashMap<String, SymbolTableEntry>, ()>,

    // The index of the node of the current scope
    cur_scope: Option<usize>
}

impl SymbolTable {
    // Constructor for a new symbol table
    pub fn new() -> Self {
        return SymbolTable {
            graph: Graph::new(),
            cur_scope: None
        };
    }

    // Function to create a new scope and set it as the current scope
    pub fn new_scope(&mut self) {
        debug!("Creating new scope");
        // Add a new node to the graph with the new hashmap
        let new_node: NodeIndex = self.graph.add_node(HashMap::new());
       
        // Check to see if we already have a scope
        if self.cur_scope.is_some() {
            // If so, then create the edge from the new scope to the parent
            self.graph.add_edge(new_node, NodeIndex::from(self.cur_scope.unwrap() as u32), ());
        }

        // Update the current scope to be the new scope
        self.cur_scope = Some(new_node.index());
    }

    // Called to end the current  
    pub fn end_cur_scope(&mut self) {
        if self.cur_scope.is_some() {
            debug!("Exiting current scope");
            // Get a vector of neighbors
            let neighbors: Vec<NodeIndex> = self.graph.neighbors(NodeIndex::new(self.cur_scope.unwrap())).collect();

            if neighbors.len() > 0 {
                // Update the current scope to be the first in the list
                self.cur_scope = Some(neighbors[0].index());
            } else {
                // In the root scope and cur will be None now
                self.cur_scope = None;
            }
        }
    }

    // Adds an identifier to the current scope and returns if it was successful
    pub fn new_identifier(&mut self, id: String, id_type: Type, id_position: (usize, usize)) -> bool {
        // Get the current scope's hash table
        let scope_table: &mut HashMap<String, SymbolTableEntry> = self.graph.node_weight_mut(NodeIndex::new(self.cur_scope.unwrap())).unwrap();
        if (*scope_table).contains_key(&id) {
            // The id already exists so return false
            return false;
        } else {
            // Add the id and its respective information to the hash table
            let mut new_entry = SymbolTableEntry {
                symbol_type: id_type,
                position: id_position,
                scope: self.cur_scope.unwrap(),
                is_initialized: false,
                num_times_used: 0
            };
            (*scope_table).insert(id, new_entry);
            return true;
        }
    }
}