use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use druid::lens;
use druid::widget::prelude::*;
use druid::widget::Button;
use druid::widget::Either;
use druid::widget::List;
use druid::widget::Scroll;
use druid::widget::SizedBox;
use druid::widget::Split;
use druid::widget::ViewSwitcher;
use druid::widget::{Align, Flex, Label, TextBox};
use druid::Color;
use druid::UnitPoint;
use druid::Vec2;
use druid::{AppLauncher, Data, Env, Lens, LocalizedString, Widget, WidgetExt, WindowDesc};
use druid::{KeyOrValue, Point, Rect, WidgetPod};
use druid_graphviz_layout::backends::druid::GraphvizWidget;
use druid_graphviz_layout::backends::druid::VisualGraphData;
use druid_graphviz_layout::core::base::Orientation;
use druid_graphviz_layout::core::style::StyleAttr;
use druid_graphviz_layout::std_shapes::shapes::Element;
use druid_graphviz_layout::std_shapes::shapes::ShapeKind;
use druid_graphviz_layout::topo::layout::VisualGraph;
use rust_lapper::Lapper;

use crate::block::Block;
use crate::block::CodeFlow;
use crate::query::node;
use crate::query::QueryGraphNode;
use crate::query::QueryGraphState;
use crate::query::QueryNode;
use crate::query::node::SelectedChild;
use druid::LensExt;

use druid::im::{vector, Vector};
// use crate::code_flow_graph::*;
// use crate::graph_layout::GraphLayout;

#[derive(Clone, Data, Lens)]
struct AppState {
    text: String,
    graph: QueryGraphState,
}

pub fn start_query_editor() {
    let main_window = WindowDesc::new(build_root_widget())
        .title("Streeling University Library Terminal")
        .window_size((400.0, 400.0));

    // let leaves : Vector<Rc<RefCell<dyn QueryNode>>>= vector![Rc::new(RefCell::new(Node::TimeRange{start:0, end:100, child:None, id:0}))];
    let k: QueryGraphNode = Rc::new(RefCell::new(node::TimeRange {
        start: 0,
        end: 100,
        child: None,
        selected_child_id:SelectedChild::None,
        id: 1,
        ver:10,
    }));
    let l: QueryGraphNode = Rc::new(RefCell::new(node::TimeRange {
        start: 0,
        end: 100,
        child: None,
        selected_child_id:SelectedChild::None,
        id: 2,
        ver:10,
    }));
    let leaves = vector![k, l];

    let initial_state = AppState {
        text: "pen".into(),
        graph: QueryGraphState::new(leaves)
    };
    AppLauncher::with_window(main_window)
        .launch(initial_state)
        .expect("Failed to launch application");
}

fn build_root_widget() -> impl Widget<AppState> {
    Split::columns(build_graph_widget(), build_side_widget()).split_point(0.75)
}

fn build_graph_widget() -> impl Widget<AppState> {
    GraphvizWidget::new().lens(AppState::graph.then(QueryGraphState::graph))
}
fn build_side_widget() -> impl Widget<AppState> {
    let button = Button::new("Add Time Range").on_click(|_ctx, data: &mut AppState, _env| {
        // data.graph = create_vgd("FUCK".into());
    });
    Flex::column()
        .with_child(button)
        .with_child(
    Scroll::new(
        List::new(|| {
            ViewSwitcher::new(
                |d: &(QueryGraphState, QueryGraphNode), _env: &_| d.1.borrow().get_id()*d.1.borrow().get_ver(),
                |selector, (shared, item): &(QueryGraphState, QueryGraphNode), _env| {
                    item.borrow().create_sideview_elem()
                },
            )
        })
        .lens(AppState::graph.map(
            |d: &QueryGraphState| (d.clone(), d.leaves.clone()),
            |d: &mut QueryGraphState,
             x: (QueryGraphState, Vector<QueryGraphNode>)| { *d = x.0; },
        )),
    )
    .border(Color::grey(0.6), 2.0)

        )
}
