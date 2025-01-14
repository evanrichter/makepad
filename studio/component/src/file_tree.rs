use {
    std::{
        collections::{HashSet},
    },
    crate::{
        makepad_platform::*,
        makepad_component::{
            scroll_shadow::ScrollShadow,
            component_map::ComponentMap,
            scroll_view::ScrollView
        },
    }
};

live_register!{
    use makepad_platform::shader::std::*;
    use makepad_component::theme::*;
    
    DrawBgQuad: {{DrawBgQuad}} {
        fn pixel(self) -> vec4 {
            return mix(
                mix(
                    COLOR_BG_EDITOR,
                    #2,
                    self.is_even
                ),
                mix(
                    COLOR_BG_UNFOCUSSED,
                    COLOR_BG_SELECTED,
                    self.focussed
                ),
                self.selected
            );
            // COLOR_BG_HOVER,
            // self.hover
            //);
        }
    }
    
    DrawNameText: {{DrawNameText}} {
        fn get_color(self) -> vec4 {
            return mix(
                mix(
                    COLOR_TEXT_DEFAULT * self.scale,
                    COLOR_TEXT_SELECTED,
                    self.selected
                ),
                COLOR_TEXT_HOVER,
                self.hover
            )
        }
        
        text_style: FONT_DATA {
            top_drop: 1.3,
        }
    }
    
    DrawIconQuad: {{DrawIconQuad}} {
        fn pixel(self) -> vec4 {
            let sdf = Sdf2d::viewport(self.pos * self.rect_size);
            let w = self.rect_size.x;
            let h = self.rect_size.y;
            sdf.box(0. * w, 0.35 * h, 0.87 * w, 0.39 * h, 0.75);
            sdf.box(0. * w, 0.28 * h, 0.5 * w, 0.3 * h, 1.);
            sdf.union();
            return sdf.fill(mix(
                mix(
                    COLOR_TEXT_DEFAULT * self.scale,
                    COLOR_TEXT_SELECTED,
                    self.selected
                ),
                COLOR_TEXT_HOVER,
                self.hover
            ));
        }
    }
    
    FileTreeNode: {{FileTreeNode}} {
        
        layout: {
            align: {y: 0.5},
            padding: {left: 5.0, bottom: 1.0,},
        }
        
        icon_walk: Walk {
            width: Size::Fixed((DIM_DATA_ICON_WIDTH)),
            height: Size::Fixed((DIM_DATA_ICON_HEIGHT)),
            margin: Margin {
                left: 1
                top: 0
                right: 4
                bottom: 0
            },
        }
        
        state : {
            hover = {
                default:off
                off = {
                    from: {all: Play::Forward {duration: 0.2}}
                    apply: {
                        hover: 0.0,
                        bg_quad: {hover: (hover)}
                        name_text: {hover: (hover)}
                        icon_quad: {hover: (hover)}
                    }
                }
                
                on = {
                    from: {all: Play::Snap}
                    apply: {hover: 1.0},
                }
            }
            
            focus = {
                default:on
                on = {
                    from: {all: Play::Snap}
                    apply: {focussed: 1.0}
                }
                
                off = {
                    from: {all: Play::Forward {duration: 0.1}}
                    apply: {focussed: 0.0}
                }
            }
            
            select = {
                default: off
                off = {
                    from: {all: Play::Forward {duration: 0.1}}
                    apply: {
                        selected: 0.0,
                        bg_quad: {selected: (selected)}
                        name_text: {selected: (selected)}
                        icon_quad: {selected: (selected)}
                    }
                }
                on = {
                    from: {all: Play::Snap}
                    apply: {selected:1.0}
                }
                
            }
            
            open = {
                default: off
                off = {
                    from: {all: Play::Exp {speed1: 0.80, speed2: 0.97}}
                    //duration: 0.2
                    redraw: true
                    //ease: Ease::OutExp
                    apply: {
                        opened: [{time: 0.0, value: 1.0}, {time: 1.0, value: 0.0}]
                        bg_quad: {opened: (opened)}
                        name_text: {opened: (opened)}
                        icon_quad: {opened: (opened)}
                    }
                }
                
                on = {
                    from: {all: Play::Exp {speed1: 0.82, speed2: 0.95}}
                    redraw: true
                    apply: {
                        opened: [{time: 0.0, value: 0.0}, {time: 1.0, value: 1.0}]
                    }
                }
            }
        }
        is_folder: false,
        indent_width: 10.0
        min_drag_distance: 10.0
    }
    
    FileTree: {{FileTree}} {
        node_height: (DIM_DATA_ITEM_HEIGHT),
        file_node: FileTreeNode {
            is_folder: false,
            bg_quad: {is_folder: 0.0}
            name_text: {is_folder: 0.0}
        }
        folder_node: FileTreeNode {
            is_folder: true,
            bg_quad: {is_folder: 1.0}
            name_text: {is_folder: 1.0}
        }
        layout: {flow: Flow::Down},
        scroll_view: {
            view: {
                debug_id: file_tree_view
            }
        }
    }
}

