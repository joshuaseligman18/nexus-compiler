use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::{Document, HtmlTextAreaElement, HtmlElement};

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
}