use std::{cell::RefCell, collections::HashMap, fmt::Display, rc::Rc, sync::Arc};

use druid::{
    kurbo::{Circle, Shape, Line},
    widget::Label,
    BoxConstraints, Color, Data, Env, Event, EventCtx, LayoutCtx, Lens, LifeCycle, LifeCycleCtx,
    PaintCtx, Point, RenderContext, Size, UpdateCtx, Widget, WidgetPod, Vec2, MouseButton, piet::TextLayoutBuilder,
};


use crate::{
    block::{Block, CodeFlow},
    graph_layout::GraphLayout,
};

// type Wrapper<T> = Rc<RefCell<Node<T>>>;

// fn wrap<T: Display>(data: T) -> Wrapper<T> {
//     Rc::new(RefCell::new(Node::new(data)))
// }

// #[derive(Debug)]
// pub struct Node<T> {
//     data: T,
//     children: Vec<Wrapper<T>>
// }

// impl<T: Display> Node<T> {
//     pub fn add_child(&mut self, child: Wrapper<T>) {
//         self.children.push(child);
//     }

//     pub fn new(data: T) -> Node<T> {
//         Node { data, children: Vec::new() }
//     }

//     fn depth_first(&self) {
//         println!("node {}", self.data);
//         for child in self.children.iter() {
//             child.borrow().depth_first();
//         }
//     }
// }

#[derive(Clone, Data, Lens)]
pub struct CodeFlowState {
    pub graph_layout: Arc<GraphLayout>,
    pub center: Vec2,
    
    pub zoom: f64,
    pub mouse_down_last: Option<Vec2>
}
#[derive(Clone, Data, Lens)]
struct BlockState {
    block: Arc<Block>,
}
#[derive(Copy, Clone, Default)]
pub struct GraphNodeParams {}
pub struct CodeFlowGraph {
    nodes: HashMap<usize, BlockWidget>,
}
struct BlockWidget {
    title: Label<BlockState>,
}

impl BlockWidget {
    pub fn new() -> BlockWidget {
        let title = Label::new(|data: &BlockState, _env: &Env| format!("H=ello !"));
        BlockWidget { title }
    }
}
impl CodeFlowGraph {
    pub fn new() -> Self {
        //     let nodes = HashMap::new();
        //     for n in *state.block_states {
        //         nodes.insert(n.block.base().clone(), BlockWidget::new());
        //     }
        CodeFlowGraph {
            nodes: HashMap::new(),
        }
    }
}
impl Widget<CodeFlowState> for CodeFlowGraph {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut CodeFlowState, env: &Env) {
    let current_size= ctx.size();
    match event{
        Event::Wheel(e)=>{
            if e.wheel_delta.y >0.0{
                data.zoom/=1.1;
            }else {
                data.zoom*=1.1;
            }
            ctx.request_paint();
        },
        Event::MouseDown(e)=> {
            data.mouse_down_last=Some(Vec2{x:e.pos.x, y:e.pos.y});
        },
        Event::MouseUp(_)=>{
            data.mouse_down_last=None;  
        },
        Event::MouseMove(e)=>{
            let pos = Vec2{x:e.pos.x, y:e.pos.y};
            if e.buttons.has_right(){ 
                if let Some(last)= data.mouse_down_last{
                    let mut delta = pos-last;
                    delta.x/= current_size.width;
                    delta.y/=current_size.height;
                    delta/=data.zoom;
                    data.center+=delta;
                    data.mouse_down_last = Some(pos);
                    ctx.request_paint();
                }
            }
        },
        _=>{},
    }
    }
    fn lifecycle(
        &mut self,
        ctx: &mut LifeCycleCtx,
        event: &LifeCycle,
        data: &CodeFlowState,
        env: &Env,
    ) {
    }
    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &CodeFlowState,
        env: &Env,
    ) -> Size {
        let max_size = bc.max();
        max_size
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &CodeFlowState, env: &Env) {
        let size: Size = ctx.size();
        let width = size.width;
        let height = size.height;
        for (link,val) in data.graph_layout.links.iter(){
            let p0 = data.graph_layout.nodes.get(&link.0).unwrap();
            let p1 = data.graph_layout.nodes.get(&link.1).unwrap();
            let origin = Point::new(
                (p0.position.x+data.center.x) *width*data.zoom, 
                (p0.position.y+data.center.y) *height*data.zoom);
            let dest = Point::new(
                (p1.position.x+data.center.x) *width*data.zoom, 
                (p1.position.y+data.center.y) *height*data.zoom);
            ctx.stroke(Line::new(origin,dest), &Color::BLUE, 1.*val.raw_link.attributes.count as f64);

        }
        for node in data.graph_layout.nodes.values() {
            let node_pos = Point::new(
                (node.position.x+data.center.x) *width*data.zoom, 
                (node.position.y+data.center.y) *height*data.zoom);

            ctx.fill(Circle::new(node_pos, 50.0), &Color::RED);
        }
    }
    fn update(
        &mut self,
        ctx: &mut UpdateCtx,
        _old_data: &CodeFlowState,
        data: &CodeFlowState,
        env: &Env,
    ) {
    }
}

// impl Widget<BlockState> for BlockWidget{
//     fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut BlockState, env: &Env) {
//         self.title.event(ctx, event, data, env);

//     }
//     fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &BlockState, env: &Env) {
//         self.title.lifecycle(ctx, event, data, env);
//     }
//     fn update(&mut self, ctx: &mut UpdateCtx, old_data: &BlockState, data: &BlockState, env: &Env) {

//         self.title.update(ctx, old_data, data, env);
//     }
//     fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &BlockState, env: &Env) -> Size {
//         self.title.layout(ctx, bc, data, env)

//     }
//     fn paint(&mut self, ctx: &mut PaintCtx, data: &BlockState, env: &Env) {
//         self.title.paint(ctx, data, env);

//     }

// }