// TODO support a shared 'inputs' struct on drawshaders
#[derive(Live, LiveHook)]#[repr(C)]
struct DrawBgQuad {
    draw_super: DrawQuad,
    is_even: f32,
    scale: f32,
    is_folder: f32,
    focussed: f32,
    selected: f32,
    hover: f32,
    opened: f32,
}

#[derive(Live, LiveHook)]#[repr(C)]
struct DrawNameText {
    draw_super: DrawText,
    is_even: f32,
    scale: f32,
    is_folder: f32,
    focussed: f32,
    selected: f32,
    hover: f32,
    opened: f32,
}

#[derive(Live, LiveHook)]#[repr(C)]
struct DrawIconQuad {
    draw_super: DrawQuad,
    is_even: f32,
    scale: f32,
    is_folder: f32,
    focussed: f32,
    selected: f32,
    hover: f32,
    opened: f32,
}

#[derive(Live, LiveHook)]
pub struct FileTreeNode {
    bg_quad: DrawBgQuad,
    icon_quad: DrawIconQuad,
    name_text: DrawNameText,
    layout: Layout,
    
    state: State,
    
    indent_width: f32,
    
    icon_walk: Walk,
    
    is_folder: bool,
    min_drag_distance: f32,
    
    opened: f32,
    focussed: f32,
    hover: f32,
    selected: f32,
}

#[derive(Live)]
pub struct FileTree {
    scroll_view: ScrollView,
    file_node: Option<LivePtr>,
    folder_node: Option<LivePtr>,
    layout: Layout,
    filler_quad: DrawBgQuad,
    
    node_height: f32,
    
    scroll_shadow: ScrollShadow,
    
    #[rust] dragging_node_id: Option<FileNodeId>,
    #[rust] selected_node_id: Option<FileNodeId>,
    #[rust] open_nodes: HashSet<FileNodeId>,
    
    #[rust] tree_nodes: ComponentMap<FileNodeId, (FileTreeNode, LiveId)>,
    
    #[rust] count: usize,
    #[rust] stack: Vec<f32>,
}

impl LiveHook for FileTree {
    fn after_apply(&mut self, cx: &mut Cx, from: ApplyFrom, index: usize, nodes: &[LiveNode]) {
        for (_, (tree_node, id)) in self.tree_nodes.iter_mut() {
            if let Some(index) = nodes.child_by_name(index, id.as_field()) {
                tree_node.apply(cx, from, index, nodes);
            }
        }
        self.scroll_view.redraw(cx);
    }
}

pub enum FileTreeAction {
    WasClicked(FileNodeId),
    ShouldStartDragging(FileNodeId),
}

pub enum FileTreeNodeAction {
    None,
    WasClicked,
    Opening,
    Closing,
    ShouldStartDragging,
}

impl FileTreeNode {
    pub fn set_draw_state(&mut self, is_even: f32, scale: f32) {
        self.bg_quad.scale = scale;
        self.bg_quad.is_even = is_even;
        self.name_text.scale = scale;
        self.name_text.is_even = is_even;
        self.icon_quad.scale = scale;
        self.icon_quad.is_even = is_even;
        self.name_text.font_scale = scale;
    }
    
    pub fn draw_folder(&mut self, cx: &mut Cx2d, name: &str, is_even: f32, node_height: f32, depth: usize, scale: f32) {
        self.set_draw_state(is_even, scale);
        
        self.bg_quad.begin(cx, Walk::size(Size::Fill, Size::Fixed(scale * node_height)), self.layout);
        
        cx.walk_turtle(self.indent_walk(depth));
        
        self.icon_quad.draw_walk(cx, self.icon_walk);
         
        self.name_text.draw_walk(cx, Walk::fit(), Align::default(), name);
        self.bg_quad.end(cx);
    }
    
    pub fn draw_file(&mut self, cx: &mut Cx2d, name: &str, is_even: f32, node_height: f32, depth: usize, scale: f32) {
        self.set_draw_state(is_even, scale);
        
        self.bg_quad.begin(cx, Walk::size(Size::Fill, Size::Fixed(scale * node_height)), self.layout);
        
        cx.walk_turtle(self.indent_walk(depth));
        
        self.name_text.draw_walk(cx, Walk::fit(), Align::default(), name);
        self.bg_quad.end(cx);
    }
    
    fn indent_walk(&self, depth: usize) -> Walk {
        Walk {
            abs_pos: None,
            width: Size::Fixed(depth as f32 * self.indent_width),
            height: Size::Fixed(0.0),
            margin: Margin {
                left: depth as f32 * 1.0,
                top: 0.0,
                right: depth as f32 * 4.0,
                bottom: 0.0,
            },
        }
    }
    
