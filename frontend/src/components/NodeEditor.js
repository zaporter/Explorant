const NodeEditor = (props) => {
  var node_name = props.node_name;
  return (
    <div className="box-wrapper">
    <div className="node-editor">
      <h3>{"Node Editor"}</h3>
      <p>{"Current node: "+node_name}</p>
      </div>
      </div>
  )
}
export default NodeEditor;
