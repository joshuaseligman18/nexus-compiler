use log::debug;
use wasm_bindgen::{JsCast, prelude::Closure};
use web_sys::{Document, HtmlSelectElement, HtmlOptionElement, HtmlElement, HtmlTextAreaElement};

use crate::util::test::*;

// Function to set up the test suite
pub fn create_test_environment(document: &Document) {
    let test_options: HtmlSelectElement = document
        .get_element_by_id("tests")
        .expect("There should be a tests element")
        .dyn_into::<HtmlSelectElement>()
        .expect("The element should be recognized as a select element");

    // Grab the compile button
    let load_test_btn: HtmlElement = document
        .get_element_by_id("load-test-btn")
        .expect("There should be an element called load-test-btn")
        .dyn_into::<HtmlElement>()
        .expect("Should be able to cast to an HtmlElement object");

    load_tests(document, &test_options);
    add_test_button_fn(document, &test_options, &load_test_btn)
}

// Function to load the tests into the select element
fn load_tests(document: &Document, test_selection: &HtmlSelectElement) {
    // Get the tests
    let tests: Vec<Test> = get_tests();
    
    // Loop through all of the tests
    for test in tests.iter() {
        // Create the new option element with the given name and value
        let new_option = document
            .create_element("option")
            .expect("Should be able to create the option element")
            .dyn_into::<HtmlOptionElement>()
            .expect("Should be able to cast to option element");
        new_option.set_inner_text(format!("[{}] - {}", test.test_type, test.test_name).as_str());
        new_option.set_value(&test.test_code);

        // Add the option element to the dropdown menu
        test_selection.add_with_html_option_element(&new_option).expect("Should be able to add the option");
    }
}

// Function to set up the tests
fn add_test_button_fn(document: &Document, test_selection: &HtmlSelectElement, load_test_btn: &HtmlElement) {
    // Get the text area to paste the code into
    let code_input: HtmlTextAreaElement = document
        .get_element_by_id("ta-code-input")
        .expect("There should be a ta-code-input element")
        .dyn_into::<HtmlTextAreaElement>()
        .expect("The element should be recognized as a textarea");


    // Get the value to paste
    let test_value: String = test_selection.value();

    // Create a function that will be used as the event listener and add it to the load test button
    let load_test_fn: Closure<dyn FnMut()> = Closure::wrap(Box::new(move || {
        // Paste the value
        code_input.set_value(&test_value);
    }) as Box<dyn FnMut()>);

    load_test_btn.add_event_listener_with_callback("click", load_test_fn.as_ref().unchecked_ref()).expect("Should be able to add the event listener");
    load_test_fn.forget();
}

// Function that returns a vector of all of the tests
fn get_tests() -> Vec<Test> {
    let tests: Vec<Test> = vec![
        Test {
            test_type: TestType::Lex,
            test_name: String::from("Alan's tests"),
            test_code: String::from("{}$\n{{{{{{}}}}}}$\n{{{{{{}}} /* comments are ignored */ }}}}$\n{ /* comments are still ignored */ int @}$\n{\nint a\na = a\nstring b\na=b\n}$")
        },
    ];

    return tests;
}