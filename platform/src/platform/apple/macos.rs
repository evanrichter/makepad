use {
    std::cell::RefCell,
    std::sync::{Arc,Mutex},
    crate::{
        makepad_live_id::*,
        makepad_math::Vec2,
        platform::{
            core_midi::CoreMidiAccess,
            cocoa_app::CocoaApp,
            cx_desktop::CxDesktop,
            metal::{MetalCx, MetalWindow},
            audio_unit::*,
        },
        audio::{
            AudioTime,
            AudioOutputBuffer
        },
        midi::{
            Midi1InputData
        },
        event::{
            WebSocket,
            WebSocketAutoReconnect,
            Timer,
            Signal,
            SignalEvent,
            Event,
            MidiInputListEvent,
            DraggedItem
        },
        menu::Menu,
        cursor::MouseCursor,
        cx_api::{CxPlatformApi},
        cx::{Cx, PlatformType},
        window::{CxWindowState, CxWindowCmd},
        pass::CxPassParent,
        
    }
};

impl Cx {
    
    pub fn event_loop<F>(&mut self, mut event_handler: F)
    where F: FnMut(&mut Cx, &mut Event),
    {
        self.event_handler = Some(&mut event_handler as *const dyn FnMut(&mut Cx, &mut Event) as *mut dyn FnMut(&mut Cx, &mut Event));
        self.event_loop_core();
        self.event_handler = None;
    }
    
