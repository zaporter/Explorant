import * as React from 'react';
import { useEffect, useMemo } from 'react';
import {  graphviz} from 'd3-graphviz';
import {useRemoteResource} from '../util.js';
import {callRemote} from '../util.js';
import * as d3 from "d3";

let counter = 0;
const getId = () => `graphviz${counter++}`;

const GraphViewer = (props) => {
  const [height, setHeight] = React.useState(10);
  const [width, setWidth] = React.useState(10);

  useEffect(() => {
        const resizeObserver = new ResizeObserver((event) => {
            // Depending on the layout, you may need to swap inlineSize with blockSize
            // https://developer.mozilla.org/en-US/docs/Web/API/ResizeObserverEntry/contentBoxSize
            setWidth(event[0].contentBoxSize[0].inlineSize);
            setHeight(event[0].contentBoxSize[0].blockSize);
        });

        resizeObserver.observe(document.getElementById("sizeDiv"));
    });
  // const sizeDiv = React.useCallback(node => {
  //   if (node !== null) {
  //     setHeight(node.getBoundingClientRect().height);
  //     setWidth(node.getBoundingClientRect().width);
  //   }
  // }, []);
  const setCurrentFile = props.setCurrentFile;
  const id = useMemo(getId, []);
  const [dotSrc, setDotSrc] = useRemoteResource({version:0,dot:`digraph { graph [label="No Loaded Graph"] }`}, {}, 'current_graph');
  const [nodesData, setNodeData] = useRemoteResource({}, {}, 'node_data');
  const defaultOptions = {
    fit: false,
    height: width,
    width: width,
    zoom: true,
  };
  //const [dotSrc, setDotSrc] = React.useState(dotSrcEx);
  // useEffect(() => {
  //   graphviz(`#${id}`, {
  //     ...defaultOptions,
  //     // ...options,
  //   }).renderDot(dotSrc);
  // }, []);
  const update_settings_and_redraw = (new_settings) => {
      callRemote({"settings":new_settings}, "set_settings")
      .then(
        callRemote({}, "current_graph")
          .then(response=>response.json())
          .then(data=>setDotSrc(data))
      );
  }
  const interactive = () => {
      console.log("interactive");
      let nodes = d3.selectAll('.node,.edge');
      nodes
          .on("click", function () { 
            console.log(this);
            let key = parseInt(this.__data__.key.substring(1));
            console.log(key);
            console.log("clicked");
            if (nodesData.nodes == null){
              console.log("Cannot select node as nodesData is not loaded");
              return;
            }
            let respective = nodesData.nodes[key];
            console.log(respective.location);
            callRemote({}, "get_settings")
              .then(response=>response.json())
              .then(old_settings=>{old_settings.selected_node_id=key; return old_settings})
              .then(new_settings => update_settings_and_redraw(new_settings));
            setCurrentFile({file:respective.location.file,line:respective.location.line_num});
          // callRemote({}, "get_settings")
          //   .then(response=>response.json())
          //   .then(old_settings=>console.log(old_settings))
          // setDotSrc({version:1,dot:`
// digraph {
    // node [style="filled"]
    // Node1 [id="NodeId1" label="N1" fillcolor="#d62728"]
    // Node2 [id="NodeId2" label="N2" fillcolor="#1f77b4"]
    // Node3 [id="NodeId3" label="N3" fillcolor="#2ca02c"]
    // Node4 [id="NodeId4" label="N4" fillcolor="#ff7f0e"]
    // Node1 -> Node3 [id="EdgeId131" label="E13"]
    // Node2 -> Node3 [id="EdgeId23" label="E23"]
    // Node3 -> Node4 [id="EdgeId34" label="E34"]
// }

          // `});
        });
  }
  useEffect(() => {

    const gviz = graphviz(`#${id}`, {...defaultOptions});
    gviz.transition(function() {
            return d3.transition()
                .delay(0)
                .duration(100);
        }).renderDot(dotSrc.dot).on("end",interactive);
    // gviz.renderDot(dotSrc.dot).on("end",interactive);

  }, [dotSrc]);

return (<div className='graph-outer' id="sizeDiv">
  <h3>{"GraphViz Viewer"}</h3>
    <div className="graph-viewer" id={id} />
  </div>
);
}

export default GraphViewer;
