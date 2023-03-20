use std::collections::HashMap;

use log::*;

use petgraph::graph::{NodeIndex, Graph};

use crate::util::nexus_log;

// Enum for determining the type of a variable in a symbol table
#[derive (Debug, PartialEq, Clone)]
pub enum Type {
    Int,
    String,
    Boolean
}

// Enum for the symbol table entry fields to keep track of to prevent code duplication
#[derive (Debug)]
pub enum SymbolTableEntryField {
    Initialized,
    Used
}

// Basic struct for what needs to be stored for every symbol table entry
// id is excluded here because it is the key in the hashmap
#[derive (Debug)]
pub struct SymbolTableEntry {
    pub symbol_type: Type,
    pub position: (usize, usize),
    pub scope: usize,
    pub is_initialized: bool,
    pub is_used: bool
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
            let new_entry = SymbolTableEntry {
                symbol_type: id_type,
                position: id_position,
                scope: self.cur_scope.unwrap(),
                is_initialized: false,
                is_used: false
            };
            (*scope_table).insert(id, new_entry);
            return true;
        }
    }

    // Returns a reference to the appropriate symbol table entry
    // based on the current scope
    pub fn get_symbol(&mut self, id: &str) -> Option<&SymbolTableEntry> {
        // Start with the current scope
        let mut cur_scope_check: usize = self.cur_scope.unwrap();
      
        // This loop has checks at the end, but work has to be done first
        loop {
            // Get the hashmap for the scope
            let scope_table: &HashMap<String, SymbolTableEntry> = self.graph.node_weight(NodeIndex::new(cur_scope_check)).unwrap();
            if (*scope_table).contains_key(id) {
                // If the variable exists, then return the entry
                return (*scope_table).get(id);
            } else {
                if cur_scope_check == 0 {
                    // We are now in the master scope, so the variable does
                    // not exist relative to the current scope
                    return None;
                } else {
                    // Get a vector of neighbors
                    let neighbors: Vec<NodeIndex> = self.graph.neighbors(NodeIndex::new(cur_scope_check)).collect();
                    
                    // Move on the the next higher scope
                    cur_scope_check = neighbors[0].index();
                }
            }
        }
    }

    // Function to set a variable to be initialized
    pub fn set_entry_field(&mut self, id: &str, field: SymbolTableEntryField) {
        // Start with the current scope
        let mut cur_scope_use: usize = self.cur_scope.unwrap();

        loop {
            // Get the hashmap for the current scope being checked
            let scope_table: &mut HashMap<String, SymbolTableEntry> = self.graph.node_weight_mut(NodeIndex::new(cur_scope_use)).unwrap();
            if (*scope_table).contains_key(id) {
                // Get the entry and update the initialized field
                let id_entry: &mut SymbolTableEntry = (*scope_table).get_mut(id).unwrap();
                
                // Set the apprpriate flag based on the inputted field
                match field {
                    SymbolTableEntryField::Initialized => id_entry.is_initialized = true,
                    SymbolTableEntryField::Used => id_entry.is_used = true
                }
                break;
            } else {
                if cur_scope_use == 0 {
                    // Scope id of 0 means we are in the master scope, so break from the loop
                    break;
                } else {
                    // Move on to the next scope in the tree
                    let neighbors: Vec<NodeIndex> = self.graph.neighbors(NodeIndex::new(cur_scope_use)).collect();
                    cur_scope_use = neighbors[0].index();
                }
            }
        }
    }

    // Function to find all of the warnings after scope and type checks are completed
    pub fn mass_warnings(&mut self) -> i32 {
        let mut warning_count: i32 = 0;
        
        // Iterate through each scope
        for scope_table in self.graph.node_weights() {
            // Iterate through each entry in the scope's symbol table
            for (id_name, entry) in scope_table.iter() {
                if !entry.is_initialized {
                    if entry.is_used {
                        // Throw warning for declared and used but not initialized
                        nexus_log::log(
                            nexus_log::LogTypes::Warning,
                            nexus_log::LogSources::SemanticAnalyzer,
                            format!("Warning at {:?}; Id [ {} ] is declared and used, but never initialized", entry.position, id_name)
                        );
                        warning_count += 1;
                    } else {
                        // Throw warning for declared but never initialized or used
                        nexus_log::log(
                            nexus_log::LogTypes::Warning,
                            nexus_log::LogSources::SemanticAnalyzer,
                            format!("Warning at {:?}; Id [ {} ] is declared, but never initialized or used", entry.position, id_name)
                        );
                        warning_count += 1;
                    }
                } else {
                    if !entry.is_used {
                        // Throw warning for declared and initialized but never used
                        nexus_log::log(
                            nexus_log::LogTypes::Warning,
                            nexus_log::LogSources::SemanticAnalyzer,
                            format!("Warning at {:?}; Id [ {} ] is declared and initialized, but never used", entry.position, id_name)
                        );
                        warning_count += 1;
                    }
                }
            }
        }
        return warning_count;
    }
}