    pub fn event_loop_core(&mut self) {
        self.platform_type = PlatformType::OSX;
        
        let mut cocoa_app = CocoaApp::new();
        
        cocoa_app.init();
        
        let mut metal_cx = MetalCx::new();
        
        let mut metal_windows: Vec<MetalWindow> = Vec::new();
        
        //self.mtl_compile_all_shaders(&metal_cx);
        self.call_event_handler(&mut Event::Construct);
        
        self.redraw_all();
        
        let mut passes_todo = Vec::new();
        
        const KEEP_ALIVE_COUNT:usize = 5;
        let mut keep_alive_counter = 0;
        cocoa_app.start_timer(0, 0.2, true);

        cocoa_app.event_loop( | cocoa_app, events | {
            
            let mut paint_dirty = false;
            for mut event in events {
                match &event {
                    Event::FingerDown(_) |
                    Event::FingerMove(_) | 
                    Event::FingerHover(_) |
                    Event::FingerUp(_) |
                    Event::FingerScroll(_) |
                    Event::KeyDown(_) |
                    Event::KeyUp(_) |
                    Event::TextInput(_)=>{
                        keep_alive_counter = KEEP_ALIVE_COUNT;
                    }
                    Event::Timer(te)=>{
                        if te.timer_id == 0{
                            if keep_alive_counter>0{
                                keep_alive_counter -= 1;
                                self.repaint_windows();
                                paint_dirty = true;
                                continue;
                            }
                        }
                    }
                    _=>()
                }
                self.process_desktop_pre_event(&mut event);
                match &event {
                    Event::AppGotFocus=>{ // repaint all window passes. Metal sometimes doesnt flip buffers when hidden/no focus
                        for mw in metal_windows.iter_mut(){
                            if let Some(main_pass_id) = self.windows[mw.window_id].main_pass_id {
                                self.repaint_pass(main_pass_id);
                            }
                        }
                        paint_dirty = true;
                        self.call_event_handler(&mut event);
                    }
                    Event::WindowResizeLoop(wr) => {
                        if let Some(metal_window) = metal_windows.iter_mut().find(|w| w.window_id == wr.window_id){
                            if wr.was_started {
                                metal_window.start_resize();
                            }
                            else {
                                metal_window.stop_resize();
                            }
                        }
                    },
                    Event::WindowGeomChange(re) => { // do this here because mac
                        if let Some(metal_window) = metal_windows.iter_mut().find(|w| w.window_id == re.window_id){
                            metal_window.window_geom = re.new_geom.clone();
                            self.windows[re.window_id].window_geom = re.new_geom.clone();
                            // redraw just this windows root draw list
                            if re.old_geom.inner_size != re.new_geom.inner_size {
                                if let Some(main_pass_id) = self.windows[re.window_id].main_pass_id {
                                    self.redraw_pass_and_child_passes(main_pass_id);
                                }
                            }
                        }
                        // ok lets not redraw all, just this window
                        self.call_event_handler(&mut event);
                    },
                    Event::WindowClosed(wc) => {
                        // lets remove the window from the set
                        self.windows[wc.window_id].window_state = CxWindowState::Closed;
                        //self.windows_free.push(wc.window_id);
                        // remove the d3d11/win32 window
                        if let Some(index) = metal_windows.iter().position(|w| w.window_id == wc.window_id){
                            metal_windows.remove(index);
                            if metal_windows.len() == 0 {
                                cocoa_app.terminate_event_loop();
                            }
                        }
                        self.call_event_handler(&mut event);
                    },
                    Event::Paint => {
                        // construct or destruct windows
                        for (index, window) in self.windows.iter_mut().enumerate() {
                            
                            window.window_state = match &window.window_state {
                                CxWindowState::Create {inner_size, position, title} => {
                                    // lets create a platformwindow
                                    let metal_window = MetalWindow::new(index, &metal_cx, cocoa_app, *inner_size, *position, &title);
                                    window.window_geom = metal_window.window_geom.clone();
                                    metal_windows.push(metal_window);
                                    CxWindowState::Created
                                },
                                CxWindowState::Close => {
                                    for metal_window in &mut metal_windows {if metal_window.window_id == index {
                                        metal_window.cocoa_window.close_window();
                                        break;
                                    }}
                                    CxWindowState::Closed
                                },
                                CxWindowState::Created => CxWindowState::Created,
                                CxWindowState::Closed => CxWindowState::Closed
                            };
                            
                            window.window_command = match &window.window_command {
                                CxWindowCmd::Restore => {
                                    for metal_window in &mut metal_windows {if metal_window.window_id == index {
                                        metal_window.cocoa_window.restore();
                                    }}
                                    CxWindowCmd::None
                                },
                                CxWindowCmd::Maximize => {
                                    for metal_window in &mut metal_windows {if metal_window.window_id == index {
                                        metal_window.cocoa_window.maximize();
                                    }}
                                    CxWindowCmd::None
                                },
                                CxWindowCmd::Minimize => {
                                    for metal_window in &mut metal_windows {if metal_window.window_id == index {
                                        metal_window.cocoa_window.minimize();
                                    }}
                                    CxWindowCmd::None
                                },
                                _ => CxWindowCmd::None,
                            };
                            
                            if let Some(topmost) = window.window_topmost {
                                for metal_window in &mut metal_windows {if metal_window.window_id == index {
                                    metal_window.cocoa_window.set_topmost(topmost);
                                }}
                            }
                        }
                        
                        let _vsync = self.process_desktop_paint_callbacks(cocoa_app.time_now());
                        self.mtl_compile_shaders(&metal_cx);

                        
                        // set a cursor
                        if let Some(cursor) = self.down_mouse_cursor {
                            cocoa_app.set_mouse_cursor(cursor)
                        }
                        else if let Some(cursor) = self.hover_mouse_cursor{
                            cocoa_app.set_mouse_cursor(cursor)
                        }
                        else {
                            cocoa_app.set_mouse_cursor(MouseCursor::Default)
                        }
                        
                        if let Some(set_ime_position) = self.platform.set_ime_position {
                            self.platform.set_ime_position = None;
                            for metal_window in &mut metal_windows {
                                metal_window.cocoa_window.set_ime_spot(set_ime_position);
                            }
                        }
                        
                        while self.platform.start_timer.len() > 0 {
                            let (timer_id, interval, repeats) = self.platform.start_timer.pop().unwrap();
                            cocoa_app.start_timer(timer_id, interval, repeats);
                        }
                        
                        while self.platform.stop_timer.len() > 0 {
                            let timer_id = self.platform.stop_timer.pop().unwrap();
                            cocoa_app.stop_timer(timer_id);
                        }
                        
                        if self.platform.set_menu {
                            self.platform.set_menu = false;
                            if let Some(menu) = &self.platform.last_menu {
                                cocoa_app.update_app_menu(menu, &self.command_settings)
                            }
                        }
                        
                        // build a list of renderpasses to repaint
                        let mut windows_need_repaint = 0;
                        self.compute_passes_to_repaint(&mut passes_todo, &mut windows_need_repaint);
                        
                        if passes_todo.len() > 0 {
                            self.repaint_id += 1;
                            for pass_id in &passes_todo {
                                match self.passes[*pass_id].parent.clone() {
                                    CxPassParent::Window(window_id) => {
                                        // find the accompanying render window
                                        // its a render window
                                        windows_need_repaint -= 1;
                                        for metal_window in &mut metal_windows {if metal_window.window_id == window_id {
                                            let dpi_factor = metal_window.window_geom.dpi_factor;
                                            metal_window.resize_core_animation_layer(&metal_cx);
                                            
                                            self.draw_pass_to_layer(
                                                *pass_id,
                                                dpi_factor,
                                                metal_window.ca_layer,
                                                &mut metal_cx,
                                                metal_window.is_resizing
                                            );
                                        }}
                                        
                                    }
                                    CxPassParent::Pass(parent_pass_id) => {
                                        let dpi_factor = self.get_delegated_dpi_factor(parent_pass_id);
                                        self.draw_pass_to_texture(
                                            *pass_id,
                                            dpi_factor,
                                            &mut metal_cx,
                                        );
                                    },
                                    CxPassParent::None => {
                                        self.draw_pass_to_texture(
                                            *pass_id,
                                            1.0,
                                            &mut metal_cx,
                                        );
                                    }
                                }
                            }
                        }
                         //self.profile_end(0);
                        
                    },
                    Event::None => {
                    },
                    Event::Signal(se) => {
                        self.handle_core_midi_signals(se);
                        // this is a PostSignal
                        // see if it was a midi signal.
                        
                        self.call_event_handler(&mut event);
                        self.call_signals_and_triggers();
                    },
                    _ => {
                        self.call_event_handler(&mut event);
                        self.call_live_edit();
                        self.call_signals_and_triggers();
                    }
                }
                
                if let Some(dragged_item) = self.platform.start_dragging.take() {
                    cocoa_app.start_dragging(dragged_item);
                }
                
                if self.process_desktop_post_event(event) {
                    cocoa_app.terminate_event_loop();
                }
            }
            
            /* if self.live_styles.changed_live_bodies.len()>0 || self.live_styles.changed_deps.len()>0 {
                let changed_live_bodies = self.live_styles.changed_live_bodies.clone();
                let mut errors = self.process_live_styles_changes();
                self.mtl_update_all_shaders(&metal_cx, &mut errors);
                self.call_live_recompile_event(changed_live_bodies, errors);
            }
            
            self.process_live_style_errors();
            */
            if self.need_redrawing() || self.new_next_frames.len() != 0 || paint_dirty{
                false
            } else {
                true
            }
        })
    }

