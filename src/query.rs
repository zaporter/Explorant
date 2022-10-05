use crate::lcs::InstructionSequence;
use std::any::Any;
use std::cell::RefCell;
use std::error::Error;
use std::rc::Rc;

use druid::im::{vector, Vector};
use druid::{Data, Widget, Lens};
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
#[derive(Clone, Data,Lens)]
pub struct QueryGraphState {
    pub leaves: Vector<QueryGraphNode>,
    pub graph: VisualGraphData,
}
impl QueryGraphState {
    pub fn new(leaves: Vector<QueryGraphNode>)->Self{
        let empty_vg = VisualGraphData::new(VisualGraph::new(Orientation::LeftToRight));
        let mut me = Self{
            leaves,
            graph:empty_vg
        };
        me.refresh_vgd();
        me
    }
    pub fn refresh_vgd(&mut self){
        let mut vg = VisualGraphData::new(VisualGraph::new(Orientation::LeftToRight));
        for leaf in self.leaves.iter() {
            leaf.borrow().add_self_to_visualgraph(&mut vg, None);
        }
        self.graph = vg;

    }
}
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
    fn has_child(&self, child: &dyn QueryNode) -> bool;
    fn get_id(&self)->usize;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}
pub mod node {

    use druid::widget::{Label, Flex, Button, ViewSwitcher};
    use druid_graphviz_layout::{
        core::{base::Orientation, style::StyleAttr},
        std_shapes::shapes::{Element, ShapeKind},
    };
    use druid_widget_nursery::DropdownSelect;

    use super::*;

    pub struct TimeRange {
        pub start: usize,
        pub end: usize,
        pub selected_child_id: Option<usize>,
        pub child: Option<QueryGraphNode>,
        pub id:usize,
    }
    impl QueryNode for TimeRange {
        fn get_type(&self) -> QueryNodeType {
            QueryNodeType::Filter
        }

        fn get_display_name(&self) -> String{
            "Time Range".to_string()
        }
        fn run(
            &mut self,
            mut config: QueryConfig,
            query_executor: Box<dyn Fn(QueryConfig) -> Box<dyn QueryResult>>,
        ) -> Result<Box<dyn QueryResult>, Box<dyn Error>> {
            config.frame_time_range.0 = self.start;
            config.frame_time_range.1 = self.end;
            self.child.as_ref().expect("No child for component to run").borrow_mut().run(config, query_executor)
        }

        fn add_self_to_visualgraph(
            &self,
            state: &mut VisualGraphData,
            parent_handle: Option<NodeHandle>,
        ) {
            let sp0 = ShapeKind::new_box("two");
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
                
                child.borrow().add_self_to_visualgraph(state, Some(self_handle));
            }
        }

    fn create_sideview_elem(&self) -> Box<dyn Widget<(QueryGraphState, QueryGraphNode)>>{
            let button =
                Button::new("Increment").on_click(|_ctx, data: &mut (QueryGraphState, QueryGraphNode), _env| {
                    println!("Hi!");
                    {
                        let mut item_index = None;
                        for (i,leaf) in data.0.leaves.iter().enumerate() {
                            if leaf.borrow().get_id() == 0 {
                                item_index = Some(i);
                                break;
                            }
                        }
                        
                        let new_child =data.0.leaves.remove(item_index.expect("Unable to find element to make my child"));
                        let mut me_map= data.1.borrow_mut();
                        let mut me = me_map.as_any_mut().downcast_mut::<TimeRange>().expect("Unable to downcast myself!");
                        me.child = Some(new_child);
                    }
                    data.0.refresh_vgd();
                });
            Box::new(Flex::column()
                .with_child(Label::new("TimeRange!"))
                .with_child(button)
                .with_child(ViewSwitcher::new(
                    |d: &(QueryGraphState, QueryGraphNode), _env: &_| d.1.borrow().get_id(),
                    |selector, (shared, item): &(QueryGraphState, QueryGraphNode), _env| {
                        let elems = vec![("0",0),("1",1)];
                        Box::new(DropdownSelect::new(elems).lens(QueryGraphNode))
                    },
                )))
        }

        fn has_child(&self, child: &dyn QueryNode) -> bool {
            todo!()
        }
        fn get_id(&self) ->usize {
            self.id
        }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    }
}
