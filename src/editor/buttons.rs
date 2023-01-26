use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::{Document, HtmlTextAreaElement, HtmlElement, Event};

use crate::{nexus::compiler, util::nexus_log};

// Function used to set up all interactive elements in the webpage
pub fn set_up_buttons(document: &Document) {
    // Grab the code input text area
    let code_input: HtmlTextAreaElement = document
        .get_element_by_id("ta-code-input")
        .expect("There should be a ta-code-input element")
        .dyn_into::<HtmlTextAreaElement>()
        .expect("The element should be recognized as a textarea");
    
    // Grab the compile button
    let compile_btn: HtmlElement = document
        .get_element_by_id("compile-btn")
        .expect("There should be an element called compile-btn")
        .dyn_into::<HtmlElement>()
        .expect("Should be able to cast to an HtmlElement object");

    // Create a function that will be used as the event listener and add it to the compile button
    let compile_btn_fn: Closure<dyn FnMut()> = Closure::wrap(Box::new(move || {
        compiler::compile(code_input.value());
    }) as Box<dyn FnMut()>);

    compile_btn.add_event_listener_with_callback("click", compile_btn_fn.as_ref().unchecked_ref()).expect("Should be able to add the event listener");
    compile_btn_fn.forget();

    // Button to clear the logs
    let clear_btn: HtmlElement = document
        .get_element_by_id("clear-btn")
        .expect("There should be an element called clear-btn")
        .dyn_into::<HtmlElement>()
        .expect("Should be able to cast to an HtmlElement object");

    // Create a function that will be used as the event listener and add it to the clear logs button
    let clear_btn_fn: Closure<dyn FnMut()> = Closure::wrap(Box::new(move || {
        nexus_log::clear_logs();
    }) as Box<dyn FnMut()>);

    clear_btn.add_event_listener_with_callback("click", clear_btn_fn.as_ref().unchecked_ref()).expect("Should be able to add the event listener");
    clear_btn_fn.forget();

    // Get each of the log mode buttons
    let nexus_log_mode: HtmlElement = document
        .get_element_by_id("nexus-log-mode")
        .expect("There should be an element called nexus-log-mode")
        .dyn_into::<HtmlElement>()
        .expect("Should be able to cast to an HtmlElement object");

    let lexer_log_mode: HtmlElement = document
        .get_element_by_id("lexer-log-mode")
        .expect("There should be an element called lexer-log-mode")
        .dyn_into::<HtmlElement>()
        .expect("Should be able to cast to an HtmlElement object");

    let parser_log_mode: HtmlElement = document
        .get_element_by_id("parser-log-mode")
        .expect("There should be an element called parser-log-mode")
        .dyn_into::<HtmlElement>()
        .expect("Should be able to cast to an HtmlElement object");

    let semantic_log_mode: HtmlElement = document
        .get_element_by_id("semantic-log-mode")
        .expect("There should be an element called semantic-log-mode")
        .dyn_into::<HtmlElement>()
        .expect("Should be able to cast to an HtmlElement object");

    let codegen_log_mode: HtmlElement = document
        .get_element_by_id("codegen-log-mode")
        .expect("There should be an element called codegen-log-mode")
        .dyn_into::<HtmlElement>()
        .expect("Should be able to cast to an HtmlElement object");

    // Universal function for toggling log mode buttons
    let toggle_log_mode_fn: Closure<dyn FnMut(_)> = Closure::wrap(Box::new(move |e: Event| {
        // Get the element that was clicked
        let target: HtmlElement = e.target().expect("Should be able to get the target").dyn_into::<HtmlElement>().expect("Should be able to cast to an HtmlElement object");
        
        // Swap verbose and simple modes
        match target.class_name().as_str() {
            "verbose" => {
                target.set_class_name("simple");
                target.set_inner_text("Simple");
            },
            "simple" => {
                target.set_class_name("verbose");
                target.set_inner_text("Verbose");
            },
            // Should not be reached
            _ => panic!("Invalid class name")
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