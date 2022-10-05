use std::{cell::RefCell, collections::HashMap, fmt::Display, rc::Rc, sync::Arc};

use druid::{
    kurbo::{Circle, Shape, Line, RoundedRect},
    widget::Label,
    BoxConstraints, Color, Data, Env, Event, EventCtx, LayoutCtx, Lens, LifeCycle, LifeCycleCtx,
    PaintCtx, Point, RenderContext, Size, UpdateCtx, Widget, WidgetPod, Vec2, MouseButton, piet::{TextLayoutBuilder, Text, TextAttribute}, RadialGradient, LinearGradient, UnitPoint,
};

use iced_x86::{Decoder, DecoderOptions, Formatter, Instruction, NasmFormatter};
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
            let (c_x, c_y) = (data.center.x-(0.5/data.zoom), data.center.y-(0.5/data.zoom));
            
            if e.wheel_delta.y >0.0{
                data.zoom/=1.1;
            }else {
                data.zoom*=1.1;
            }
            data.center.x = c_x + (0.5/data.zoom);
            data.center.y = c_y + (0.5/data.zoom);
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
            let grad = LinearGradient::new(UnitPoint::new(p0.position.x,p0.position.y), UnitPoint::new(p1.position.x,p1.position.y),(Color::FUCHSIA, Color::AQUA));
            ctx.stroke(Line::new(origin,dest), &grad, 1.*val.raw_link.attributes.count as f64);

        }
        for node in data.graph_layout.nodes.values() {
            let mut node_pos = Point::new(
                (node.position.x+data.center.x) *width*data.zoom, 
                (node.position.y+data.center.y) *height*data.zoom);

            let width= 250.;
            let height = 14.*node.val.instructions().len()as f64;
            node_pos.x-=width/2.;
            node_pos.y-=height/2.;
            ctx.fill(RoundedRect::new(node_pos.x-7.,node_pos.y-7.,node_pos.x+width+7.,node_pos.y+height+7.,10.0), &Color::BLACK);
            ctx.fill(RoundedRect::new(node_pos.x-5.,node_pos.y-5.,node_pos.x+width+5.,node_pos.y+height+5.,10.0), &Color::WHITE);
           let mut formatter = NasmFormatter::new();

            // // Change some options, there are many more
            // formatter.options_mut().set_digit_separator("`");
            formatter.options_mut().set_first_operand_char_index(10);
            formatter.options_mut().set_leading_zeros(false);
            // formatter.options_mut().set_se_operand_char_index(10);

            // String implements FormatterOutput
            let mut output = String::new();


            for instruction in node.val.instructions(){
                output.clear();
                formatter.format(&instruction, &mut output);


                let text = ctx.text();
                let line = text.new_text_layout(output.clone())
                    .default_attribute(TextAttribute::FontSize(14.0))
                    .build().unwrap();
                
                ctx.draw_text(&line, node_pos);
                node_pos.y+=12.;
            }
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
