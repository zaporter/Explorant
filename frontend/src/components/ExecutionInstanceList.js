import React, { useEffect, useState } from 'react';
import { useRemoteResource } from '../util.js';
import { callRemote } from '../util.js';
import TextModal from './TextModal.js';
import { Tutorial, ExecutionExplorerHelp } from '../tutorials.js';
import { Timeline } from 'react-svg-timeline'

const ExecutionInstanceList = (props) => {
  // Declare a state variable to store the currently hovered item
  const [hoveredItem, setHoveredItem] = useState({ frame_time: 0 });

  const [modalText, setModalText] = useState(null);
  // let setExecutionInstances = props.setExecutionInstances;
  // let nodesData = props.nodesData;
  let currentNodeId = props.currentNodeId.id;
  //let currentNode = nodesData.nodes[currentNodeId];

  const [instances, _set] = useRemoteResource({ val: [] },
    { "synoptic_node_id": currentNodeId },
    'addr_occurrences', [currentNodeId])

  // Function to handle hover events on list items
  const handleHover = (item) => {
    // Update the state variable with the hovered item
    setHoveredItem(item);

  }

  let generalInfo = props.generalInfo;
  //let [recordedFrames,_setFrames] = useRemoteResource(null,{},'recorded_frames');
  const [lanes, setLanes] = useState([]);
  const [events, setEvents] = useState([]);
  useEffect(() => {
    let i_lanes = [];
    let i_events = [];
    for (const trace of generalInfo.traces) {

      let frameTimeMap = trace.frame_time_map;
      let times = frameTimeMap.times;
      //console.log(recordedFrames)

      let max_frame_time = Math.max(...(Object.keys(times).map(Number)));
      let min_frame_time = Math.min(...(Object.keys(times).map(Number)));
      let max_clock_time = times[max_frame_time];
      let min_clock_time = times[min_frame_time];
      const laneId = 'execution-lane-' + trace.id;
      i_lanes.push({ laneId: laneId, label: `Program execution` })
      i_events.push({
        eventId: `execution-${trace.id}`,
        laneId,
        startTimeMillis: min_clock_time,
        endTimeMillis: max_clock_time,
      })
    }

    i_lanes.push({ laneId: 'i', label: `Event instances` })
    const hoveredColor = "#ea8080";
    let usedfts = [];
    let ftmap = generalInfo.traces[0].frame_time_map;
    for (const instance of instances.val) {
      if (usedfts.includes(instance.frame_time)) {
        continue;
      }
      usedfts.push(instance.frame_time);

      if (hoveredItem != null && ftmap.times[instance.frame_time] == ftmap.times[hoveredItem.frame_time]) {
        continue;
      } else {
        i_events.push({
          eventId: `e-${instance.frame_time}`,
          laneId: 'i',
          startTimeMillis: ftmap.times[instance.frame_time],
        })
      }

    }
    if (hoveredItem != null) {
      i_events.push({
        eventId: `e-${hoveredItem.frame_time}`,
        laneId: 'i',
        color: hoveredColor,
        startTimeMillis: ftmap.times[hoveredItem.frame_time],
      })
    }
    setLanes(i_lanes);
    setEvents(i_events);
  }, [hoveredItem, instances])
  const dateFormat = (ms) => new Date(ms).toJSON();
  // Function to handle click events on list items
  const handleClick = (item) => {
    callRemote({ "start_time": item }, "create_gdb_server")
      .then(response => response.json())
      .then(response => setModalText(response.value));
  }

  return (
    <div className="box-wrapper">
      <div className="tutorial-div">
        <h3>{"Execution Explorer"}</h3>
        <Tutorial><ExecutionExplorerHelp /></Tutorial>
      </div>

      {modalText && <TextModal onClose={() => setModalText(null)}>
        <div>
          <p className='text-modal-text'>{"To go to this location in the trace, execute:"}</p>
          <p className='text-modal-text'>{modalText}</p>

        </div>
      </TextModal>}
      <br />

      <div className='execution-instance-list-split'>
        <div className='lane-viewer'>
          {lanes.length != 0 &&
            <Timeline className='lane-viewer-timeline' width={600} height={300} events={events} lanes={lanes} dateFormat={dateFormat} />}
        </div>
        <div className='execution-instance-list-split-child'>
          <p>{"Click one of the instances below to start a gdb server at that location:"}</p>

          <div className="execution-instance-list-scroll">
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
                      key={instance.frame_time * 1000000000 + instance.instance_of_addr}
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
        </div>
      </div>
    </div>
  );
}

export default ExecutionInstanceList;
