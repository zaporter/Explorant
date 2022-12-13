import React, { useState } from 'react';
import TextModal from './TextModal.js';
import { Tutorial, EventLoaderHelp } from '../tutorials.js';

const EventLoader = (props) => {
  const [modalText, setModalText] = useState(null);
  const downloadNodes = () => {
    const element = document.createElement("a");
    const file = new Blob([JSON.stringify(props.rawNodesData)], { type: 'text/plain;charset=utf-8' });
    element.href = URL.createObjectURL(file);
    element.download = "event-data.json";
    document.body.appendChild(element);
    element.click();
  }
  const readFile = async (e) => {
    const reader = new FileReader()
    reader.onload = async (e) => {
      const text = (e.target.result)
      let new_raw_data = JSON.parse(text);

      let update_raw_fn = () => {
        new_raw_data.rerun_level = 0;
        return new_raw_data;
      }
      props.updateNodeData(update_raw_fn);

    };
    reader.readAsText(e.target.files[0])
    setModalText(
      <div>
        <h2 style={{ color: "green" }}>{"Successfully Imported File!"}</h2>
      </div>)
  }
  return (<div className="box-wrapper">
    {modalText && <TextModal onClose={() => { setModalText(null) }}>{modalText}</TextModal>}

    <div className="tutorial-div">
      <h3>{"Event Saver/Loader"}</h3>
      <Tutorial><EventLoaderHelp /></Tutorial>
    </div>
    <br />
    <div style={{ display: "inline-flex", gap: "60px" }}>
      <div>
        <p>{`Number of nodes: ${Object.keys(props.rawNodesData.nodes).length}`}</p>
        <p>{`Number of modules: ${Object.keys(props.rawNodesData.modules).length}`}</p>
      </div>
      <div>
        <button className="node-editor__button" onClick={downloadNodes}>{"Export Events to File 📝"}</button>

        <br />
        <br />
        <div>
          <label htmlFor="e-upload" className="node-editor__button" >{"Inport Events from File 📄"}</label>
          <input 
            style={{opacity:"0"}}
            type="file" 
            accept=".json"
            id="e-upload"
            name='e-upload' 
            onChange={readFile} />
        </div>
      </div>
    </div>

  </div>);



};

export default EventLoader;
