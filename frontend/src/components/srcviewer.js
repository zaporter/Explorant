import { Timeline } from 'react-svg-timeline'
import {useState, useEffect, useRef} from 'react'
import {useRemoteResource} from '../util.js';
import {Prism as SyntaxHighlighter } from 'react-syntax-highlighter';
//import { atomOneDark } from 'react-syntax-highlighter/dist/esm/styles/hljs';
import { a11yDark } from "react-syntax-highlighter/dist/cjs/styles/prism";
import StringCompletionInput from './StringCompletionInput.js';
const SrcViewer = (props) => {
  let nodesData = props.nodesData;
  

  let currentFile = props.currentFilePath ? {file: props.currentFilePath,line_num:props.currentFileLineNum} : {file:"[none selected]", line_num:0};
  const [centeredLine, setCenteredLine] = useState(props.currentFileLineNum);
  useEffect(()=>{setCenteredLine(props.currentFileLineNum)},[props.currentFileLineNum]);


  let valid_files = ["/home/zack/test","/home/zack/chicken"];
  let [cf,setCf] = useState("/home/zack/test");
  const onUpdate = (new_val) => {setCf(new_val)};
  const [data, _setData] = useRemoteResource({data:""}, {file_name: currentFile.file}, "source_file", [props.currentFilePath]);
  let lines = data.data.split("\n");
  
  let minLine = Math.max(0,centeredLine-20);
  let maxLine = Math.min(lines.length-1, centeredLine+20);
  let usedLines = lines.slice(minLine,maxLine);
  
  let toDisplay = usedLines.join("\n");

  const handleScroll = (event) => {
    if (event.shiftKey) {
      event.preventDefault();
      console.log(event.deltaY);
      let amount = Math.floor(event.deltaY / 30);
      setCenteredLine(centeredLine+amount);
    }
  };
  const dropdownRef = useRef();
  const [showDropdown, setShowDropdown] = useState(false);
  const [x, setX] = useState(0);
  const [y, setY] = useState(0);
  const [clickedLine, setClickedLine] = useState(null);
  const handleRightClick = (e) => {
    e.preventDefault();
    setShowDropdown(true);
    setX(e.pageX);
    setY(e.pageY);
  }
   const addEvent = () => {
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
    <h3>{"Source Viewer"}</h3>
    <p>{`Viewing ${cf}`}</p>
    <StringCompletionInput 
      default={cf}
      onUpdate={onUpdate}
      list={valid_files}
      />
    <div className="src-inner" 
      onWheel={handleScroll}
      onScroll={(e)=>{e.preventDefault()}}  
    >
    <div ref={dropdownRef}>
      <SyntaxHighlighter 
        language="clike" 
        style={a11yDark} 
        startingLineNumber={minLine}
        showLineNumbers={true} 
        wrapLines={true}
        lineProps={(lineNumber) => {
          console.log(lineNumber);
          const style = { display: "block", width: "fit-content" };
          console.log(lineNumber);
          if (currentFile.line_num == lineNumber+1) {
            style.backgroundColor = "#ca0a0a";
          }
          //style.onClick = () => {console.log("test")};
          let onClick = () => {console.log("test")}
          let onContextMenu = (e) => {
            setClickedLine(lineNumber);
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
    </div>
  );
}

export default SrcViewer;
