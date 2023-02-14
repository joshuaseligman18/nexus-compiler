// Code from https://github.com/magjac/d3-graphviz
export function createCst(dotSrc, svgId) {
    const graphContainer = d3.select(`#${svgId}`).attr('width', '100%').attr('height', '100%');
    graphContainer.graphviz().width(graphContainer.width).height(graphContainer.height).renderDot(dotSrc);    
}