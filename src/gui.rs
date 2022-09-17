
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

const WINDOW_TITLE: LocalizedString<CodeFlowState> = LocalizedString::new("Code Flow Graph Examiner");

#[derive(Clone, Data, Lens)]
struct BlockState {
    block : Arc<Block>,
}

pub fn start_code_flow_examiner(code_flow : CodeFlow) {
    let main_window = WindowDesc::new(build_root_widget)
        .title(WINDOW_TITLE)
        .window_size((400.0, 400.0));


    let initial_state = CodeFlowState {
        graph_layout: Arc::new(GraphLayout::new(code_flow)),
        center: Vec2::default(),
        zoom: 1.,
        mouse_down_last:None,
    };

    AppLauncher::with_window(main_window)
        .launch(initial_state)
        .expect("Failed to launch application");
}


fn build_root_widget() -> CodeFlowGraph{
    let label = CodeFlowGraph::new();
   label
}

