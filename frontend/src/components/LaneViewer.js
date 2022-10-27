import { Timeline } from 'react-svg-timeline'
import useRemoteResource from '../util.js';

const LaneViewer = (props) => {
  let [recordedFrames,_setFrames] = useRemoteResource(null,{},'recorded_frames');
  let frameTimeMap = props.generalInfo.frame_time_map;
  let times = frameTimeMap.times;
  console.log(recordedFrames)
  
  let max_frame_time = Math.max(...(Object.keys(times).map(Number)));
  let min_frame_time = Math.min(...(Object.keys(times).map(Number)));
  let max_clock_time = times[max_frame_time];
  let min_clock_time = times[min_frame_time];
  const laneId = 'execution-lane'
  const lanes = [
    {
      laneId,
      label: 'Program Execution',
    },
  ]
  const events = [
    {
      eventId: 'execution-1',
      tooltip: 'Execution',
      laneId,
      startTimeMillis: min_clock_time,
      endTimeMillis: max_clock_time,
    },
  ]
  const dateFormat = (ms) => ms && new Date(Math.ceil(ms)).toISOString();
  
  return (
    <div>
      <Timeline width={600} height={300} events={events} lanes={lanes} dateFormat={dateFormat} />
    </div>
  )
}

export default LaneViewer;
