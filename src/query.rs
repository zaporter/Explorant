use crate::lcs::InstructionSequence;
use std::any::Any;
use std::cell::RefCell;
use std::error::Error;
use std::rc::Rc;

use druid::im::{vector, Vector};
use druid::{Data, Lens, Widget, WidgetExt};
use druid_graphviz_layout::adt::dag::NodeHandle;
use druid_graphviz_layout::backends::druid::VisualGraphData;
use druid_graphviz_layout::core::base::Orientation;
use druid_graphviz_layout::topo::layout::VisualGraph;

pub enum Fidelity {
    // Dynamic code segment instrumentation with singlesteps
    DynamicEveryInstruction, // Very slow
    DynamicHighFidelity,     // 2000-4000x original time
    // Use static code segment instrumentation
    StaticHighFidelity, // 5-10x original time
    StaticLowFidelity,  // 1-2x original time
}
impl Fidelity {
    pub fn get_higher<'a>(&'a self, other: &'a Fidelity) -> &Fidelity {
        if self.to_int() > other.to_int() {
            return self;
        } else {
            return other;
        }
    }
    fn to_int(&self) -> usize {
        match self {
            Self::StaticLowFidelity => 0,
            Self::StaticHighFidelity => 1,
            Self::DynamicHighFidelity => 9,
            Self::DynamicEveryInstruction => 10,
        }
    }
}

// pub enum QueryNode{
//     TimeRange(usize,usize),
//     FidelityFilter(Rc<QueryNode>, Fidelity),
//     // MemoryRange(usize,usize),
//     // Union(Rc<Selection>, Rc<Selection>),
//     // Intersection(Rc<Selection>, Rc<Selection>),
//     // Exclude(Rc<Selection>, Rc<Selection>),
//     RunCountFilter(Option<usize>,Option<usize>),
//     // SectionFilter(Vec<String>), // memory mapped sections to monitor
//     // ThreadFilter(Vec<String>),
//     Query(Box<dyn Query>),
// }

pub struct QueryConfig {
    fidelity: Fidelity,
    frame_time_range: (usize, usize),
}
pub enum QueryNodeType {
    Filter,
    Config,
    InstructionComparator,
    JumpQuery,
}
pub enum QueryResultType {
    InstructionSequence,
}
pub trait QueryResult {
    fn get_type(&self) -> QueryResultType;
}
pub type QueryGraphNode = Rc<RefCell<dyn QueryNode>>;

#[derive(Clone, Data, Lens)]
pub struct QueryGraphState {
    pub leaves: Vector<QueryGraphNode>,
    pub graph: VisualGraphData,
    pub last_node_id : usize,
    pub ver : usize,
}
impl QueryGraphState {
    pub fn new(leaves: Vector<QueryGraphNode>) -> Self {
        let empty_vg = VisualGraphData::new(VisualGraph::new(Orientation::LeftToRight));
        let mut me = Self {
            leaves,
            graph: empty_vg,
            last_node_id:0,
            ver:0,
        };
        me.refresh_vgd();
        me
    }
    pub fn refresh_vgd(&mut self) {
        let mut vg = VisualGraphData::new(VisualGraph::new(Orientation::LeftToRight));
        for leaf in self.leaves.iter() {
            leaf.borrow().add_self_to_visualgraph(&mut vg, None);
        }
        self.graph = vg;
    }
}
// pub struct NodeParameter{

// }
// pub struct QueryNode {
//     children : Vector<usize>
// }
pub trait BasicNodeMetadata {
    fn get_type(&self) -> QueryNodeType;
    fn display_name(&self) -> String;
    fn num_children(&self) -> usize;
    fn basic(&self)-> &BasicNodeData;
    fn basic_mut(&mut self)->&mut BasicNodeData;
    fn run(
        &mut self,
        config: QueryConfig,
        query_executor: Box<dyn Fn(QueryConfig) -> Box<dyn QueryResult>>,
    ) -> Result<Box<dyn QueryResult>, Box<dyn Error>>;
    fn add_self_to_visualgraph(
        &self,
        state: &mut VisualGraphData,
        parent_handle: Option<NodeHandle>,
    );
}

#[derive(Clone, Data, PartialEq, PartialOrd, Ord, Eq, Debug)]
pub enum SelectedChild {
    Some(usize),
    None,
}
#[derive(Clone, Data, Default)]
pub struct BasicNodeData {
    pub selected_child_ids: Vector<SelectedChild>,
    pub children: Vector<Option<QueryGraphNode>>,
    pub id: usize,
}
pub trait BasicNodeFunctionality : BasicNodeMetadata{
    fn contains_node(&self, check_id:usize)->bool;
    fn get_id(&self) -> usize;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}
