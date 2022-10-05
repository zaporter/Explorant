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
}
impl QueryGraphState {
    pub fn new(leaves: Vector<QueryGraphNode>) -> Self {
        let empty_vg = VisualGraphData::new(VisualGraph::new(Orientation::LeftToRight));
        let mut me = Self {
            leaves,
            graph: empty_vg,
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
pub trait QueryNode {
    fn get_type(&self) -> QueryNodeType;
    fn get_display_name(&self) -> String;
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
    fn create_sideview_elem(&self) -> Box<dyn Widget<(QueryGraphState, QueryGraphNode)>>;
    fn contains_node(&self, check_id:usize)->bool;
    fn get_id(&self) -> usize;
    fn get_ver(&self) -> usize;
    fn inc_ver(&mut self);
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub mod node {

    #[derive(Clone, Data, PartialEq, PartialOrd, Ord, Eq, Debug)]
    pub enum SelectedChild {
        Some(usize),
        None,
    }


    use druid::{
        lens,
        widget::{Button, Flex, Label, ViewSwitcher, Either},
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
        pub selected_child_id: SelectedChild,
        pub child: Option<QueryGraphNode>,
        pub id: usize,
        pub ver: usize,
    }
    impl QueryNode for TimeRange {
        fn get_type(&self) -> QueryNodeType {
            QueryNodeType::Filter
        }

        fn get_display_name(&self) -> String {
            "Time Range".to_string()
        }
        fn run(
            &mut self,
            mut config: QueryConfig,
            query_executor: Box<dyn Fn(QueryConfig) -> Box<dyn QueryResult>>,
        ) -> Result<Box<dyn QueryResult>, Box<dyn Error>> {
            config.frame_time_range.0 = self.start;
            config.frame_time_range.1 = self.end;
            self.child
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
            let sp0 = ShapeKind::new_box(&format!("id: {}", self.get_id()));
            let look0 = StyleAttr::simple();
            let sz = druid_graphviz_layout::core::geometry::Point::new(100., 100.);
            let node0 = Element::create(sp0, look0, Orientation::LeftToRight, sz);

            let self_handle = state.graph.borrow_mut().add_node(node0);

            if let Some(parent_handle) = parent_handle {
                let arrow = druid_graphviz_layout::std_shapes::shapes::Arrow::simple("123");
                state
                    .graph
                    .borrow_mut()
                    .add_edge(arrow, parent_handle, self_handle);
            }
            if let Some(child) = &self.child {
                child
                    .borrow()
                    .add_self_to_visualgraph(state, Some(self_handle));
            }
        }

        fn create_sideview_elem(&self) -> Box<dyn Widget<(QueryGraphState, QueryGraphNode)>> {
            Box::new(
                Flex::column()
                    .with_child(Label::new(
                        |(_, item): &(QueryGraphState, TimeRange), _env: &_| {
                            format!("{} :#{}",item.get_display_name(), item.get_id())
                        },
                    ))
                    // .with_child(button)
                    .with_child(Label::new("Child:"))
                    .with_child(ViewSwitcher::new(
                        |d: &(QueryGraphState, TimeRange), _env: &_| d.1.get_id() * d.1.get_ver(),
                        |selector, (shared, item): &(QueryGraphState, TimeRange), _env| {
                            let mut elems = vec![("None".to_owned(), SelectedChild::None)];

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
                            if let Some(child) = &item.child {
                                let child_id = child.borrow().get_id();
                                elems.push((
                                    format!("#{}", child_id),
                                    SelectedChild::Some(child_id),
                                ));
                            }
                            Box::new(ListSelect::new(elems).lens(lens::Field::new(
                                |(_, item): &(QueryGraphState, TimeRange)| &item.selected_child_id,
                                |(_, item)| &mut item.selected_child_id,
                            )))
                        },
                    ))
                    .with_child(ViewSwitcher::new(|d:&(QueryGraphState,TimeRange),_env:&_|d.1.child.is_some(),
                    |selector, (shared_outer,item_outer): &(QueryGraphState, TimeRange), _env| {
                        if item_outer.child.is_some() {
                            Box::new(
                                ViewSwitcher::new(
                                    |d: &(QueryGraphState, QueryGraphNode), _env: &_| d.1.borrow().get_id() * d.1.borrow().get_ver(),
                                    |selector, (shared, item): &(QueryGraphState, QueryGraphNode), _env| {
                                            item.borrow().create_sideview_elem()
                                    },
                                ).lens(lens::Identity.map(
                                    |d: &(QueryGraphState, TimeRange)| {(d.0.clone(), d.1.child.clone().unwrap())},
                                    |d: &mut (QueryGraphState, TimeRange), x: (QueryGraphState, QueryGraphNode)| {
                                        d.0 = x.0;
                                        
                                    }
                                    ))
                                )
                        }else {
                            Box::new(Label::new("No Child"))

                        }

                    }))
                    .border(Color::grey(0.6),2.0)
                    .rounded(5.0)
                    .lens(lens::Identity.map(
                        |d: &(QueryGraphState, QueryGraphNode)| {
                            (
                                d.0.clone(),
                                d.1.borrow_mut()
                                    .as_any()
                                    .downcast_ref::<TimeRange>()
                                    .expect("WFT")
                                    .clone(),
                            )
                        },
                        |d: &mut (QueryGraphState, QueryGraphNode),
                         mut x: (QueryGraphState, TimeRange)| {
                            d.0 = x.0;

                            if let SelectedChild::Some(child_id) = x.1.selected_child_id {
                                if x.1.child.is_none() {
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
                                    new_child.borrow_mut().inc_ver();
                                    x.1.child = Some(new_child);
                                    x.1.inc_ver();
                                }
                            }
                            if let SelectedChild::None = x.1.selected_child_id {
                                if x.1.child.is_some() {
                                    let child = x.1.child.unwrap();
                                    child.borrow_mut().inc_ver();
                                    d.0.leaves.push_front(child.clone());
                                    x.1.child = None;
                                    x.1.inc_ver();
                                }
                            }
                            {
                                let mut me = d.1.borrow_mut();
                                let real: &mut TimeRange =
                                    me.as_any_mut().downcast_mut::<TimeRange>().unwrap();
                                *real = x.1;
                            }
                            d.0.refresh_vgd();
                        },
                    ))
            )
        }


        fn contains_node(&self, check_id:usize)->bool{

            if let Some(child) = &self.child {
                if child.borrow().get_id() == check_id {
                    return true;
                }else {
                    return child.borrow().contains_node(check_id);
                }
            }else {return false;}
        }
        fn get_id(&self) -> usize {
            self.id
        }
        fn get_ver(&self) -> usize {
            self.ver
        }
        fn inc_ver(&mut self) {
            self.ver += 1;
        }
        fn as_any(&self) -> &dyn Any {
            self
        }
        fn as_any_mut(&mut self) -> &mut dyn Any {
            self
        }
    }
}
