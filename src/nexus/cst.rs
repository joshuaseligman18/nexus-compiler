use std::{collections::HashMap};

use log::{info, debug};
use petgraph::{graph::{NodeIndex, Graph, WalkNeighbors}, dot::{Dot, Config}, prelude::EdgeIndex};

use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{Window, Document, HtmlTextAreaElement, Element, DomTokenList};

use crate::{nexus::cst_node::{CstNode, NonTerminals, CstNodeTypes}, util::nexus_log};

use string_builder::Builder;

// Code from https://github.com/rustwasm/wasm-bindgen/blob/main/examples/import_js/crate/src/lib.rs
// Have to import the treeRenderer js module
#[wasm_bindgen(module = "/treeRenderer.js")]
extern "C" {
    // Import the createCst function from js so we can call it from the Rust code
    #[wasm_bindgen(js_name = "createCst")]
    fn create_cst_rendering(dotSrc: &str, svgId: &str);
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
        Cst::clear_display();
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

    pub fn display(&self, program_number: &u32) {
        let svg_id: String = self.create_display_area(program_number);

        let cst_string: String = self.create_text();
        // Get the preliminary objects
        let window: Window = web_sys::window().expect("Should be able to get the window");
        let document: Document = window.document().expect("Should be able to get the document");
        let text_area_cst: HtmlTextAreaElement = document.get_element_by_id(format!("program{}-cst-text", *program_number).as_str())
                                                    .expect("Should be able to get the textarea")
                                                    .dyn_into::<HtmlTextAreaElement>()
                                                    .expect("Should be able to convert to textarea");

        text_area_cst.set_value(&cst_string);



        // Draw the image to the webpage
        self.create_image(svg_id);
    }

    fn create_text(&self) -> String {
        let mut tree_builder: Builder = Builder::default();

        self.create_text_dfs(&mut tree_builder, self.root.unwrap(), 0);

        return tree_builder.string().unwrap();
    }

    fn create_text_dfs(&self, builder: &mut Builder, cur_id: usize, level: usize) {
        // Set the level
        for i in 0..level {
            builder.append("-");
        }
        
        // Set the appropriate text output
        match self.graph.node_weight(NodeIndex::new(cur_id)).unwrap() {
            CstNode::Terminal(token) => builder.append(format!("[{}]\n", token.text)),
            CstNode::NonTerminal(non_terminal) => builder.append(format!("<{}>\n", non_terminal))
        }
        
        // Get the neighbors (children) of the current node
        let neighbors: Vec<NodeIndex> = self.graph.neighbors(NodeIndex::new(cur_id)).collect();

        // Loop through them and perform a dfs on each child
        for neighbor_index in neighbors.into_iter().rev() {
            self.create_text_dfs(builder, neighbor_index.index(), level + 1);
        }
    }

    // Function that creates 
    fn create_image(&self, svg_id: String) {
        // Convert the graph into a dot format
        let graph_dot: Dot<&Graph<CstNode, ()>> = Dot::with_config(&self.graph, &[Config::EdgeNoLabel]);
        
        // Call the JS to create the graph on the webpage using d3.js
        create_cst_rendering(format!("{:?}", graph_dot).as_str(), &svg_id);
    }

    fn create_display_area(&self, program_number: &u32) -> String {
        // Get the preliminary objects
        let window: Window = web_sys::window().expect("Should be able to get the window");
        let document: Document = window.document().expect("Should be able to get the document");

        // The ul of the tabs
        let tabs_area: Element = document.get_element_by_id("cst-tabs").expect("Should be able to find the element");
    
        // Create the new tab in the list
        let new_li: Element = document.create_element("li").expect("Should be able to create the li element");

        // Add the appropriate classes
        let li_classes: DomTokenList = new_li.class_list();
        li_classes.add_1("nav-item").expect("Should be able to add the class");
        new_li.set_attribute("role", "presentation").expect("Should be able to add the attribute");

        // From https://getbootstrap.com/docs/4.3/components/navs/
        // <button class="nav-link active" id="home-tab" data-bs-toggle="tab" data-bs-target="#home-tab-pane" type="button" role="tab" aria-controls="home-tab-pane" aria-selected="true">Home</button>

        // Create the button
        let new_button: Element = document.create_element("button").expect("Should be able to create the button");
        let btn_classes: DomTokenList = new_button.class_list();
        btn_classes.add_1("nav-link").expect("Should be able to add the class");

        // Only make the first one active
        if tabs_area.child_element_count() == 0 {
            btn_classes.add_1("active").expect("Should be able to add the class");
            new_button.set_attribute("aria-selected", "true").expect("Should be able to add the attribute");
        } else {
            new_button.set_attribute("aria-selected", "false").expect("Should be able to add the attribute");
        }

        // Set the id of the button
        new_button.set_id(format!("program{}-cst-btn", *program_number).as_str());

        // All of the toggle elements from the example above
        new_button.set_attribute("data-bs-toggle", "tab").expect("Should be able to add the attribute");
        new_button.set_attribute("type", "button").expect("Should be able to add the attribute");
        new_button.set_attribute("role", "tab").expect("Should be able to add the attribute");
        new_button.set_attribute("data-bs-target", format!("#program{}-cst-pane", *program_number).as_str()).expect("Should be able to add the attribute");
        new_button.set_attribute("aria-controls", format!("program{}-cst-pane", *program_number).as_str()).expect("Should be able to add the attribute");

        // Set the inner text
        new_button.set_inner_html(format!("Program {}", *program_number).as_str());

        // Append the button and the list element to the area
        new_li.append_child(&new_button).expect("Should be able to add the child node");
        tabs_area.append_child(&new_li).expect("Should be able to add the child node");

        // Get the content area
        let content_area: Element = document.get_element_by_id("cst-tab-content").expect("Should be able to find the element");

        // Create the individual pane div
        let display_area_div: Element = document.create_element("div").expect("Should be able to create the element");

        // Also from the example link above to only let the first pane initially show and be active
        let display_area_class_list: DomTokenList = display_area_div.class_list();
        display_area_class_list.add_1("tab-pane").expect("Should be able to add the class");
        if content_area.child_element_count() == 0 {
            display_area_class_list.add_2("show", "active").expect("Should be able to add the classes");
        }

        // Add the appropriate attributes
        display_area_div.set_attribute("role", "tabpanel").expect("Should be able to add the attribute");
        display_area_div.set_attribute("tabindex", "0").expect("Should be able to add the attribute");
        display_area_div.set_attribute("aria-labeledby", format!("program{}-cst-btn", *program_number).as_str()).expect("Should be able to add the attribute");

        // Set the id of the pane
        display_area_div.set_id(format!("program{}-cst-pane", *program_number).as_str());

        // The div is a container for the content of the cst info
        display_area_class_list.add_2("container", "cst-pane").expect("Should be able to add the classes");

        // Single row container
        let row_div: Element = document.create_element("div").expect("Should be able to create the div");
        row_div.set_class_name("row");
        
        // The text area is needed for the text representation
        let cst_text_area: HtmlTextAreaElement = document.create_element("textarea")
                                                    .expect("Should be able to create the textarea")
                                                    .dyn_into::<HtmlTextAreaElement>()
                                                    .expect("Should be able to convert to textarea");

        // Set the appropriate styles and general information
        let cst_text_classes: DomTokenList = cst_text_area.class_list();
        cst_text_classes.add_2("col-4", "cst-text").expect("Should be able to add the classes");
        cst_text_area.set_read_only(true);
        cst_text_area.set_id(format!("program{}-cst-text", *program_number).as_str());
        row_div.append_child(&cst_text_area).expect("Should be able to add child node");

        // The div for the svg where d3 will render the graph
        let svg_div_elem: Element = document.create_element("div").expect("Should be able to create the element");
        let svg_classes: DomTokenList = svg_div_elem.class_list();
        svg_classes.add_2("col-8", "cst-svg-div").expect("Should be able to add the classes");
        svg_div_elem.set_id(format!("program{}-cst-svg-div", *program_number).as_str());
        row_div.append_child(&svg_div_elem).expect("Should be able to add child node");

        // Add the row to the container
        display_area_div.append_child(&row_div).expect("Should be able to append child");

        // Add the div to the pane
        content_area.append_child(&display_area_div).expect("Should be able to add the child node");

        // Return the id of the svg div for use by d3
        return svg_div_elem.id();
    }

    // Resets the CST and clears everything in it
    pub fn reset(&mut self) {
        self.graph.clear();
        self.parents.clear();
        self.current = None;
        self.root = None;
    }

    pub fn clear_display() {
        // Get the preliminary objects
        let window: Window = web_sys::window().expect("Should be able to get the window");
        let document: Document = window.document().expect("Should be able to get the document");

        // Clear the entire area
        let tabs_area: Element = document.get_element_by_id("cst-tabs").expect("Should be able to find the element");
        tabs_area.set_inner_html("");
        let content_area: Element = document.get_element_by_id("cst-tab-content").expect("Should be able to find the element");
        content_area.set_inner_html("");
    }
}