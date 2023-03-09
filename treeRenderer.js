// Keeps track of the best dimensions so all images can be rendered
let realDim = [0, 0];

export function createCst(dotSrc, svgId) {
    // Get the width and height of the container
    let width = document.querySelector(`#${svgId}`).offsetWidth;
    let height = document.querySelector(`#${svgId}`).offsetHeight;

    // Width and height of 0 means that there was another program that was successful
    if (width === 0 && height === 0) {
        // Use the same dimensions as that program
        width = realDim[0];
        height = realDim[1];
    } else {
        // If there are new dimensions, then we want to save if for future use
        realDim[0] = width;
        realDim[1] = height;
    }
    
    // Render the new image within the container
    // Code from https://github.com/magjac/d3-graphviz
    d3.select(`#${svgId}`).graphviz().width(width).height(height).renderDot(dotSrc);
}