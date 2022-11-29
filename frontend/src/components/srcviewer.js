import { Timeline } from 'react-svg-timeline'
import {useRemoteResource} from '../util.js';
import SyntaxHighlighter from 'react-syntax-highlighter';
import { atomOneDark } from 'react-syntax-highlighter/dist/esm/styles/hljs';
const SrcViewer = (props) => {
  let currentFile = props.currentFile;
  const [data, setData] = useRemoteResource("", {file_name: currentFile}, "source_file", [currentFile]);
  const codeString = '(num) => num + 1';
  return (
  <div className="src-viewer">
    <h3>{"Source Viewer"}</h3>
    <p>{`Viewing ${currentFile}`}</p>
    {/* <p key={currentFile }>{data.data}</p> */}
    <div className="src-inner">
      <SyntaxHighlighter language="c-like" style={atomOneDark} showLineNumbers={true} children="1">
        {data.data}
      </SyntaxHighlighter>
    </div>
    </div>
  );
}

export default SrcViewer;
