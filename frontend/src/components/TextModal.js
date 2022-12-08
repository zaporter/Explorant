import React from "react";

const TextModal = (props) => {
  return (
    <div className="text-modal-container">
      <div className="text-modal-background" onClick={props.onClose}></div>
      <div className="text-modal-content">
        <p className="text-modal-text">{props.text}</p>
        <button className="text-modal-close" onClick={props.onClose}>Close</button>
      </div>
    </div>
  );
};

export default TextModal;
