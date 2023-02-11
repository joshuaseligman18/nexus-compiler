// Code from https://codepen.io/Jarold/pen/PzbYPb

export function createCst(dotSrc) {
    // Parse the dot format into something that can be used
    let graphObj = graphlibDot.parse(dotSrc);

    // Render the graph in the svg image on the webpage
    let renderer = new dagreD3.Renderer();
    renderer.run(graphObj, d3.select('#cst-container'));

    // Update the svg image to fit the new content
    let svg = document.querySelector('#cst-container');
    let bbox = svg.getBBox();
    svg.style.width = bbox.width + 'px';
    svg.style.height = bbox.height + 'px';
}