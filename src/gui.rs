
use std::sync::Arc;

use druid::Vec2;
use druid::widget::prelude::*;
use druid::widget::SizedBox;
use druid::widget::{Align, Flex, Label, TextBox};
use druid::{AppLauncher, Data, Env, Lens, LocalizedString, Widget, WidgetExt, WindowDesc};
use druid::{KeyOrValue, Point, Rect, WidgetPod};
use rust_lapper::Lapper;

use crate::block::Block;
use crate::block::CodeFlow;
use crate::code_flow_graph::*;
use crate::graph_layout::GraphLayout;
const VERTICAL_WIDGET_SPACING: f64 = 20.0;
const TEXT_BOX_WIDTH: f64 = 200.0;

const WINDOW_TITLE: LocalizedString<CodeFlowState> = LocalizedString::new("Code Flow Graph Examiner");

#[derive(Clone, Data, Lens)]
struct BlockState {
    block : Arc<Block>,
}
struct BlockWidget {
     title : Label<BlockState>,
}

pub fn start_code_flow_examiner(code_flow : CodeFlow) {
    // describe the main window
    let main_window = WindowDesc::new(build_root_widget)
        .title(WINDOW_TITLE)
        .window_size((400.0, 400.0));

    // create the initial app state

    let initial_state = CodeFlowState {
        graph_layout: Arc::new(GraphLayout::new(code_flow)),
        center: Vec2::default(),
        zoom: 1.,
        mouse_down_last:None,
    };

    // start the application
    AppLauncher::with_window(main_window)
        .launch(initial_state)
        .expect("Failed to launch application");
}


fn build_root_widget() -> CodeFlowGraph{//impl Widget<CodeFlowState> {
    // a label that will determine its text based on the current app data.
    //let label = Label::new(|data: &CodeFlowState, _env: &Env| format!("Hello !!"));

    // let block_widget = BlockWidget::new();
    // a textbox that modifies `name`.
    // let textbox = TextBox::new()
    //     .with_placeholder("Who are we greeting?")
    //     .fix_width(TEXT_BOX_WIDTH)
    //     .lens(CodeFlowState::name);

    // arrange the two widgets vertically, with some padding
    // let layout = Flex::column()
    //     .with_child(label)
    //     .with_spacer(VERTICAL_WIDGET_SPACING)
    //     .with_child(textbox);
    // let layout = CodeFlowGraph::new()
    //     .with_node(label, GraphNodeParams {});
        // .with_node(textbox, GraphNodeParams {});
    // center the two widgets in the available space
    let label = CodeFlowGraph::new();
   label
    // Align::centered(layout)
}