    fn handle_core_midi_signals(&mut self, se:&SignalEvent){
        if self.platform.midi_access.is_some(){
            if se.signals.contains(&id!(CoreMidiInputData).into()){
                let out_data = if let Ok(data) = self.platform.midi_input_data.lock(){
                    let mut data = data.borrow_mut();
                    let out_data = data.clone();
                    data.clear();
                    out_data
                }
                else{
                    panic!();
                };
                self.call_event_handler(&mut Event::Midi1InputData(out_data));
            }
            else if se.signals.contains(&id!(CoreMidiInputsChanged).into()){
                let inputs = self.platform.midi_access.as_ref().unwrap().connect_all_inputs();
                self.call_event_handler(&mut Event::MidiInputList(MidiInputListEvent{inputs}));
            }
        }
    }

}

impl CxPlatformApi for Cx{

    fn show_text_ime(&mut self, x: f32, y: f32) {
        self.platform.set_ime_position = Some(Vec2 {x: x, y: y});
    }
    
    fn hide_text_ime(&mut self) {
    }
    
    fn set_window_outer_size(&mut self, size: Vec2) {
        self.platform.set_window_outer_size = Some(size);
    }
    
    fn set_window_position(&mut self, pos: Vec2) {
        self.platform.set_window_position = Some(pos);
    }
    
