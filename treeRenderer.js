// Keeps track of the best dimensions so all images can be rendered
let realCstDim = [0, 0];
let realAstDim = [0, 0];

export function createCst(dotSrc, svgId) {
    // Get the width and height of the container
    let width = document.querySelector(`#${svgId}`).offsetWidth;
    let height = document.querySelector(`#${svgId}`).offsetHeight;

    // Width and height of 0 means that there was another program that was successful
    if (width === 0 && height === 0) {
        if (svgId.includes('cst')) {
            // Use the cst dimensions
            width = realCstDim[0];
            height = realCstDim[1];
        } else {
            width = realAstDim[0];
            height = realAstDim[1];
        }
    } else {
        if (svgId.includes('cst')) {
            // Store the dimensions for future CSTs
            realCstDim[0] = width;
            realCstDim[1] = height;
        } else {
            // Store the dimensions for future ASTs
            realAstDim[0] = width;
            realAstDim[1] = height;
        }
    }
    
    // Render the new image within the container
    // Code from https://github.com/magjac/d3-graphviz
    d3.select(`#${svgId}`).graphviz().width(width).height(height).renderDot(dotSrc);
}