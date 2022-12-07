import React, { useState } from 'react';
import { useRemoteResource } from '../util.js';
import { callRemote } from '../util.js';

const ExecutionInstanceList = (props) => {
  // Declare a state variable to store the currently hovered item
  const [hoveredItem, setHoveredItem] = useState(null);

  let setExecutionInstances = props.setExecutionInstances;
  let nodesData = props.nodesData;
  let currentNodeId = props.currentNodeId;
  //let currentNode = nodesData.nodes[currentNodeId];

  const [instances, _set] = useRemoteResource({val:[]},
    { "synoptic_node_id": currentNodeId },
    'addr_occurrences', [currentNodeId])

  // Function to handle hover events on list items
  const handleHover = (item) => {
    // Update the state variable with the hovered item
    setHoveredItem(item);

  }
  const [message, setMessage] = useState("Click on one of the executions to start a gdb server at that location");

  // Function to handle click events on list items
  const handleClick = (item) => {
    callRemote({ "start_time": item }, "create_gdb_server")
      .then(response => response.json())
      .then(response => setMessage(response.value));
  }

  return (
    <div className="box-wrapper">
      <h3>{"Execution Instances:"}</h3>
      <p>{message}</p>


      <table className="execution-instance-list">
        <thead>
          <tr>
            <th className="execution-instance-list__header">Frametime</th>
            <th className="execution-instance-list__header">Addr instance in frame</th>
          </tr>
        </thead>
        <tbody>
          {
            // Map over the instances and create a table row for each instance
            instances.val.map((instance) => (
              <tr
                className="execution-instance-list__row"
                key={instance.frame_time*1000000000+instance.instance_of_addr}
                // Set the hover and click handlers for each row
                onMouseEnter={() => handleHover(instance)}
                onMouseLeave={() => handleHover(null)}
                onClick={() => handleClick(instance)}
              >
                <td className="execution-instance-list__cell">{instance.frame_time}</td>
                <td className="execution-instance-list__cell">{instance.instance_of_addr}</td>
              </tr>
            ))
          }
        </tbody>
      </table>
    </div>
  );
}

export default ExecutionInstanceList;
