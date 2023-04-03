// Function that returns the text in the editor
export function getCodeInput() {
    return editor.getValue();
}

// Function to load the text into the editor
export function loadProgram(newCode) {
    editor.setValue(newCode);
    editor.gotoLine(Number.MAX_SAFE_INTEGER);
}

// Uses the clipboard api to set the device's clipboard
// From https://www.freecodecamp.org/news/copy-text-to-clipboard-javascript/
export function setClipboard(newText) {
    navigator.clipboard.writeText(newText);
}
