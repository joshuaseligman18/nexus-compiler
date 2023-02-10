// Code from https://codepen.io/Jarold/pen/PzbYPb

export function createGraph(dotSrc) {
    let graphObj = graphlibDot.parse(dotSrc);

    let renderer = new dagreD3.Renderer();
    renderer.run(graphObj, d3.select('svg g'));

    let svg = document.querySelector('#graph-container');
    let bbox = svg.getBBox();
    svg.style.width = bbox.width + 40.0 + "px";
    svg.style.height = bbox.height + 40.0 + "px";
}