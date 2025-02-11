use crate::{
    makepad_platform::*,
    frame_component::*,
    fold_button::*
};

live_register!{
    FoldHeader: {{FoldHeader}} {
        walk:{
            width:Size::Fill,
            height:Size::Fit
        }
        body_walk:{
            width:Size::Fill,
            height:Size::Fit
        }
        layout:{
            flow:Flow::Down,
        }
        
        state:{
            open = {
                default: on
                off = {
                    from: {all: Play::Exp {speed1: 0.96, speed2: 0.97}}
                    redraw: true
                    apply: {
                        opened: [{time: 0.0, value: 1.0}, {time: 1.0, value: 0.0}]
                    }
                }
                on = {
                    from: {all: Play::Exp {speed1: 0.98, speed2: 0.95}}
                    redraw: true
                    apply: {
                        opened: [{time: 0.0, value: 0.0}, {time: 1.0, value: 1.0}]
                    }
                }
            }
        }
    }
}

#[derive(Live, LiveHook)]
#[live_register(frame_component!(FoldHeader))]
pub struct FoldHeader {
    #[rust] draw_state: DrawStateWrap<DrawState>,
    header: FrameComponentRef,
    body: FrameComponentRef,

    state: State,
    opened: f32,

    view: View,
    layout: Layout,
    walk: Walk,
    body_walk: Walk,
}

#[derive(Clone)]
enum DrawState{
    DrawHeader,
    DrawBody
}

impl FrameComponent for FoldHeader {
    fn handle_component_event(&mut self, cx: &mut Cx, event: &mut Event, _self_id: LiveId) -> FrameComponentActionRef {
        if self.state_handle_event(cx, event).must_redraw() {
            if self.state.is_track_animating(cx, id!(open)) {
                let rect = self.view.get_rect(cx);
                self.view.set_scroll_pos(cx, vec2(0.0,rect.size.y * (1.0-self.opened)));
                //cx.redraw_all();
                self.view.redraw(cx);
            }
        };
        let mut actions = Vec::new();
        if let Some(child) = self.header.as_mut(){
            if let Some(action) = child.handle_component_event(cx, event, id!(header)){
                for item in action.cast::<FrameActions>(){
                    if item.id == id!(fold_button){
                        match item.action.cast(){
                            FoldButtonAction::Opening=>{
                                self.animate_state(cx, ids!(open.on))
                            }
                            FoldButtonAction::Closing=>{
                                self.animate_state(cx, ids!(open.off))
                            }
                            _=>()
                        }
                    }
                }
                actions.merge(id!(header), Some(action));
            }
        }
        if let Some(child) = self.body.as_mut(){
            actions.merge(id!(body), child.handle_component_event(cx, event, id!(body)));
        }
        FrameActions::from_vec(actions).into()
    }

    fn redraw(&mut self, cx:&mut Cx){
        self.view.redraw(cx);
        self.header.redraw(cx);
        self.body.redraw(cx);
    }
    
    fn get_walk(&self) -> Walk {
        self.walk
    }
    
    fn find_child(&mut self, id: &[LiveId]) -> ChildResult {
        self.header.find_child(id)?;
        self.body.find_child(id)?;
        NoChild
    }
    
    fn create_child(&mut self, cx:&mut Cx, at:CreateAt, id:LiveId, path: &[LiveId], nodes:&[LiveNode]) -> ChildResult {
        self.header.create_child(cx, at, id, path, nodes)?;
        self.body.create_child(cx, at, id, path, nodes)?;
        NoChild
    }
    
    fn draw_component(&mut self, cx: &mut Cx2d, walk: Walk) -> Result<(), LiveId> {
        if self.draw_state.begin(cx, DrawState::DrawHeader){
            cx.begin_turtle(walk, self.layout);
            // lets draw our header
        }
        if let DrawState::DrawHeader = self.draw_state.get(){
            if let Some(child) = self.header.as_mut(){
                child.draw_walk_component(cx)?;
            }
            if self.view.begin(cx, self.body_walk, Layout::flow_down()).is_err(){
                self.reverse_walk_opened(cx);
                cx.end_turtle();
                self.draw_state.end();
                return Ok(())
            };
            self.draw_state.set(DrawState::DrawBody);
        }
        if let DrawState::DrawBody = self.draw_state.get(){
            if let Some(child) = self.body.as_mut(){
                child.draw_walk_component(cx)?;
            }
            self.view.end(cx);
            // reverse walk
            self.reverse_walk_opened(cx);
            cx.end_turtle();
            self.draw_state.end();
        }
        Ok(())
    }
}

impl FoldHeader{
    fn reverse_walk_opened(&mut self, cx:&mut Cx2d){
        let rect = self.view.get_rect(cx);
        cx.walk_turtle(Walk::size(Size::Fill, Size::Negative(rect.size.y * (1.0-self.opened))));
    }
}

#[derive(Clone, FrameComponentAction)]
pub enum FoldHeaderAction {
    Opening,
    Closing,
    None
}