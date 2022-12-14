use std::collections::HashMap;
use std::path::PathBuf;

use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::{
    erebor::Erebor,
    graph_builder::GraphBuilder,
    shared_structs::{GraphModule, GraphNode, LineLocation},
};
use std::fs::File;
use std::io::{self, BufRead, BufReader};

pub fn parse_annotations(erebor: &Erebor, graph_builder: &mut GraphBuilder) -> anyhow::Result<()> {
    let mut modules: HashMap<String, GraphModule> = HashMap::new();
    let mut nodes: HashMap<usize, GraphNode> = HashMap::new();
    // we are not guaranteed to read the files
    // in any particular order so the FQNs of the nodes
    // will actually be their module::name and the
    // FQN will be resolved later.
    modules.insert(
        "".into(),
        GraphModule {
            name: "".into(),
            parent: None,
            module_attributes: HashMap::new(),
        },
    );

    let mut fake_addr = 0;
    for (file_name, file_info) in &erebor.files {
        log::info!("reading file {} ", file_name.to_string_lossy());
        let file = File::open(file_name);
        let Ok(file) = file else {
            log::warn!("Skipping reading file {} due to an error", file_name.to_string_lossy());
            continue;
        };
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
                    fake_addr += 1;
                    log::info!("Registered event {}, ", name);

                    let mut t_name = name.clone();
                    let mut mn_iter = t_name.split("::");
                    let m_name = mn_iter.next().ok_or(anyhow::anyhow!("Event {name} does not have a module"))?;
                    let n_name = mn_iter.next().ok_or(anyhow::anyhow!("Event {name} does not have a node"))?;
                    nodes.insert(
                        fake_addr, // WILL BE REPLACED BY graph_builder
                        GraphNode {
                            FQN: "".into(), // WILL BE REPLACED BY graph_builder
                            address: 0, // WILL BE REPLACED BY graph_builder
                            module: m_name.into(),
                            name: n_name.into(),
                            node_type: "Event".into(),
                            location: LineLocation {
                                file: file_name.clone(),
                                line_num: 1+line_num as u32, // WILL BE REPLACED BY graph_builder
                                column_num: 0,
                            },
                            labeled_transitions: Vec::new(),
                            node_attributes: HashMap::new(),
                        },
                    );
                }
                Annotation::Flow { name } => {
                    fake_addr += 1;
                    log::info!("Registered event {}, ", name);

                    let mut t_name = name.clone();
                    let mut mn_iter = t_name.split("::");
                    let m_name = mn_iter.next().ok_or(anyhow::anyhow!("Flow {name} does not have a module"))?;
                    let n_name = mn_iter.next().ok_or(anyhow::anyhow!("Flow {name} does not have a node"))?;
                    nodes.insert(
                        fake_addr, // WILL BE REPLACED BY graph_builder
                        GraphNode {
                            FQN: "".into(), // WILL BE REPLACED BY graph_builder
                            address: 0, // WILL BE REPLACED BY graph_builder
                            module: m_name.into(),
                            name: n_name.into(),
                            node_type: "Flow".into(),
                            location: LineLocation {
                                file: file_name.clone(),
                                line_num: line_num as u32, // WILL BE REPLACED BY graph_builder
                                column_num: 0,
                            },
                            labeled_transitions: Vec::new(),
                            node_attributes: HashMap::new(),
                        },
                    );
                }
            }
        }
    }
    // for mut node in &mut nodes.values_mut() {
    //     node.FQN = name_to_fqn(&node.FQN, &modules)?;
    // }
    graph_builder.update_raw_modules(modules);
    graph_builder.update_raw_nodes(nodes, &erebor);
    Ok(())
}
// TODO: prevent infinite recursion with a module having itself
// as its parent
pub fn name_to_fqn(name: &str, modules: &HashMap<String, GraphModule>) -> anyhow::Result<String> {
    let mut ret: String = name.to_string();
    loop {
        let curr_module_str = ret.split("::").next().ok_or(anyhow::anyhow!("{} is an invalid event name. Ensure that it has a module specifier with module::event_name (module can be empty (::event))",name))?;
        let curr_module = modules.get(curr_module_str);
        let Some(curr_module) = curr_module else {
            return Err(anyhow::anyhow!("Invalid module name {}", curr_module_str));
        };
        if curr_module.parent.is_some() {
            ret.insert_str(0, "::");
            ret.insert_str(0, &curr_module.parent.as_ref().unwrap());
        } else {
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
    fn fqn_discovery() {
        let mut modules: HashMap<String, GraphModule> = HashMap::new();
        modules.insert(
            "".into(),
            GraphModule {
                name: "".into(),
                parent: None,
                module_attributes: HashMap::new(),
            },
        );
        modules.insert(
            "animal".into(),
            GraphModule {
                name: "animal".into(),
                parent: Some("".into()),
                module_attributes: HashMap::new(),
            },
        );
        modules.insert(
            "dog".into(),
            GraphModule {
                name: "dog".into(),
                parent: Some("animal".into()),
                module_attributes: HashMap::new(),
            },
        );
        assert!(name_to_fqn("error", &modules).is_err());
        assert_eq!(name_to_fqn("::error", &modules).unwrap(), "::error");
        assert_eq!(
            name_to_fqn("animal::error", &modules).unwrap(),
            "::animal::error"
        );
        assert_eq!(
            name_to_fqn("dog::error", &modules).unwrap(),
            "::animal::dog::error"
        );
    }
}
