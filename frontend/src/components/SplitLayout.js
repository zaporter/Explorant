import React, { useState } from 'react';

const SplitLayout = ({ children }) => {
  const [barPosition, setBarPosition] = useState(50);

  const handleDragStart = (event) => {
    event.preventDefault();
    event.stopPropagation();

    const handleDrag = (event) => {
      const newBarPosition = event.clientX / window.innerWidth * 100;
      setBarPosition(newBarPosition);
    };

    const handleDragEnd = (event) => {
      document.removeEventListener('mousemove', handleDrag);
      document.removeEventListener('mouseup', handleDragEnd);
    };

    document.addEventListener('mousemove', handleDrag);
    document.addEventListener('mouseup', handleDragEnd);
  };

  return (
<div className="split-layout">
      <div className="split-layout-left" style={{ width: `${barPosition}%` }}>
        {children[0]}
      </div>
      <div
        className="split-layout-bar"
        style={{ width: `${1}px`, marginLeft: `${-1}px` }}
        onMouseDown={handleDragStart}
      >
        &nbsp;
      </div>
      <div className="split-layout-right" style={{ width: `${100 - barPosition}%` }}>
        {children[1]}
      </div>
    </div>
  );
};

export default SplitLayout;