    fn set_is_selected(&mut self, cx: &mut Cx, is: bool, animate: Animate) {
        self.toggle_state(cx, is, animate, ids!(select.on), ids!(select.off))
    }
    
    fn set_is_focussed(&mut self, cx: &mut Cx, is: bool, animate: Animate) {
        self.toggle_state(cx, is, animate, ids!(focus.on), ids!(focus.off))
    }
    
    pub fn set_folder_is_open(&mut self, cx: &mut Cx, is: bool, animate: Animate) {
        self.toggle_state(cx, is, animate, ids!(open.on), ids!(open.off));
    }
    
    pub fn handle_event(
        &mut self,
        cx: &mut Cx,
        event: &mut Event,
        dispatch_action: &mut dyn FnMut(&mut Cx, FileTreeNodeAction),
    ) {
        if self.state_handle_event(cx, event).must_redraw() {
            self.bg_quad.area().redraw(cx);
        }
        match event.hits(cx, self.bg_quad.area()) {
            HitEvent::FingerHover(f) => {
                cx.set_hover_mouse_cursor(MouseCursor::Hand);
                match f.hover_state {
                    HoverState::In => {
                        self.animate_state(cx, ids!(hover.on));
                    }
                    HoverState::Out => {
                        self.animate_state(cx, ids!(hover.off));
                    }
                    _ => {}
                }
            }
            HitEvent::FingerMove(f) => {
                if f.abs.distance(&f.abs_start) >= self.min_drag_distance {
                    dispatch_action(cx, FileTreeNodeAction::ShouldStartDragging);
                }
            }
            HitEvent::FingerDown(_) => {
                self.animate_state(cx, ids!(select.on));
                if self.is_folder {
                    if self.state.is_in_state(cx, ids!(open.on)) {
                        self.animate_state(cx, ids!(open.off));
                        dispatch_action(cx, FileTreeNodeAction::Closing);
                    }
                    else {
                        self.animate_state(cx, ids!(open.on));
                        dispatch_action(cx, FileTreeNodeAction::Opening);
                    }
                    
                }
                dispatch_action(cx, FileTreeNodeAction::WasClicked);
            }
            _ => {}
        }
    }
}


impl FileTree {
    
    pub fn begin(&mut self, cx: &mut Cx2d) -> Result<(), ()> {
        self.scroll_view.begin(cx, Walk::default(), self.layout) ?;
        self.count = 0;
        Ok(())
    }
    
    pub fn end(&mut self, cx: &mut Cx2d) {
        // lets fill the space left with blanks
        let height_left = cx.turtle().height_left();
        let mut walk = 0.0;
        while walk < height_left {
            self.count += 1;
            self.filler_quad.is_even = Self::is_even(self.count);
            self.filler_quad.draw_walk(cx, Walk::size(Size::Fill, Size::Fixed(self.node_height.min(height_left - walk))));
            walk += self.node_height.max(1.0);
        }
        
        self.scroll_shadow.draw(cx, &self.scroll_view, vec2(0., 0.));
        self.scroll_view.end(cx);
        
        let selected_node_id = self.selected_node_id;
        self.tree_nodes.retain_visible_and( | node_id, _ | Some(*node_id) == selected_node_id);
    }
    
    pub fn is_even(count: usize) -> f32 {
        if count % 2 == 1 {0.0}else {1.0}
    }
    
    pub fn should_node_draw(&mut self, cx: &mut Cx2d) -> bool {
        let scale = self.stack.last().cloned().unwrap_or(1.0);
        let height = self.node_height * scale;
        let walk = Walk::size(Size::Fill, Size::Fixed(height));
        if scale > 0.01 && cx.walk_turtle_would_be_visible(walk, self.scroll_view.get_scroll_pos(cx)) {
            return true
        }
        else {
            cx.walk_turtle(walk);
            return false
        }
    }
    
    pub fn begin_folder(
        &mut self,
        cx: &mut Cx2d,
        node_id: FileNodeId,
        name: &str,
    ) -> Result<(), ()> {
        let scale = self.stack.last().cloned().unwrap_or(1.0);
        
        if scale > 0.2 {
            self.count += 1;
        }
        
        let is_open = self.open_nodes.contains(&node_id);
        
        if self.should_node_draw(cx) {
            let folder_node = self.folder_node;
            let (tree_node, _) = self.tree_nodes.get_or_insert(cx, node_id, | cx | {
                let mut tree_node = FileTreeNode::new_from_ptr(cx, folder_node);
                if is_open {
                    tree_node.set_folder_is_open(cx, true, Animate::No)
                }
                (tree_node, id!(folder_node))
            });
            
            tree_node.draw_folder(cx, name, Self::is_even(self.count), self.node_height, self.stack.len(), scale);
            self.stack.push(tree_node.opened * scale);
            if tree_node.opened == 0.0 {
                self.end_folder();
                return Err(());
            }
        }
        else {
            if is_open {
                self.stack.push(scale * 1.0);
            }
            else {
                return Err(());
            }
        }
        Ok(())
    }
    
