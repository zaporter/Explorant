import { Timeline } from 'react-svg-timeline'
import {useRemoteResource} from '../util.js';
import SyntaxHighlighter from 'react-syntax-highlighter';
//import { atomOneDark } from 'react-syntax-highlighter/dist/esm/styles/hljs';
import { a11yDark } from "react-syntax-highlighter/dist/cjs/styles/prism";
const SrcViewer = (props) => {
  let currentFile = props.currentFile;
  const [data, setData] = useRemoteResource({data:""}, {file_name: currentFile.file}, "source_file", [currentFile]);
  let lines = data.data.split("\n");
  
  let minLine = Math.max(0,currentFile.line-30);
  let maxLine = Math.min(lines.length-1, currentFile.line+30);
  let usedLines = lines.slice(minLine,maxLine);
  
  let toDisplay = usedLines.join("\n");
  return (
  <div className="src-viewer">
    <h3>{"Source Viewer"}</h3>
    <p>{`Viewing ${currentFile.file}`}</p>
    {/* <p key={currentFile }>{data.data}</p> */}
    <div className="src-inner">
      <SyntaxHighlighter 
        language="c-like" 
        style={a11yDark} 
        startingLineNumber={minLine}
        showLineNumbers={true} 
        wrapLines={true}
        lineProps={(lineNumber) => {
          console.log(lineNumber);
          const style = { display: "block", width: "fit-content" };
          console.log(lineNumber);
          if (currentFile.line == lineNumber+1) {
            style.backgroundColor = "#ca0a0a";
          }
          return { style };
        }}
      >
        {toDisplay}
      </SyntaxHighlighter>
    </div>
    </div>
  );
}

export default SrcViewer;