pub trait QueryNode : BasicNodeMetadata + BasicNodeFunctionality{
    fn create_sideview_elem(&self) -> Box<dyn Widget<(QueryGraphState, QueryGraphNode)>>;
}

pub mod node {

    use druid::{
        lens,
        widget::{Button, Flex, Label, ViewSwitcher, Either, List},
        LensExt, Color,
    };
    use druid_graphviz_layout::{
        core::{base::Orientation, style::StyleAttr},
        std_shapes::shapes::{Element, ShapeKind},
    };
    use druid_widget_nursery::{DropdownSelect, ListSelect};
    use nix::libc::printf;
    use serde::{Deserialize, Serialize};

    use super::*;


    #[derive(Clone, Data, Lens)]
    pub struct TimeRange {
        pub start: usize,
        pub end: usize,
        basic : BasicNodeData,
    }
    impl TimeRange {
        pub fn new(id:usize)->Self{
            let mut me =Self{
                start:0,
                end:0,
                basic: BasicNodeData::default(),
            };
            me.basic.id = id;
            for _ in 0..me.num_children() {
                me.basic.children.push_front(None);
                me.basic.selected_child_ids.push_front(SelectedChild::None);
            }
            me
        }

    }
    impl BasicNodeMetadata for TimeRange {
        fn get_type(&self) -> QueryNodeType {
            QueryNodeType::Filter
        }

        fn display_name(&self) -> String {
            "Time Range".to_string()
        }

        fn num_children(&self) -> usize{2}
        fn basic(&self)-> &BasicNodeData{&self.basic}
        fn basic_mut(&mut self)-> &mut BasicNodeData{&mut self.basic}
        
        fn run(
            &mut self,
            mut config: QueryConfig,
            query_executor: Box<dyn Fn(QueryConfig) -> Box<dyn QueryResult>>,
        ) -> Result<Box<dyn QueryResult>, Box<dyn Error>> {
            config.frame_time_range.0 = self.start;
            config.frame_time_range.1 = self.end;
            self.basic().children.get(0).unwrap()
                .as_ref()
                .expect("No child for component to run")
                .borrow_mut()
                .run(config, query_executor)
        }

