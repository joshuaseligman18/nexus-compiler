// Function that returns the text in the editor
export function getCodeInput() {
    return editor.getValue();
}

// Function to load the text into the editor
export function loadProgram(newCode) {
    editor.setValue(newCode);
    editor.gotoLine(Number.MAX_SAFE_INTEGER);
}