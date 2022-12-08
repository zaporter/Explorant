import React from "react";
import NodeEditor from "./NodeEditor";

const AddNodeModal = (props) => {
  return (
    <div className="add-node-modal-container">
      <div className="add-node-modal-background" ></div>
      <div className="add-node-modal-content">
        <NodeEditor mode={"add"} {...props.child_props}/>
        <button className="add-node-modal-close" onClick={props.onClose}>Close</button>
      </div>
    </div>
  );
};

export default AddNodeModal;
