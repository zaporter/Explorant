import React, {useEffect, useState} from 'react';
import TextModal from './components/TextModal.js';

export const Tutorial= (props) => {
  const [showHelp, setShowHelp] = useState(false);
  return (
    <div>
      <h3 style={{margin:"20px", alt:"te"}} onClick={()=>{setShowHelp(true)}}>
      {"ℹ️"}
      </h3>
      {showHelp && <TextModal onClose={()=>{setShowHelp(false)}}> 
        {props.children}
        </TextModal>}
    </div>
  );
};

export const SrcReaderHelp = () => {
  return (
      <div>
      <h2>The Source Reader</h2>
      The source reader displays any source file related to the project you are working on. When a graph-node is selected, the source viewer goes to that line and highlights the line which contains the address which this event is instrumenting.
      
      <h3>Interaction Options</h3>
      <ul>
        <li><b>Shift+scroll:</b> Scroll up and down in the file</li>
        <li><b>Right click a src line:</b> Open a context menu to add a new event or module definition</li>
        <li><b>Enter a new file in the file input:</b> Change what file is currently being selected/edited</li>
        </ul>
    </div>);
};
export const GraphViewerHelp = () => {
  return (
      <div>
      <h2>The Graph Viewer</h2>
        The graph viewer shows the produced graph from executing the trace. Each node corresponds to an event that was executed. However, each event may appear multiple times because <a href="https://github.com/ModelInference/synoptic"> synoptic </a> mines the FSM to simplify analysis and seperate paths that happen to go through the same events though they are different logical paths. 
        The graph viewer also displays modules and then sections of strictly sequential nodes. For more information on how this works, see the paper.
      
      <h3>Interaction Options</h3>
      <ul>
        <li><b>Left click on a node:</b> Select that node for further examination. You can then edit the event or go to where it was run in the trace.</li>
        <li><b>Click on the name of a module:</b> Collapse/expand that module</li>
        <li><b>Pan/zoom:</b> Change what file is currently being selected/edited</li>
        </ul>
    </div>);
};
export const NodeEditorHelp = () => {
  return (
      <div>
      <h2>The Node Editor</h2>
      This component allows you to edit properties of a node such as what line it is instrumenting, what module it is part of, and what it is named. 

      The difference between and Event and a Flow is entirely cosmetic and is simply useful to indicate whether you expect a node branch vs whether that node is simply a step along a pipeline.

      Also note that if bind this event to a line where there is no available instructions to instrument, it will select the next line where there is an address to watch.
      
    </div>);
};
export const ExecutionExplorerHelp = () => {
  return (
      <div>
      <h2>The Execution Explorer</h2>

        The execution explorer allows you to open up the trace in gdb to any point in time where the node was executed. This can be useful for diving into exactly what happened during this particular execution of the node. 

      <h3>Interaction Options</h3>
      <ul>
        <li><b>Click on a list element:</b> Use rr to open up a gdb server at the exact spot in the program that you clicked.</li>
        <li><b>Hover over a list element:</b> Highlight this particular instance of the event on the timeline.</li>
        </ul>
        
        <h3>Keep in mind:</h3>
        <b>This component does not currently de-allocate the gdb servers so if you click on many of them, you will quickly eat through your computing resources. This is being worked on. </b>
        <b>The gdb command may not work instantly as the trace is being fast forwarded to desired location. This will become especially apparent when the spot in the execution is very far through the program. This is being worked on. </b>
      
    </div>);
};
