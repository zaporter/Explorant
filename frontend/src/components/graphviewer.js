import * as React from 'react';
import { unstable_batchedUpdates } from 'react-dom';
import { useEffect, useMemo } from 'react';
import { graphviz } from 'd3-graphviz';
import { useRemoteResource } from '../util.js';
import { callRemote } from '../util.js';
import LoadingModal from './LoadingModal.js';
import * as d3 from "d3";
import { Tutorial, GraphViewerHelp } from '../tutorials.js';
import Switch from "react-switch";

let counter = 0;
const getId = () => `graphviz${counter++}`;

const GraphViewer = (props) => {
  const [height, setHeight] = React.useState(10);
  const [width, setWidth] = React.useState(10);
  const [isLoading, setIsLoading] = React.useState(false);
  var updateCurrentNode_int = props.updateCurrentNode;

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
  const id = useMemo(getId, []);
  const [graphVer, setGraphVer] = React.useState(0);
  const [dotSrc, _setDotSrc] = useRemoteResource({ version: 0, dot: `digraph { graph [label="No Loaded Graph"] }` }, {}, 'current_graph', [props.nodesData, graphVer,null]);
  const [initialSettings, setSettings] = useRemoteResource({show_unreachable_nodes:false, selected_node_id:0}, {}, 'get_settings', []);

  const defaultOptions = {
    fit: false,
    height: width,
    width: width,
    zoom: true,
  };
  const interactive = () => {
    console.log("interactive");
    setIsLoading(false);
    let clusters = d3.selectAll('.cluster');
    clusters
      .on("click", function() {
        let textContent = this.textContent;
        let lines = textContent.split("\n");
        let name = lines[3];
        console.log(name);

        let update_raw_fn = (raw_n_data) => {
          let is_collapsed = raw_n_data.modules[name].module_attributes["collapsed"];
          if (is_collapsed == null) {
            is_collapsed = "false";
          }
          let new_val = is_collapsed == "true" ? "false" : "true"; 
          raw_n_data.modules[name].module_attributes["collapsed"] = new_val;
          raw_n_data.rerun_level = 2;
          return raw_n_data;
        }
        props.updateNodeData(update_raw_fn);
      })
    let nodes = d3.selectAll('.node,.edge');
    nodes
      .on("click", function() {
        console.log(this);
        let is_raw = this.__data__.key.startsWith("U");
        let is_collapsed = this.__data__.key.startsWith("C");
        let key = parseInt(this.__data__.key.substring(1));
        console.log(is_raw);
        console.log(key);
        if (props.nodesData.nodes == null) {
          console.log("Cannot select node as nodesData is not loaded");
          return;
        }
        let respective = null;
        if (is_raw) {
          respective = props.rawNodesData.nodes[key];
          console.log(props.rawNodesData.nodes);
        }else {
          respective = props.nodesData.nodes[key];
        }

        callRemote({}, "get_settings")
          .then(response => response.json())
          .then(old_settings => { old_settings.selected_node_id = key; return old_settings })
          .then(new_settings => callRemote({ "settings": new_settings }, "set_settings")//.then(
            .then(_ => unstable_batchedUpdates(() => {
              setGraphVer(graphVer + 1)
              if (!is_collapsed && respective != null) {
                updateCurrentNode_int({id:key,is_raw:is_raw})
                props.setCurrentFilePath(respective.location.file)
                props.setCurrentFileLineNum(respective.location.line_num)
              }
            })))
        //) 
        // .then(_ => updateCurrentNode_int(key))
      });
  }
  const handleShowUnreachableNodes=(checked)=> {
        callRemote({}, "get_settings")
          .then(response => response.json())
          .then(old_settings => { old_settings.show_unreachable_nodes = checked; return old_settings })
          .then(new_settings => {setSettings(new_settings); return new_settings})
          .then(new_settings => callRemote({ "settings": new_settings }, "set_settings")//.then(
            .then(_ => unstable_batchedUpdates(() => {
              setGraphVer(graphVer + 1)

            })))
  }
  useEffect(() => {
    const gviz = graphviz(`#${id}`, { ...defaultOptions });
    gviz.transition(function() {
      return d3.transition()
        .delay(0)
        .duration(100);
    }).renderDot(dotSrc.dot).on("end", interactive);

  }, [dotSrc,graphVer,null]);

  useEffect(()=>{
    setGraphVer(graphVer+1);
  },[]);

  return (
    <div className="box-wrapper">
      <div className='graph-outer' id="sizeDiv">
        <div className="tutorial-div">
          <h3>{"Graph Viewer"}</h3>
          <Tutorial><GraphViewerHelp /></Tutorial>
        </div>
        <button onClick={()=>{setGraphVer(graphVer+1)}}>↻</button>
        <div className="graph-viewer" id={id} />
        <div style={{display:"inline-flex", gap:"20px"}}>
          <p> Display unreachable events: </p>
          <div style={{padding:"0.9em 0em"}}>
          <Switch onChange={handleShowUnreachableNodes} checked={initialSettings.show_unreachable_nodes} />
          </div>
          </div>
      </div>
    </div>
  );
}

export default GraphViewer;
