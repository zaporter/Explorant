import React from 'react';

const LoadingModal = () => {
  return (
    <div className="loading-modal-container">
      <div className="modal-overlay" />
      <div className="modal-content">
        <div className="loading-dots">
          <span className="dot" />
          <span className="dot" />
          <span className="dot" />
        </div>
      </div>
    </div>
  );
};
export default LoadingModal;
