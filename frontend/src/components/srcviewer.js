import { Timeline } from 'react-svg-timeline'
import {useState, useEffect, useRef} from 'react'
import {useRemoteResource} from '../util.js';
import {Prism as SyntaxHighlighter } from 'react-syntax-highlighter';
//import { atomOneDark } from 'react-syntax-highlighter/dist/esm/styles/hljs';
import { a11yDark } from "react-syntax-highlighter/dist/cjs/styles/prism";
import StringCompletionInput from './StringCompletionInput.js';
import TextModal from './TextModal.js';
import NodeEditor from './NodeEditor.js'
import ModuleEditor from './ModuleEditor.js'
import {Tutorial, SrcReaderHelp} from '../tutorials.js';
const SrcViewer = (props) => {
  let nodesData = props.nodesData;
  
  // This had better be an even number
  const numLines = 30;

  let currentFile = props.currentFilePath ? {file: props.currentFilePath,line_num:props.currentFileLineNum} : {file:"[none selected]", line_num:0};
  const [centeredLine, setCenteredLine] = useState(props.currentFileLineNum-(numLines/2));
  useEffect(()=>{setCenteredLine(props.currentFileLineNum-(numLines/2))},[props.currentFileLineNum]);


  const [allFiles, _setAllFiles] = useRemoteResource({files:["[none selected]"]},{}, 'source_files');

  const onUpdate = (new_val) => {
    props.setCurrentFileLineNum(0);
    props.setCurrentFilePath(new_val)
  };
  const [data, _setData] = useRemoteResource({data:""}, {file_name: currentFile.file}, "source_file", [props.currentFilePath]);
  let lines = data.data.split("\n");
  
  let minLine = Math.max(0,centeredLine);
  let maxLine = Math.min(lines.length-1, centeredLine+numLines);
  let usedLines = lines.slice(minLine,maxLine);
  while (usedLines.length < numLines) {
    usedLines.push("");
  }
  
  let toDisplay = usedLines.join("\n");
  let numLinesInFile = lines.length;

  const handleScroll = (event) => {
    if (event.shiftKey) {
      event.preventDefault();
      let amount = Math.floor(event.deltaY / 25);
      let newLineNum = centeredLine+amount;
      setCenteredLine(Math.min(numLinesInFile-numLines,Math.max(0,newLineNum)));
    }
  };
  const dropdownRef = useRef();
  const [showDropdown, setShowDropdown] = useState(false);
  const [x, setX] = useState(0);
  const [y, setY] = useState(0);
  const [clickedLineNum, setClickedLineNum] = useState(0);
  const [showAddNodeModal, setShowAddNodeModal] = useState(false);
  const [showUpdateModuleModal, setShowUpdateModuleModal] = useState(false);

  const handleRightClick = (e) => {
    e.preventDefault();
    setShowDropdown(true);
    setX(e.pageX);
    setY(e.pageY);
  }
   const addEvent = () => {
     setShowAddNodeModal(true);
     setShowDropdown(false);
  }

  const addModule = () => {
     setShowUpdateModuleModal(true);
     setShowDropdown(false);
  }
  const handleClick = (e) => {
    if (dropdownRef.current && !dropdownRef.current.contains(e.target)) {
      setShowDropdown(false);
    }
  }
  return (
  <div className="box-wrapper" onClick={handleClick}>
  {showDropdown && (
        <div ref={dropdownRef} className="src-viewer-dropdown-menu" style={{ left: x, top: y }}>
          <div className="src-viewer-dropdown-option" onClick={addEvent}>
            ➕ Add Event
          </div>
          <div className="src-viewer-dropdown-option" onClick={addModule}>
            ➕ Add / Update Module 
          </div>
        </div>
      )}

  <div className="src-viewer">
    <div className="tutorial-div">
      <h3>{"Source Viewer"}</h3>
      <Tutorial><SrcReaderHelp/></Tutorial>
    </div>
    <div style={{padding:"0px 5px 0px 0px"}}>
    <StringCompletionInput 
      key={props.currentFilePath}
      default={props.currentFilePath}
      onUpdate={onUpdate}
      list={allFiles.files.concat(["[none selected]"])}
      />
      </div>
    <div className="src-inner" 
      onWheel={handleScroll}
      onScroll={(e)=>{e.preventDefault()}}  
    >
    <div ref={dropdownRef}>
      <SyntaxHighlighter 
        customStyle={{overflow:"hidden"}}
        language="clike" 
        style={a11yDark} 
        startingLineNumber={minLine+1}
        showLineNumbers={true} 
        wrapLines={true}
        lineProps={(lineNumber) => {
          const style = { display: "block", width: "fit-content" };
          if (currentFile.line_num == lineNumber) {
            style.backgroundColor = "#ca0a0a";
          }
          //style.onClick = () => {console.log("test")};
          let onClick = () => {console.log("test")}
          let onContextMenu = (e) => {
            setClickedLineNum(lineNumber);
            handleRightClick(e);
          };
          return { style, onContextMenu};
        }}
        
      >
        {toDisplay}
      </SyntaxHighlighter>
      </div>
    </div>
    </div>

    {showAddNodeModal && <TextModal onClose={()=>{setShowAddNodeModal(false)}}>
        <NodeEditor 
          mode={"add"}
          name="name"
          currentNodeId={{id:0,is_raw:false}}
          line={clickedLineNum}
          file={currentFile.file}
          updateNodeData={props.updateNodeData}
          onClose={()=>{setShowAddNodeModal(false)}}
          nodesData={props.nodesData}/>
      </TextModal>}
    {showUpdateModuleModal && <TextModal onClose={()=>{setShowUpdateModuleModal(false)}}>
      <ModuleEditor
        nodesData={props.nodesData}
        updateNodeData={props.updateNodeData}
        onClose={()=>{setShowUpdateModuleModal(false)}}
      />

      </TextModal>}
    </div>
  );
}

export default SrcViewer;
