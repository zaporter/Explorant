import React, { useState } from 'react';
import StringCompletionInput from './StringCompletionInput.js';
import { Tutorial, ModuleEditorHelp } from '../tutorials.js';

const ModuleEditor = (props) => {
  const [parent, setParent] = useState("");
  const [current, setCurrent] = useState("");
  const [isInserting, setInserting] = useState(true);

  let validModules = Object.keys(props.nodesData.modules);
  const handleCurrentChange = (event) => {
    let val = event.target.value;
    let this_module = props.nodesData.modules[val];
    if (this_module != null) {
      setInserting(false);
      console.log(this_module);
      if (this_module.parent != null) {
        setParent(this_module.parent);
      }
    }else{
      setInserting(true);
    }
    setCurrent(val);
  }
  const onUpdateParent = (new_val) => {
    setParent(new_val);
  }
  const handleInsertModule = () => {
    if (parent==current) {
      console.error("Parent cannot equal Child module. Exiting module creator")
      props.onClose();
      return;
    }
    if (current=="") {
      console.error("You cannot update the empty module. That is almost certainly a terrible idea.")
      props.onClose();
      return;
    }

    let update_raw_fn = (raw_n_data) => {
      if (isInserting){
        raw_n_data.modules[current] = {
          name: current,
          parent: parent,
          module_attributes: {}
        }
      }else{
        raw_n_data.modules[current].parent = parent;
      }
      raw_n_data.rerun_level = 1;
      return raw_n_data;
    }
    props.updateNodeData(update_raw_fn);
    props.onClose();

  }
  return (
    <div className="node-editor">

      <div className="tutorial-div">
        <h3>{"Module Editor: "}</h3>
        <Tutorial><ModuleEditorHelp /></Tutorial>
      </div>
      <label className="node-editor__label">
        Module Name:
        <input className="node-editor__input" type="text" value={current} onChange={handleCurrentChange} />
      </label>

      <label className="node-editor__label">
        Parent Module:
        <StringCompletionInput
          key={parent}
          default={parent}
          onUpdate={onUpdateParent}
          list={validModules}
        />
      </label>


        <button className="node-editor__button" onClick={handleInsertModule}>{isInserting?"Insert Module":"Update Module"}</button>
      </div>
  )
}

export default ModuleEditor;
