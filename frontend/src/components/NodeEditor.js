import React, { useState } from 'react';

const NodeEditor = (props) => {
  const [name, setName] = useState('');
  const [type, setType] = useState('');
  const [lineLocation, setLineLocation] = useState(0);

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
    // add node logic
  }

  const handleSaveNode = () => {
    // save node logic
  }

  const handleDeleteNode = () => {
    // delete node logic
  }

  return (
  <div className="box-wrapper">
    <div className="node-editor">
      <h3>{"Edit Node:"}</h3>
      <label className="node-editor__label">
        Name:
        <input className="node-editor__input" type="text" value={name} onChange={handleNameChange} />
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
    </div>
  );
}

export default NodeEditor;
