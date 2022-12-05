import React, { useState } from 'react';

const ExecutionInstanceList = (props) => {
  // Declare a state variable to store the currently hovered item
  const [hoveredItem, setHoveredItem] = useState(null);

  // Create a list of instances to display
  const instances = [
    { order: 1, frametime: 100 },
    { order: 2, frametime: 200 },
    { order: 3, frametime: 300 }
  ];

  // Function to handle hover events on list items
  const handleHover = (item) => {
    // Update the state variable with the hovered item
    setHoveredItem(item);

    // Pass the hovered item to the parent component through the props
    props.onHover(item);
  }

  // Function to handle click events on list items
  const handleClick = (item) => {
    // Print "Clicked!" to the console
    console.log('Clicked!');

    // Pass the clicked item to the parent component through the props
    props.onClick(item);
  }

  return (
    <div className="box-wrapper">
      <h3>{"Execution Instances:"}</h3>
      

   <table className="execution-instance-list">
      <thead>
        <tr>
          <th className="execution-instance-list__header">Order</th>
          <th className="execution-instance-list__header">Frametime</th>
        </tr>
      </thead>
      <tbody>
        {
          // Map over the instances and create a table row for each instance
          instances.map((instance) => (
            <tr
              className="execution-instance-list__row"
              // Set the hover and click handlers for each row
              onMouseEnter={() => handleHover(instance)}
              onMouseLeave={() => handleHover(null)}
              onClick={() => handleClick(instance)}
            >
              <td className="execution-instance-list__cell">{instance.order}</td>
              <td className="execution-instance-list__cell">{instance.frametime}</td>
            </tr>
          ))
        }
      </tbody>
    </table>
      </div>
  );
}

export default ExecutionInstanceList;
