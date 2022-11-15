use std::collections::HashMap;
use std::path::PathBuf;

use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::{
    erebor::Erebor,
    shared_structs::{GraphModule, GraphNode, LineLocation},
};
use std::fs::File;
use std::io::{self, BufRead, BufReader};

pub struct FileNodeParsingResult {}

pub fn parse_annotations(erebor: &mut Erebor) -> anyhow::Result<()> {
    let mut modules: HashMap<String, GraphModule> = HashMap::new();
    let mut nodes: Vec<GraphNode> = Vec::new();
    // we are not guaranteed to read the files
    // in any particular order so the FQNs of the nodes
    // will actually be their module::name and the
    // FQN will be resolved later.
    modules.insert("".into(),GraphModule {
        name: "".into(),
        parent:None, 
        module_attributes: HashMap::new(),
    });
    for (file_name, file_info) in &erebor.files {
        let file = File::open(file_name)?;
        let reader = BufReader::new(file);

        'line: for (line_num, line) in reader.lines().enumerate() {
            let anno = parse_line(&line?)?;
            let Some(anno) = anno else {
                continue 'line;
            };
            match anno {
                Annotation::Module {
                    name,
                    parent_module,
                    comment,
                } => {
                    if modules
                        .insert(
                            name.clone(),
                            GraphModule {
                                name: name.clone(),
                                // point to the root node rather than nothing
                                parent: Some(parent_module.unwrap_or("".into())),
                                module_attributes: HashMap::new(),
                            },
                        )
                        .is_some()
                    {
                        return Err(anyhow::anyhow!("Duplicate module entry ({})", name));
                    }
                }
                Annotation::Event { name } => {
                    let mut event_addr = None;
                    'addr_search: for offset in 0..1000 {
                        let addrs = file_info.lines.get(&((line_num+offset) as u32));
                        if let Some(addrs) = addrs {
                            if addrs.len() > 0 {
                                event_addr = Some(addrs.get(0).unwrap());
                                break 'addr_search;
                            }
                        }
                    }
                    let Some(event_addr) = event_addr else {
                        return Err(anyhow::anyhow!("Unable to find an address for the {} event annotation", name));
                    };
                    nodes.push(GraphNode {
                        FQN: name,
                        address: 0,
                        node_type: "event".into(),
                        location: LineLocation { file: file_name.clone(), line_num: line_num as u32, column_num: 0 },
                        labeled_transisitons: Vec::new(),
                        node_attributes: HashMap::new(),
                    });
                }
                Annotation::Flow { name } => {}
            }
        }
    }
       

    Ok(())
}
// TODO: prevent infinite recursion with a module having itself 
// as its parent
fn name_to_fqn(name : &str, modules: &HashMap<String,GraphModule>)->anyhow::Result<String>{
    let mut ret : String = name.to_string();
    loop {
        let curr_module_str = ret.split("::").next().ok_or(anyhow::anyhow!("{} is an invalid event name. Ensure that it has a module specifier with module::event_name (module can be empty (::event))",name))?;
        let curr_module = modules.get(curr_module_str);
        let Some(curr_module) = curr_module else {
            return Err(anyhow::anyhow!("Invalid module name {}", curr_module_str));
        };
        if curr_module.parent.is_some() {
            ret.insert_str(0, "::");
            ret.insert_str(0, &curr_module.parent.as_ref().unwrap());
        }else {
            break;
        }
    }
    Ok(ret)
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(tag = "type")]
enum Annotation {
    #[serde(rename = "module")]
    Module {
        name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        parent_module: Option<String>,
        comment: Option<String>,
    },
    #[serde(rename = "event")]
    // {type:"flow", name:"module::self"}
    Event { name: String },
    #[serde(rename = "flow")]
    // {type:"flow", name:"module::self"}
    Flow { name: String },
}

fn parse_line(line: &str) -> anyhow::Result<Option<Annotation>> {
    let re = Regex::new(r"\[\[\{(.*)\}\]\]")?;
    let caps = re.captures(line);
    let Some(cap) = caps else {
        return Ok(None);
    };
    if cap.len() != 2 {
        return Err(anyhow::anyhow!(
            "Invalid ({}) number of annotations on a single line. ",
            cap.len() - 1
        ));
    }
    let anno = json5::from_str(&format!("{{{}}}", &cap[1]))?;
    Ok(anno)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn empty_deserialize() {
        let line = "";
        assert_eq!(parse_line(line).unwrap(), None);
    }
    #[test]
    fn code_deserialize() {
        let line = "random content here k [[512]]";
        assert_eq!(parse_line(line).unwrap(), None);
    }
    #[test]
    fn module_deserialize() {
        let line = r#"[[{type:"module", name:"pizza", comment:"delicious food"}]]"#;
        let eq = Annotation::Module {
            name: "pizza".into(),
            parent_module: None,
            comment: Some("delicious food".into()),
        };
        assert_eq!(parse_line(line).unwrap(), Some(eq));
    }
    #[test]
    fn event_deserialize() {
        let line = r#"[[{type:"event", name:"parent::pizza"}]]"#;
        let eq = Annotation::Event {
            name: "parent::pizza".into(),
        };
        assert_eq!(parse_line(line).unwrap(), Some(eq));
    }
    #[test]
    fn flow_deserialize() {
        let line = r#"[[{type:"flow", name:"::pizza"}]]"#;
        let eq = Annotation::Flow {
            name: "::pizza".into(),
        };
        assert_eq!(parse_line(line).unwrap(), Some(eq));
    }
    #[test]
    fn fqn_discovery(){
        
        let mut modules: HashMap<String, GraphModule> = HashMap::new();
        modules.insert("".into(),GraphModule {
            name: "".into(),
            parent:None, 
            module_attributes: HashMap::new(),
        });
        modules.insert("animal".into(),GraphModule {
            name: "animal".into(),
            parent:Some("".into()), 
            module_attributes: HashMap::new(),
        });
        modules.insert("dog".into(),GraphModule {
            name: "dog".into(),
            parent:Some("animal".into()), 
            module_attributes: HashMap::new(),
        });
        assert!(name_to_fqn("error", &modules).is_err());
        assert_eq!(name_to_fqn("::error", &modules).unwrap(), "::error");
        assert_eq!(name_to_fqn("animal::error", &modules).unwrap(), "::animal::error");
        assert_eq!(name_to_fqn("dog::error", &modules).unwrap(), "::animal::dog::error");
    }
}
