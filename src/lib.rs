use wasm_bindgen::{prelude::*, JsCast};
use log::{Level, info, debug};
use web_sys::{Window, Document, HtmlTextAreaElement, HtmlElement};

mod nexus;
mod util;

use crate::nexus::compiler;
use crate::util::nexus_log;

// Function to initialize Nexus
#[wasm_bindgen]
pub fn nexus_init() {
    // Set up console logs for debugging
    console_log::init_with_level(Level::Debug).expect("Should be able to connect to the browser's console");
    console_error_panic_hook::set_once();

    // Grab the window and document elements for DOM manipulation
    let window: Window = web_sys::window().expect("The window object should exist.");
    let document: Document = window.document().expect("The document object should exist");

    // Set up the event listeners
    set_up_dom(&document);

    info!("Nexus initialized");
}

// Function used to set up all interactive elements in the webpage
fn set_up_dom(document: &Document) {
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
        debug!("Compile called");
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
        debug!("Clear called");
    }) as Box<dyn FnMut()>);

    clear_btn.add_event_listener_with_callback("click", clear_btn_fn.as_ref().unchecked_ref()).expect("Should be able to add the event listener");
    clear_btn_fn.forget();
}