    fn start_timer(&mut self, interval: f64, repeats: bool) -> Timer {
        self.timer_id += 1;
        self.platform.start_timer.push((self.timer_id, interval, repeats));
        Timer(self.timer_id)
    }
    
    fn stop_timer(&mut self, timer: Timer) {
        if timer.0 != 0 {
            self.platform.stop_timer.push(timer.0);
        }
    }    

    fn post_signal(signal: Signal) {
        CocoaApp::post_signal(signal.0.0);
    }
    
    fn spawn_thread<F>(&mut self, f: F) where F: FnOnce() + Send + 'static{
        std::thread::spawn(f);
    }

    fn web_socket_open(&mut self, _url: String, _rec: WebSocketAutoReconnect) -> WebSocket {
        todo!()
    }
    
    fn web_socket_send(&mut self, _websocket: WebSocket, _data: Vec<u8>) {
        todo!()
    }

    fn start_midi_input(&mut self){
        let midi_input_data = self.platform.midi_input_data.clone();
        if self.platform.midi_access.is_none(){
            if let Ok(ma) = CoreMidiAccess::new_midi_1_input(
                move |datas|{
                    if let Ok(midi_input_data) = midi_input_data.lock(){
                        let mut midi_input_data = midi_input_data.borrow_mut();
                        midi_input_data.extend_from_slice(&datas);
                        Cx::post_signal(id!(CoreMidiInputData).into());
                    }
                },
                move ||{
                    Cx::post_signal(id!(CoreMidiInputsChanged).into());
                }
            ){
                self.platform.midi_access = Some(ma);
            }
        }
        Cx::post_signal(id!(CoreMidiInputsChanged).into());
    }
    
    fn spawn_audio_output<F>(&mut self, f: F) where F: FnMut(AudioTime, &mut dyn AudioOutputBuffer) + Send + 'static{
        let fbox = std::sync::Arc::new(std::sync::Mutex::new(Box::new(f)));
        std::thread::spawn(move || {
            let out = &AudioUnitFactory::query_audio_units(AudioUnitType::DefaultOutput)[0];
            let fbox = fbox.clone();
            AudioUnitFactory::new_audio_unit(out, move | result | {
                match result {
                    Ok(audio_unit) => {
                        let fbox = fbox.clone();
                        audio_unit.set_input_callback(move |time, output|{
                            if let Ok(mut fbox) = fbox.lock(){
                                fbox(time, output);
                            }
                        });
                        loop {
                            std::thread::sleep(std::time::Duration::from_millis(100));
                        }
                    }
                    Err(err) => println!("Error {:?}", err)
                }
            });
        });
    }
    
    fn update_menu(&mut self, menu: &Menu) {
        // lets walk the menu and do the cocoa equivalents
        let platform = &mut self.platform;
        if platform.last_menu.is_none() || platform.last_menu.as_ref().unwrap() != menu {
            platform.last_menu = Some(menu.clone());
            platform.set_menu = true;
        }
    }
    
    fn start_dragging(&mut self, dragged_item: DraggedItem) {
        assert!(self.platform.start_dragging.is_none());
        self.platform.start_dragging = Some(dragged_item);
    }
}

#[derive(Default)]
pub struct CxPlatform {
    pub midi_access: Option<CoreMidiAccess>,
    pub midi_input_data: Arc<Mutex<RefCell<Vec<Midi1InputData>>>>,
    pub bytes_written: usize,
    pub draw_calls_done: usize,
    pub last_menu: Option<Menu>,
    pub set_menu: bool,
    pub set_window_position: Option<Vec2>,
    pub set_window_outer_size: Option<Vec2>,
    pub set_ime_position: Option<Vec2>,
    pub start_timer: Vec<(u64, f64, bool)>,
    pub stop_timer: Vec<u64>,
    pub text_clipboard_response: Option<String>,
    pub desktop: CxDesktop,
    pub start_dragging: Option<DraggedItem>,
}