import * as React from 'react';
import { useEffect, useMemo } from 'react';
import {  graphviz} from 'd3-graphviz';
import useRemoteResource from '../util.js';
import * as d3 from "d3";

let counter = 0;
const getId = () => `graphviz${counter++}`;

const GraphViewer = (props) => {
  const id = useMemo(getId, []);
  const [dotSrc, setDotSrc] = useRemoteResource({version:0,dot:`digraph { graph [label="No Loaded Graph"] }`}, {}, 'current_graph');
const defaultOptions = {
  fit: true,
  height: 500,
  width: 500,
  zoom: false,
};
var dotSrcEx = `
digraph {
    graph [label="Click on a node or an edge to delete it" labelloc="t", fontsize="20.0" tooltip=" "]
    node [style="filled"]
    Node1 [id="NodeId1" label="N1" fillcolor="#d62728"]
    Node2 [id="NodeId2" label="N2" fillcolor="#1f77b4"]
    Node3 [id="NodeId3" label="N3" fillcolor="#2ca02c"]
    Node4 [id="NodeId4" label="N4" fillcolor="#ff7f0e"]
    Node1 -> Node2 [id="EdgeId12" label="E14331231"]
    Node1 -> Node3 [id="EdgeId131" label="E13"]
    Node2 -> Node3 [id="EdgeId23" label="E23"]
    Node3 -> Node4 [id="EdgeId34" label="E34"]
}
`;
  //const [dotSrc, setDotSrc] = React.useState(dotSrcEx);
  // useEffect(() => {
  //   graphviz(`#${id}`, {
  //     ...defaultOptions,
  //     // ...options,
  //   }).renderDot(dotSrc);
  // }, []);
const interactive = () => {
    console.log("interactive");
    let nodes = d3.selectAll('.node,.edge');
    nodes
        .on("click", function () { 
          console.log("clicked");
          setDotSrc({version:1,dot:`
digraph {
    node [style="filled"]
    Node1 [id="NodeId1" label="N1" fillcolor="#d62728"]
    Node2 [id="NodeId2" label="N2" fillcolor="#1f77b4"]
    Node3 [id="NodeId3" label="N3" fillcolor="#2ca02c"]
    Node4 [id="NodeId4" label="N4" fillcolor="#ff7f0e"]
    Node1 -> Node3 [id="EdgeId131" label="E13"]
    Node2 -> Node3 [id="EdgeId23" label="E23"]
    Node3 -> Node4 [id="EdgeId34" label="E34"]
}

          `});
        });
  }
  useEffect(() => {

    const gviz = graphviz(`#${id}`, {...defaultOptions});
    gviz.transition(function() {
            return d3.transition()
                .delay(100)
                .duration(1000);
        }).renderDot(dotSrc.dot).on("end",interactive);

  }, [dotSrc]);

  return <div className={"test"} id={id} />;
}

export default GraphViewer;
