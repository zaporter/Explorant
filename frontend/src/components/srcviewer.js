import { Timeline } from 'react-svg-timeline'
import {useState, useEffect, useRef} from 'react'
import {useRemoteResource} from '../util.js';
import {Prism as SyntaxHighlighter } from 'react-syntax-highlighter';
//import { atomOneDark } from 'react-syntax-highlighter/dist/esm/styles/hljs';
import { a11yDark } from "react-syntax-highlighter/dist/cjs/styles/prism";
import StringCompletionInput from './StringCompletionInput.js';
import TextModal from './TextModal.js';
import NodeEditor from './NodeEditor.js'
import {Tutorial, SrcReaderHelp} from '../tutorials.js';
const SrcViewer = (props) => {
  let nodesData = props.nodesData;
  
  // This had better be an even number
  const numLines = 40;

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
      console.log(event.deltaY);
      let amount = Math.floor(event.deltaY / 25);
      let newLineNum = centeredLine+amount;
      console.log(numLinesInFile);
      setCenteredLine(Math.min(numLinesInFile-numLines,Math.max(0,newLineNum)));
    }
  };
  const dropdownRef = useRef();
  const [showDropdown, setShowDropdown] = useState(false);
  const [x, setX] = useState(0);
  const [y, setY] = useState(0);
  const [clickedLineNum, setClickedLineNum] = useState(0);
  const [showAddNodeModal, setShowAddNodeModal] = useState(false);

  const handleRightClick = (e) => {
    e.preventDefault();
    setShowDropdown(true);
    setX(e.pageX);
    setY(e.pageY);
  }
   const addEvent = () => {
     setShowAddNodeModal(true);
     setShowDropdown(false);
    // Add Event code here
  }

  const addModule = () => {
    // Add Module Definition code here
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
            ➕ Add Module Definition
          </div>
        </div>
      )}

  <div className="src-viewer">
    <div className="tutorial-div">
      <h3>{"Source Viewer"}</h3>
      <Tutorial><SrcReaderHelp/></Tutorial>
    </div>
    
    <StringCompletionInput 
      key={props.currentFilePath}
      default={props.currentFilePath}
      onUpdate={onUpdate}
      list={allFiles.files.concat(["[none selected]"])}
      />
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
          line={clickedLineNum}
          file={currentFile.file}
          updateNodeData={props.updateNodeData}
          nodesData={props.nodesData}/>
      </TextModal>}
    </div>
  );
}

export default SrcViewer;
