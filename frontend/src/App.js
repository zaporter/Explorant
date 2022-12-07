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
// import { Graphviz } from 'graphviz-react';
// const useRemoteResource = (defaultVal, requestBody, endpoint) => {
//   const [count, setCount] = useState(defaultVal);
//   const requestOptions = {
//     method: 'POST',
//     headers: {'Content-Type': 'application/json'},
//     body: JSON.stringify(requestBody)
//   };
//   useEffect(()=>{
//     fetch('http://127.0.0.1:8080/'+endpoint,requestOptions)
//       .then(response=>response.json())
//       .then(data=>setCount(data))
//   },[]);
//   return [count,setCount];
// };

function App() {
  const [count, _setCount] = useRemoteResource(5, { id: 21 }, 'ping');
  const [ip, _setIp] = useRemoteResource(null, { trace_id: 0 }, 'instruction_pointer');
  const [generalInfo, _setGeneralInfo] = useRemoteResource(null, {}, 'general_info');
  const [currentNodeId, setCurrentNodeId] = useState(null);
  const [highlightedNodeIds, setHighlightedNodeIds] = useState([])
  const updateCurrentNode = (newId) => {
    setCurrentNodeId(newId);
  }

  const [nodesData, setNodeData] = useRemoteResource(null, {}, 'node_data');
  const [currentFile, setCurrentFile] = useState({ file: "", line: 0 });
  const [currentExecutionInstances, setCurrentExecutionInstances] = useState([{ frame_time: 0, addr: 0, instance_of_addr: 0 }])
  const [isLoading, setIsLoading] = useState(false);

  const [currentFilePath, setCurrentFilePath] = useState(null);
  const [currentFileLineNum, setCurrentFileLineNum] = useState(1);

  return (
    <div className="App">
      {isLoading && <LoadingModal />}
      <h1 className='title'>TraceTrail / Execumap</h1>
      {
        (generalInfo && nodesData) ? (<SplitLayout>
          <SrcViewer
            nodesData={nodesData}
            currentFilePath={currentFilePath}
            setCurrentFilePath={setCurrentFilePath}
            currentFileLineNum={currentFileLineNum}
            setCurrentFileLineNum={setCurrentFileLineNum} />
          <GraphViewer
            nodesData={nodesData}
            setCurrentFilePath={setCurrentFilePath}
            setCurrentFileLineNum={setCurrentFileLineNum}
            updateCurrentNode={updateCurrentNode} />
        </SplitLayout>)
          :
          (<p>{"General and node data did not load. Did the backend crash?"}</p>)
      }
      {currentNodeId && nodesData && <div>
        <NodeEditor
          generalInfo={generalInfo}
          nodesData={nodesData}
          currentNodeId={currentNodeId} />
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
      <p>{ip && ip.instruction_pointer}</p>
      {/* <p>{generalInfo && JSON.stringify(generalInfo)}</p> */}
    </div>
  );
}


//         <div style={{display:"flex", flexDirection:"row", width:"100%", flexBasis:"50%"}}>
//           {generalInfo && <SrcViewer currentFile={currentFile}/>}
//           {generalInfo && <GraphViewer setCurrentFile={setCurrentFile} setNodeName={setNodeName}/>}
//         </div>
export default App;