    pub fn end_folder(&mut self) {
        self.stack.pop();
    }
    
    pub fn file(&mut self, cx: &mut Cx2d, node_id: FileNodeId, name: &str) {
        let scale = self.stack.last().cloned().unwrap_or(1.0);
        
        if scale > 0.2 {
            self.count += 1;
        }
        if self.should_node_draw(cx) {
            let file_node = self.file_node;
            let (tree_node, _) = self.tree_nodes.get_or_insert(cx, node_id, | cx | {
                (FileTreeNode::new_from_ptr(cx, file_node), id!(file_node))
            });
            tree_node.draw_file(cx, name, Self::is_even(self.count), self.node_height, self.stack.len(), scale);
        }
    }
    
    pub fn forget(&mut self) {
        self.tree_nodes.clear();
    }
    
    pub fn forget_node(&mut self, file_node_id: FileNodeId) {
        self.tree_nodes.remove(&file_node_id);
    }
    
    pub fn set_folder_is_open(
        &mut self,
        cx: &mut Cx,
        node_id: FileNodeId,
        is_open: bool,
        animate: Animate,
    ) {
        if is_open {
            self.open_nodes.insert(node_id);
        }
        else {
            self.open_nodes.remove(&node_id);
        }
        if let Some((tree_node, _)) = self.tree_nodes.get_mut(&node_id) {
            tree_node.set_folder_is_open(cx, is_open, animate);
        }
    }
    
    pub fn start_dragging_file_node(
        &mut self,
        cx: &mut Cx,
        node_id: FileNodeId,
        dragged_item: DraggedItem,
    ) {
        self.dragging_node_id = Some(node_id);
        cx.start_dragging(dragged_item);
    }
    
    pub fn redraw(&mut self, cx: &mut Cx) {
        self.scroll_view.redraw(cx);
    }
    
    pub fn handle_event(&mut self, cx: &mut Cx, event: &mut Event) -> Vec<FileTreeAction> {
        let mut a = Vec::new();
        self.handle_event_with_fn(cx, event, &mut | _, v | a.push(v));
        a
    }
    
    pub fn handle_event_with_fn(
        &mut self,
        cx: &mut Cx,
        event: &mut Event,
        dispatch_action: &mut dyn FnMut(&mut Cx, FileTreeAction),
    ) {
        if self.scroll_view.handle_event(cx, event) {
            self.scroll_view.redraw(cx);
        }
        
        match event {
            Event::DragEnd => self.dragging_node_id = None,
            _ => ()
        }
        
        let mut actions = Vec::new();
        for (node_id, (node, _)) in self.tree_nodes.iter_mut() {
            node.handle_event(cx, event, &mut | _, e | actions.push((*node_id, e)));
        }
        
        for (node_id, action) in actions {
            match action {
                FileTreeNodeAction::Opening => {
                    self.open_nodes.insert(node_id);
                }
                FileTreeNodeAction::Closing => {
                    self.open_nodes.remove(&node_id);
                }
                FileTreeNodeAction::WasClicked => {
                    cx.set_key_focus(self.scroll_view.area());
                    if let Some(last_selected) = self.selected_node_id {
                        if last_selected != node_id {
                            self.tree_nodes.get_mut(&last_selected).unwrap().0.set_is_selected(cx, false, Animate::Yes);
                        }
                    }
                    self.selected_node_id = Some(node_id);
                    dispatch_action(cx, FileTreeAction::WasClicked(node_id));
                }
                FileTreeNodeAction::ShouldStartDragging => {
                    if self.dragging_node_id.is_none() {
                        dispatch_action(cx, FileTreeAction::ShouldStartDragging(node_id));
                    }
                }
                _ => ()
            }
        }
        
        match event.hits(cx, self.scroll_view.area()) {
            HitEvent::KeyFocus(_) => {
                if let Some(node_id) = self.selected_node_id {
                    self.tree_nodes.get_mut(&node_id).unwrap().0.set_is_focussed(cx, true, Animate::Yes);
                }
            }
            HitEvent::KeyFocusLost(_) => {
                if let Some(node_id) = self.selected_node_id {
                    self.tree_nodes.get_mut(&node_id).unwrap().0.set_is_focussed(cx, false, Animate::Yes);
                }
            }
            _ => ()
        }
    }
}

#[derive(Clone, Debug, Default, Eq, Hash, Copy, PartialEq, FromLiveId)]
pub struct FileNodeId(pub LiveId);

