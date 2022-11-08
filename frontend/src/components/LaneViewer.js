import { Timeline } from 'react-svg-timeline'
import useRemoteResource from '../util.js';

const LaneViewer = (props) => {
  let generalInfo = props.generalInfo;
  //let [recordedFrames,_setFrames] = useRemoteResource(null,{},'recorded_frames');
  const lanes = [];
  const events = [];
  for (const trace of generalInfo.traces){

    let frameTimeMap = trace.frame_time_map;
    let times = frameTimeMap.times;
    //console.log(recordedFrames)
    
    let max_frame_time = Math.max(...(Object.keys(times).map(Number)));
    let min_frame_time = Math.min(...(Object.keys(times).map(Number)));
    let max_clock_time = times[max_frame_time];
    let min_clock_time = times[min_frame_time];
    const laneId = 'execution-lane-'+trace.id;
    lanes.push({laneId : laneId, label: `Program execution ${trace.id}`})
    events.push({
      eventId: `execution-${trace.id}`,
      laneId,
      startTimeMillis: 1,
      endTimeMillis: 1+(max_clock_time-min_clock_time),
    })
  } 
  const dateFormat = (ms) => ms && ms;
  
  return (
    <div>
      <Timeline width={600} height={300} events={events} lanes={lanes} dateFormat={dateFormat} />
    </div>
  )
}

export default LaneViewer;