        fn add_self_to_visualgraph(
            &self,
            state: &mut VisualGraphData,
            parent_handle: Option<NodeHandle>,
        ) {
            let shape = ShapeKind::new_box(&format!("id: {}", self.get_id()));
            let look = StyleAttr::simple();
            let sz = druid_graphviz_layout::core::geometry::Point::new(100., 100.);
            let node = Element::create(shape, look, Orientation::LeftToRight, sz);

            let self_handle = state.graph.borrow_mut().add_node(node);

            if let Some(parent_handle) = parent_handle {
                let arrow = druid_graphviz_layout::std_shapes::shapes::Arrow::simple("");
                state
                    .graph
                    .borrow_mut()
                    .add_edge(arrow, parent_handle, self_handle);
            }
            if let Some(child) = &self.basic().children[0] {
                child
                    .borrow()
                    .add_self_to_visualgraph(state, Some(self_handle));
            }
            if let Some(child) = &self.basic().children[1] {
                child
                    .borrow()
                    .add_self_to_visualgraph(state, Some(self_handle));
            }
        }

    }
    impl<T> BasicNodeFunctionality for T 
    where 
        T: BasicNodeMetadata + 'static
    {
        fn contains_node(&self, check_id:usize)->bool{
            for child in self.basic().children.iter() {
                if let Some(child) = &child {
                    if child.borrow().get_id() == check_id || child.borrow().contains_node(check_id){
                        return true;
                    }
                }
            }
            false
        }
        fn get_id(&self) -> usize {
            self.basic().id
        }
        fn as_any(&self) -> &dyn Any {
            self
        }
        fn as_any_mut(&mut self) -> &mut dyn Any {
            self
        }
    }
    impl<T> QueryNode for T 
    where 
        T: BasicNodeFunctionality+ BasicNodeMetadata + Data+ 'static
    {

        fn create_sideview_elem(&self) -> Box<dyn Widget<(QueryGraphState, QueryGraphNode)>> {
            Box::new(
                Flex::column()
                    .with_child(Label::new(
                        |(_, item): &(QueryGraphState, T), _env: &_| {
                            format!("{} :#{}",item.display_name(), item.get_id())
                        },
                    ))
                    // .with_child(button)
                    .with_child(List::new(||{
                        Flex::column()
                        .with_child(Label::new("Child:"))

                    .with_child(ViewSwitcher::new(
                        |d: &(QueryGraphState, (T,usize)), _env: &_| d.0.ver,
                        |selector, (shared, (item,index)): &(QueryGraphState, (T,usize)), _env| {
                            let mut elems = vec![("None".to_owned(), SelectedChild::None)];
                            
                            if let Some(child) = &item.basic().children[*index] {
                                let child_id = child.borrow().get_id();
                                elems.push((
                                    format!("#{}", child_id),
                                    SelectedChild::Some(child_id),
                                ));
                            }else {
                                for k in shared.leaves.iter() {
                                    // Dont include my parent as an option
                                    if k.borrow().contains_node(item.get_id()){
                                        continue;
                                    }
                                    let other_id = k.borrow().get_id();
                                    if other_id != item.get_id() {
                                        elems.push((
                                            format!("#{}", other_id),
                                            SelectedChild::Some(other_id),
                                        ));
                                    }
                                }

                            }
                            Box::new(ListSelect::new(elems).lens(lens::Field::new(
                                |(_, (item,index)): &(QueryGraphState, (T,usize))| &item.basic().selected_child_ids[*index],
                                |(_, (item,index))| &mut item.basic_mut().selected_child_ids[*index],
                            )))
                        },
                    ))

                    .with_child(ViewSwitcher::new(|d:&(QueryGraphState,(T,usize)),_env:&_|d.1.0.basic().children[d.1.1].is_some(),
                    |selector, (shared_outer,(item_outer,child_index)): &(QueryGraphState, (T,usize)), _env| {
                        if item_outer.basic().children[*child_index].is_some() {
                            Box::new(
                                ViewSwitcher::new(
                                    |d: &(QueryGraphState, QueryGraphNode), _env: &_| d.0.ver,
                                    |selector, (shared, item): &(QueryGraphState, QueryGraphNode), _env| {
                                            item.borrow().create_sideview_elem()
                                    },
                                ).lens(lens::Identity.map(
                                    |d: &(QueryGraphState, (T,usize))| {(d.0.clone(), d.1.0.basic().children[d.1.1].clone().unwrap())},
                                    |d: &mut (QueryGraphState, (T,usize)), x: (QueryGraphState, QueryGraphNode)| {
                                        d.0 = x.0;
                                        
                                    }
                                    ))
                                )
                        }else {
                            Box::new(Label::new("No Child"))
                        }

                    }))

                    })
                        .lens(lens::Identity.map(

                                |d: &(QueryGraphState, T)| {
                                    let mut res = Vector::new();
                                    for i in 0..d.1.num_children() {
                                        res.push_back((d.1.clone(), i));
                                    }
                                    (d.0.clone(), res)
                                }, 
                                |d: &mut (QueryGraphState, T), x:(QueryGraphState,Vector<(T, usize)>)| {

                                    d.0 = x.0;
                                    for i in 0..d.1.num_children() {
                                        d.1.basic_mut().children[i] = x.1[i].0.basic().children[i].clone();
                                        d.1.basic_mut().selected_child_ids[i] = x.1[i].0.basic().selected_child_ids[i].clone();
                                    }

                                })) 
                    )
                    .border(Color::grey(0.6),2.0)
                    .rounded(5.0)
                    .lens(lens::Identity.map(
                        |d: &(QueryGraphState, QueryGraphNode)| {
                            (
                                d.0.clone(),
                                d.1.borrow_mut()
                                    .as_any()
                                    .downcast_ref::<T>()
                                    .expect("WFT")
                                    .clone(),
                            )
                        },
                        |d: &mut (QueryGraphState, QueryGraphNode),
                         mut x: (QueryGraphState, T)| {
                            d.0 = x.0;
                        for child_index in 0..d.1.borrow().num_children(){
                            if let SelectedChild::Some(child_id) = x.1.basic().selected_child_ids[child_index] {
                                if x.1.basic().children[child_index].is_none() {
                                    let mut item_index = None;
                                    for (i, leaf) in d.0.leaves.iter().enumerate() {
                                        if leaf.borrow().get_id() == child_id {
                                            item_index = Some(i);
                                            break;
                                        }
                                    }

                                    let new_child = d.0.leaves.remove(
                                        item_index
                                            .expect("Unable to find element to make my child"),
                                    );
                                    x.1.basic_mut().children[child_index] = Some(new_child);
                                    d.0.ver+=1;
                                }
                            }
                            if let SelectedChild::None = x.1.basic().selected_child_ids[child_index] {
                                if x.1.basic().children[child_index].is_some() {
                                    let child = x.1.basic_mut().children[child_index].as_mut().unwrap();
                                    d.0.leaves.push_back(child.clone());
                                    x.1.basic_mut().children[child_index] = None;
                                    d.0.ver+=1;
                                }
                            }
                         }
                            {
                                let mut me = d.1.borrow_mut();
                                let real: &mut T =
                                    me.as_any_mut().downcast_mut::<T>().unwrap();
                                *real = x.1;
                            }
                            d.0.refresh_vgd();
                        },
                    ))
            )
        }


    }
}
