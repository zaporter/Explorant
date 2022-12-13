import React, { useState } from 'react';
import StringCompletionInput from './StringCompletionInput.js';
import { Tutorial, NodeEditorHelp } from '../tutorials.js';

const NodeEditor = (props) => {
  let nodesData = props.nodesData;
  let nid = props.currentNodeId.id;
  const node = props.currentNodeId.is_raw ? props.rawNodesData.nodes[nid] : nodesData.nodes[nid];
  let validModules = Object.keys(nodesData.modules);
  const [selectedModule, setSelectedModule] = useState(props.mode == "add" ? '' : node.module);
  const onUpdateModule = (new_val) => {
    setSelectedModule(new_val);
  }
  const [name, setName] = useState(props.mode == 'add' ? props.name : node.name);
  const [type, setType] = useState(props.mode == 'add' ? 'Event' : node.node_type);
  const [lineLocation, setLineLocation] = useState(props.mode == 'add' ? props.line : node.location.line_num);

  const handleNameChange = (event) => {
    setName(event.target.value);
  }

  const handleTypeChange = (event) => {
    setType(event.target.value);
  }

  const handleLineLocationChange = (event) => {
    setLineLocation(event.target.value);
  }

  const handleAddNode = () => {
    let update_raw_fn = (raw_n_data) => {
      console.log(raw_n_data);
      let new_node = {
        FQN: "",
        address: 0,
        name: name,
        module: selectedModule,
        node_type: type,
        location: {
          file: props.file,
          line_num: parseInt(lineLocation),
          column_num: 0,
        },
        labeled_transitions: [],
        node_attributes: {},
      };
      raw_n_data.nodes[0] = (new_node);
      raw_n_data.rerun_level = 0;
      return raw_n_data;
    }
    props.updateNodeData(update_raw_fn);
    props.onClose();
  }

  const handleSaveNode = () => {
    let addr = null;
    if (!props.currentNodeId.is_raw) {
      addr = nodesData.nodes[nid].address;
    }else {
      addr = props.rawNodesData.nodes[nid].address;
    }
    let update_raw_fn = (raw_n_data) => {
      raw_n_data.nodes[addr].name = name;
      raw_n_data.nodes[addr].module = selectedModule;
      raw_n_data.nodes[addr].node_type = type;
      raw_n_data.nodes[addr].location.line_num = parseInt(lineLocation);
      raw_n_data.rerun_level = 0;
      return raw_n_data;
    }
    props.updateNodeData(update_raw_fn);
  }

  const handleDeleteNode = () => {
    let addr = null;
    if (!props.currentNodeId.is_raw) {
      addr = nodesData.nodes[nid].address;
    }else {
      addr = props.rawNodesData.nodes[nid].address;
    }
    let update_raw_fn = (raw_n_data) => {
      delete raw_n_data.nodes[addr];

      raw_n_data.rerun_level = 0;
      return raw_n_data;
    }
    props.updateNodeData(update_raw_fn);
    // delete node logic
  }

  return (
    <div className="node-editor">

      <div className="tutorial-div">
        {props.mode == 'add' ? (
          <h3>{"Add Event"}</h3>) : (<h3> {"Edit Event"}</h3>)
        }
        <Tutorial><NodeEditorHelp /></Tutorial>
      </div>
      <label className="node-editor__label">
        Name:
        <input className="node-editor__input" type="text" value={name} onChange={handleNameChange} />
      </label>

      <label className="node-editor__label">
        Module:
        <StringCompletionInput
          default={selectedModule}
          onUpdate={onUpdateModule}
          list={validModules}
        />
      </label>
      <label className="node-editor__label">
        Type:
        <select className="node-editor__input" value={type} onChange={handleTypeChange}>
          <option value="Event">Event</option>
          <option value="Flow">Flow</option>
        </select>
      </label>
      <label className="node-editor__label">
        Line Location:
        <input className="node-editor__input" type="number" value={lineLocation} onChange={handleLineLocationChange} />
      </label>
      {
        props.mode === 'add' ? (
          <button className="node-editor__button" onClick={handleAddNode}>Add Node</button>
        ) : (
          <div className="node-editor__buttons">
            <button className="node-editor__button" onClick={handleSaveNode}>Save Node</button>
            <button className="node-editor__button" onClick={handleDeleteNode}>Delete Node</button>
          </div>
        )
      }
    </div>
  );
}

export default NodeEditor;
