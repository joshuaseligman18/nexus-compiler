use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::{Document, HtmlElement, Event, Element, DomTokenList};

use crate::{nexus::{compiler, syntax_tree::SyntaxTree}, util::nexus_log};

use wasm_bindgen::prelude::*;

// Have to import the editor js module
#[wasm_bindgen(module = "/editor.js")]
extern "C" {
    // Import the getCodeInput function from js so we can call it from the Rust code
    #[wasm_bindgen(js_name = "getCodeInput")]
    fn get_code_input() -> String;
}

// Function used to set up all interactive elements in the webpage
pub fn set_up_buttons(document: &Document) {    
    // Grab the compile button
    let compile_btn: Element = document
        .get_element_by_id("compile-btn")
        .expect("There should be an element called compile-btn");

    // Create a function that will be used as the event listener and add it to the compile button
    let compile_btn_fn: Closure<dyn FnMut()> = Closure::wrap(Box::new(move || {
        compiler::compile(&get_code_input());
    }) as Box<dyn FnMut()>);

    compile_btn.add_event_listener_with_callback("click", compile_btn_fn.as_ref().unchecked_ref()).expect("Should be able to add the event listener");
    compile_btn_fn.forget();

    // Button to clear the logs
    let clear_btn: Element = document
        .get_element_by_id("clear-btn")
        .expect("There should be an element called clear-btn");

    // Create a function that will be used as the event listener and add it to the clear logs button
    let clear_btn_fn: Closure<dyn FnMut()> = Closure::wrap(Box::new(|| {
        nexus_log::clear_logs();
<<<<<<< HEAD
        SyntaxTree::clear_display();
=======
        Cst::clear_display();
>>>>>>> main
    }) as Box<dyn FnMut()>);

    clear_btn.add_event_listener_with_callback("click", clear_btn_fn.as_ref().unchecked_ref()).expect("Should be able to add the event listener");
    clear_btn_fn.forget();

    // Get the reset button
    let reset_btn: Element = document
        .get_element_by_id("reset-btn")
        .expect("There should be an element called reset-btn");

    // Create the event listener for the reset button
    let reset_btn_fn: Closure<dyn FnMut()> = Closure::wrap(Box::new(|| {
        // Get the window and reload the page
        web_sys::window().expect("Should be able to get the window").location().reload().expect("Should be able to reload the page");
    }) as Box<dyn FnMut()>);

    // Add the event listener to the button
    reset_btn.add_event_listener_with_callback("click", reset_btn_fn.as_ref().unchecked_ref()).expect("Should be able to add the event listener");
    reset_btn_fn.forget();

    // Get each of the log mode buttons
    let nexus_log_mode: Element = document
        .get_element_by_id("nexus-log-mode")
        .expect("There should be an element called nexus-log-mode");

    let lexer_log_mode: Element = document
        .get_element_by_id("lexer-log-mode")
        .expect("There should be an element called lexer-log-mode");

    let parser_log_mode: Element = document
        .get_element_by_id("parser-log-mode")
        .expect("There should be an element called parser-log-mode");

    let semantic_log_mode: Element = document
        .get_element_by_id("semantic-log-mode")
        .expect("There should be an element called semantic-log-mode");

    let codegen_log_mode: Element = document
        .get_element_by_id("codegen-log-mode")
        .expect("There should be an element called codegen-log-mode");

    // Universal function for toggling log mode buttons
    let toggle_log_mode_fn: Closure<dyn FnMut(_)> = Closure::wrap(Box::new(|e: Event| {
        // Get the element that was clicked
        let target: HtmlElement = e.target().expect("Should be able to get the target").dyn_into::<HtmlElement>().expect("Should be able to cast to an HtmlElement object");

        let target_classes: DomTokenList = target.class_list();
        if target_classes.contains("verbose") {
            target_classes.remove_1("verbose").expect("Should be able to remove the class");
            target_classes.add_1("simple").expect("Should be able to add the class");
            target.set_inner_text("Simple");
        } else if target_classes.contains("simple") {
            target_classes.remove_1("simple").expect("Should be able to remove the class");
            target_classes.add_1("verbose").expect("Should be able to add the class");
            target.set_inner_text("Verbose");
        }
    }) as Box<dyn FnMut(_)>);

    // Add the event listener
    nexus_log_mode.add_event_listener_with_callback("click", toggle_log_mode_fn.as_ref().unchecked_ref()).expect("Should be able to add the event listener");
    lexer_log_mode.add_event_listener_with_callback("click", toggle_log_mode_fn.as_ref().unchecked_ref()).expect("Should be able to add the event listener");
    parser_log_mode.add_event_listener_with_callback("click", toggle_log_mode_fn.as_ref().unchecked_ref()).expect("Should be able to add the event listener");
    semantic_log_mode.add_event_listener_with_callback("click", toggle_log_mode_fn.as_ref().unchecked_ref()).expect("Should be able to add the event listener");
    codegen_log_mode.add_event_listener_with_callback("click", toggle_log_mode_fn.as_ref().unchecked_ref()).expect("Should be able to add the event listener");

    toggle_log_mode_fn.forget();
}
