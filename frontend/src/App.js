import logo from './logo.svg';
import './App.css';
import React, {useEffect, useState} from 'react';
import LaneViewer from './components/LaneViewer.js';
import NodeEditor from './components/NodeEditor.js';
import SrcViewer from './components/srcviewer.js';
import GraphViewer from './components/graphviewer.js';
import useRemoteResource from './util.js';
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
  const [count,_setCount] = useRemoteResource(5,{id:21},'ping');
  const [ip,_setIp] = useRemoteResource(null,{trace_id:0},'instruction_pointer');
  const [generalInfo,_setGeneralInfo] = useRemoteResource(null,{},'general_info');
  const [currentFile, setCurrentFile] = useState("");
  return (
    <div className="App">
{/* <Graphviz dot={dot} /> */}
      <h1 className='title'>Trace2State</h1>

        <div style={{display:"flex", flexDirection:"row", width:"100%", flexBasis:"50%"}}>
          {generalInfo && <SrcViewer currentFile={currentFile}/>}
          {generalInfo && <GraphViewer setCurrentFile={setCurrentFile} />}
        </div>
          {generalInfo && <NodeEditor generalInfo={generalInfo}/>}
          {generalInfo && <LaneViewer generalInfo={generalInfo}/>}
        {/* {generalInfo && <p> {JSON.stringify(generalInfo)}</p>} */}
        <p>{count.id}</p>
        <p>{ip&&ip.instruction_pointer}</p>
        {/* <p>{generalInfo && JSON.stringify(generalInfo)}</p> */}
    </div>
  );
}

export default App;
