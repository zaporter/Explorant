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
import EventLoader from './components/EventLoader.js';
import { useRemoteResource } from './util.js';
import { callRemote } from './util.js';

function App() {
  const [generalInfo, _setGeneralInfo] = useRemoteResource(null, {}, 'general_info');
  const [currentNodeId, setCurrentNodeId] = useState({ id: null, is_raw: false });
  const updateCurrentNode = (newId) => {
    setCurrentNodeId(newId);
  }
  const [nodesData, setNodeData] = useRemoteResource(null, {}, 'node_data');
  const [rawNodesData, setRawNodeData] = useRemoteResource(null, {}, 'get_raw_nodes_and_modules', [nodesData]);
  const [isLoading, setIsLoading] = useState(false);

  const [currentFilePath, setCurrentFilePath] = useState("[none selected]");
  const [currentFileLineNum, setCurrentFileLineNum] = useState(1);


  const updateNodeData = (update_raw_fn) => {
    callRemote({}, 'get_raw_nodes_and_modules')
      .then(resp => resp.json())
      .then(dta => update_raw_fn(dta))
      .then(dta => callRemote(dta, 'update_raw_nodes_and_modules')
        .then(_ => callRemote({}, 'node_data').then(resp => resp.json()).then(resp => { console.log(resp); setNodeData(resp) })

          .then(_ => updateCurrentNode({ id: null, is_raw: false }))
        )
      )
  }

  return (
    <div className="App">
      {isLoading && <LoadingModal />}
      <div style={{margin:"1rem", gap:"1rem", display: "flex", justifyContent: "center" }}>
        <h1 className='title'>Explorant </h1>
        <img style={{ width: "5rem" , height: "5rem"}} src="logo_zoomed.png" />
      </div>
      {generalInfo &&
        <p className='subtitle'>{generalInfo.recording_dir}</p>
      }
      {
        (generalInfo && nodesData) ? (
          <SplitLayout>
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
              updateNodeData={updateNodeData}
              rawNodesData={rawNodesData}
              setCurrentFilePath={setCurrentFilePath}
              setCurrentFileLineNum={setCurrentFileLineNum}
              updateCurrentNode={updateCurrentNode} />
          </SplitLayout>)
          :
          (<h3 style={{}}>{"General and node data did not load. Did the backend crash? Retrying..."}</h3>)
      }
      {currentNodeId.id != null && nodesData &&
        <SplitLayout
          default_split={25}
        >
          <div className='box-wrapper'>
            <NodeEditor
              key={currentNodeId.id}
              mode={"edit"}
              generalInfo={generalInfo}
              updateNodeData={updateNodeData}
              nodesData={nodesData}
              rawNodesData={rawNodesData}
              currentNodeId={currentNodeId} />
          </div>
          {!currentNodeId.is_raw && <ExecutionInstanceList
            nodesData={nodesData}
            generalInfo={generalInfo}
            currentNodeId={currentNodeId} />}
        </SplitLayout>

      }

      {generalInfo && rawNodesData &&
        <EventLoader
          generalInfo={generalInfo}
          updateNodeData={updateNodeData}
          rawNodesData={rawNodesData}
        />
      }

    </div>
  );
}


//         <div style={{display:"flex", flexDirection:"row", width:"100%", flexBasis:"50%"}}>
//           {generalInfo && <SrcViewer currentFile={currentFile}/>}
//           {generalInfo && <GraphViewer setCurrentFile={setCurrentFile} setNodeName={setNodeName}/>}
//         </div>
export default App;
