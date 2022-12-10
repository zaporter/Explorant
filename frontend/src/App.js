import logo from './logo.svg';
import './App.css';
import React, { useEffect, useState, useContext } from 'react';
import LaneViewer from './components/LaneViewer.js';
import NodeEditor from './components/NodeEditor.js';
import SrcViewer from './components/srcviewer.js';
import SplitLayout from './components/SplitLayout.js';
import LoadingModal from './components/LoadingModal.js';
import ExecutionInstanceList from './components/ExecutionInstanceList.js';
import GraphViewer from './components/graphviewer.js';
import { useRemoteResource } from './util.js';
import { callRemote } from './util.js';

function App() {
  const [count, _setCount] = useRemoteResource(5, { id: 21 }, 'ping');
  const [generalInfo, _setGeneralInfo] = useRemoteResource(null, {}, 'general_info');
  const [currentNodeId, setCurrentNodeId] = useState(null);
  const [highlightedNodeIds, setHighlightedNodeIds] = useState([])
  const updateCurrentNode = (newId) => {
    setCurrentNodeId(newId);
  }
  const [nodesData, setNodeData] = useRemoteResource(null, {}, 'node_data');
  const [currentExecutionInstances, setCurrentExecutionInstances] = useState([{ frame_time: 0, addr: 0, instance_of_addr: 0 }])
  const [isLoading, setIsLoading] = useState(false);

  const [currentFilePath, setCurrentFilePath] = useState("[none selected]");
  const [currentFileLineNum, setCurrentFileLineNum] = useState(1);
  

  const updateNodeData = (update_raw_fn) => {
    console.log("UPDATED");
    callRemote({}, 'get_raw_nodes_and_modules')
      .then(resp => resp.json())
      .then(dta => update_raw_fn(dta))
      .then(dta => callRemote(dta,'update_raw_nodes_and_modules')
        .then(callRemote({},'node_data').then(resp=>resp.json()).then(resp=>setNodeData(resp)))
        .then(_=>updateCurrentNode(null))
      )
  }

  return (
    <div className="App">
      {isLoading && <LoadingModal />}
      <h1 className='title'>TraceTrail / Execumap</h1>
      {
        (generalInfo && nodesData) ? (<SplitLayout>
          <SrcViewer
            key={nodesData}
            nodesData={nodesData}
            currentFilePath={currentFilePath}
            setCurrentFilePath={setCurrentFilePath}
            currentFileLineNum={currentFileLineNum}
            updateNodeData={updateNodeData}
            setCurrentFileLineNum={setCurrentFileLineNum} />
          <GraphViewer
            key={nodesData}
            nodesData={nodesData}
            setCurrentFilePath={setCurrentFilePath}
            setCurrentFileLineNum={setCurrentFileLineNum}
            updateCurrentNode={updateCurrentNode} />
        </SplitLayout>)
          :
          (<p>{"General and node data did not load. Did the backend crash?"}</p>)
      }
      {currentNodeId && nodesData && <div>
        <div className='box-wrapper'>
        <NodeEditor
          key={currentNodeId}
          mode={"edit"}
          generalInfo={generalInfo}
          updateNodeData={updateNodeData}
          nodesData={nodesData}
          currentNodeId={currentNodeId} />
          </div>
         <ExecutionInstanceList 
           nodesData={nodesData} 
           currentNodeId={currentNodeId}
           setHighlightedNodeIds={setHighlightedNodeIds}
           executionInstances={currentExecutionInstances} 
           setExecutionInstances={setCurrentExecutionInstances} />
        {/* <LaneViewer 
        {/*   generalInfo={generalInfo} */}
        {/*   nodesData={nodesData} */}
        {/*   executionInstances={currentExecutionInstances} /> */}
      </div>}

      {/* {generalInfo && <p> {JSON.stringify(generalInfo)}</p>} */}
      <p>{count.id}</p>
      {/* <p>{generalInfo && JSON.stringify(generalInfo)}</p> */}
    </div>
  );
}


//         <div style={{display:"flex", flexDirection:"row", width:"100%", flexBasis:"50%"}}>
//           {generalInfo && <SrcViewer currentFile={currentFile}/>}
//           {generalInfo && <GraphViewer setCurrentFile={setCurrentFile} setNodeName={setNodeName}/>}
//         </div>
export default App;
