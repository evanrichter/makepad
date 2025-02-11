use {
    std::ptr,
    std::mem,
    std::sync::{Arc, Mutex},
    crate::{
        audio::*,
        midi::*,
        platform::apple::cocoa_delegate::*,
        platform::apple::frameworks::*,
        platform::apple::cocoa_app::*,
        platform::apple::apple_util::*,
        objc_block,
    },
};


#[derive(Copy, Clone)]
pub enum AudioUnitType {
    DefaultOutput,
    MusicDevice,
    Effect
}

pub struct AudioUnitFactory {}

#[derive(Clone)]
pub struct AudioUnitInfo {
    pub name: String,
    pub unit_type: AudioUnitType,
    desc: CAudioComponentDescription
}

unsafe impl Send for AudioUnit {}
unsafe impl Sync for AudioUnit {}
pub struct AudioUnit {
    param_tree_observer: Option<KeyValueObserver>,
    av_audio_unit: ObjcId,
    au_audio_unit: ObjcId,
    render_block: Option<ObjcId>,
    view_controller: Arc<Mutex<Option<ObjcId >> >,
    unit_type: AudioUnitType
}

unsafe impl Send for AudioUnitClone {}
unsafe impl Sync for AudioUnitClone {}
pub struct AudioUnitClone {
    av_audio_unit: ObjcId,
    _au_audio_unit: ObjcId,
    render_block: Option<ObjcId>,
    unit_type: AudioUnitType
}

pub struct AudioUnitOutputBuffer {
    data: [*mut f32; MAX_AUDIO_BUFFERS],
    frame_count: usize,
    channel_count: usize
}

impl AudioOutputBuffer for AudioUnitOutputBuffer {
    fn frame_count(&self)->usize{self.frame_count}
    fn channel_count(&self)->usize{self.channel_count}

    fn channel_mut(&mut self, channel: usize) -> &mut [f32] {
        unsafe {
            std::slice::from_raw_parts_mut(
                self.data[channel] as *mut f32,
                self.frame_count as usize
            )
        }
    }
    
    fn zero(&mut self) {
        for i in 0..self.channel_count {
            let data = self.channel_mut(i);
            for j in 0..data.len() {
                data[j] = 0.0;
            }
        }
    }
    
    fn copy_from_buffer(&mut self, buffer:&AudioBuffer){
        if self.channel_count != buffer.channel_count{
            panic!("Output buffer channel_count != buffer channel_count {} {}",self.channel_count, buffer.channel_count );
        }
        if self.frame_count != buffer.frame_count{
            panic!("Output buffer frame_count != buffer frame_count ");
        }
        for i in 0..self.channel_count{
            let output = self.channel_mut(i);
            let input = buffer.channel(i);
            output.copy_from_slice(input);
        }
    }
}


impl AudioTime {
    fn to_audio_time_stamp(&self) -> CAudioTimeStamp {
        CAudioTimeStamp {
            mSampleTime: self.sample_time,
            mHostTime: self.host_time,
            mRateScalar: self.rate_scalar,
            mWordClockTime: 0,
            mSMPTETime: SMPTETime::default(),
            mFlags: 7,
            mReserved: 0
        }
    }
}

impl AudioBuffer {
    unsafe fn to_audio_buffer_list(&mut self) -> CAudioBufferList {
        let mut ab = CAudioBufferList {
            mNumberBuffers: self.channel_count.min(MAX_AUDIO_BUFFERS) as u32,
            mBuffers: [
                _AudioBuffer {
                    mNumberChannels: 0,
                    mDataByteSize: 0,
                    mData: 0 as *mut ::std::os::raw::c_void
                };
                MAX_AUDIO_BUFFERS
            ]
        };
        for i in 0..self.channel_count.min(MAX_AUDIO_BUFFERS) {
            ab.mBuffers[i] = _AudioBuffer {
                mNumberChannels: 0,
                mDataByteSize: (self.frame_count * 4) as u32,
                mData: self.channel_mut(i).as_ptr() as *mut ::std::os::raw::c_void,
            }
        }
        ab
    }
}
/*
fn print_hex(data: &[u8]) {
    // lets print hex data
    // we print 16 bytes per line
    let mut o = 0;
    while o < data.len() {
        let line = (data.len() - o).min(16);
        print!("{:08x} ", o);
        for i in o..o + line {
            print!("{:02x} ", data[i]);
        }
        for i in o..o + line {
            let c = data[i];
            if c>30 && c < 127 {
                print!("{}", c as char);
            }
            else {
                print!(".");
            }
        }
        println!("");
        o += line;
    }
}*/

#[derive(Default)]
pub struct AudioInstrumentState {
    manufacturer: u64,
    data: Vec<u8>,
    vstdata: Vec<u8>,
    subtype: u64,
    version: u64,
    ty: u64,
    name: String
}


impl AudioUnitClone {
    
    pub fn render_to_audio_buffer(&self, time: AudioTime, outputs: &mut [&mut AudioBuffer], inputs: &[&AudioBuffer]) {
        match self.unit_type {
            AudioUnitType::MusicDevice => (),
            AudioUnitType::Effect => (),
            _ => panic!("render_to_audio_buffer not supported on this device")
        }
        if let Some(render_block) = self.render_block {
            unsafe {
                let inputs_ptr = inputs.as_ptr() as *const *const AudioBuffer as u64;
                let inputs_len = inputs.len();
                let output_provider = objc_block!(
                    move | _flags: *mut u32,
                    _timestamp: *const CAudioTimeStamp,
                    frame_count: u32,
                    input_bus: u64,
                    buffers: *mut CAudioBufferList |: i32 {
                        let inputs = std::slice::from_raw_parts(
                            inputs_ptr as *const &AudioBuffer,
                            inputs_len as usize
                        );
                        let buffers = &*buffers;
                        let input_bus = input_bus as usize;
                        if input_bus >= inputs.len() {
                            println!("render_to_audio_buffer - input bus number > input len {} {}", input_bus, inputs.len());
                            return 0
                        }
                        // ok now..
                        if buffers.mNumberBuffers as usize != inputs[input_bus].channel_count {
                            println!("render_to_audio_buffer - input channel count doesnt match {} {}", buffers.mNumberBuffers, inputs[input_bus].channel_count);
                            return 0
                        }
                        if buffers.mNumberBuffers as usize > MAX_AUDIO_BUFFERS {
                            println!("render_to_audio_buffer - number of channels requested > MAX_AUDIO_BUFFER_LIST_SIZE");
                            return 0
                        }
                        if frame_count as usize != inputs[input_bus].frame_count {
                            println!("render_to_audio_buffer - frame count doesnt match {} {}", inputs[input_bus].frame_count, frame_count);
                            return 0
                        }
                        for i in 0..inputs[input_bus].channel_count {
                            // ok so. we have an inputs
                            let output_buffer = std::slice::from_raw_parts_mut(
                                buffers.mBuffers[i].mData as *mut f32,
                                frame_count as usize
                            );
                            let input_buffer = inputs[input_bus].channel(i);
                            output_buffer.copy_from_slice(input_buffer);
                        }
                        0
                    }
                );
                // ok we need to construct all these things
                let mut flags: u32 = 0;
                let timestamp = time.to_audio_time_stamp();
                for i in 0..outputs.len() {
                    let mut buffer_list = outputs[i].to_audio_buffer_list();
                    objc_block_invoke!(render_block, invoke(
                        (&mut flags as *mut u32): *mut u32,
                        (&timestamp as *const CAudioTimeStamp): *const CAudioTimeStamp,
                        (outputs[i].frame_count as u32): u32,
                        (i as u64): u64,
                        (&mut buffer_list as *mut CAudioBufferList): *mut CAudioBufferList,
                        (&output_provider as *const _ as ObjcId): ObjcId
                        //(nil): ObjcId
                    ) -> i32);
                }
            };
        }
    }
    
    pub fn handle_midi_1_data(&self, event: Midi1Data) {
        match self.unit_type {
            AudioUnitType::MusicDevice => (),
            _ => panic!("send_midi_1_event not supported on this device")
        }
        unsafe {
            let () = msg_send![self.av_audio_unit, sendMIDIEvent: event.data0 data1: event.data1 data2: event.data2];
        }
    }
}

impl AudioUnit {
    
    pub fn clone(&self) -> AudioUnitClone {
        AudioUnitClone {
            av_audio_unit: self.av_audio_unit,
            _au_audio_unit: self.au_audio_unit,
            render_block: self.render_block,
            unit_type: self.unit_type,
        }
    }
    
    pub fn parameter_tree_changed(&mut self, callback: Box<dyn Fn() + Send>) {
        if self.param_tree_observer.is_some() {
            panic!();
        }
        let observer = KeyValueObserver::new(self.au_audio_unit, "parameterTree", callback);
        self.param_tree_observer = Some(observer);
    }
    
    pub fn dump_parameter_tree(&self) {
        match self.unit_type {
            AudioUnitType::MusicDevice => (),
            AudioUnitType::Effect => (),
            _ => panic!("dump_parameter_tree on this device")
        }
        unsafe {
            let root: ObjcId = msg_send![self.au_audio_unit, parameterTree];
            unsafe fn recur_walk_tree(node: ObjcId, depth: usize) {
                let children: ObjcId = msg_send![node, children];
                let count: usize = msg_send![children, count];
                let param_group: ObjcId = msg_send![class!(AUParameterGroup), class];
                for i in 0..count {
                    let node: ObjcId = msg_send![children, objectAtIndex: i];
                    let class: ObjcId = msg_send![node, class];
                    let display: ObjcId = msg_send![node, displayName];
                    for _ in 0..depth {
                        print!("|  ");
                    }
                    if class == param_group {
                        recur_walk_tree(node, depth + 1);
                    }
                    else {
                        let min: f32 = msg_send![node, minValue];
                        let max: f32 = msg_send![node, maxValue];
                        let value: f32 = msg_send![node, value];
                        println!("{} : min:{} max:{} value:{}", nsstring_to_string(display), min, max, value);
                    }
                }
            }
            recur_walk_tree(root, 0);
        }
    }
    
    pub fn get_instrument_state(&self) -> AudioInstrumentState {
        match self.unit_type {
            AudioUnitType::MusicDevice => (),
            _ => panic!("start_audio_output_with_fn on this device")
        }
        unsafe {
            let dict: ObjcId = msg_send![self.au_audio_unit, fullState];
            let all_keys: ObjcId = msg_send![dict, allKeys];
            let count: usize = msg_send![all_keys, count];
            let mut out_state = AudioInstrumentState::default();
            for i in 0..count {
                let key: ObjcId = msg_send![all_keys, objectAtIndex: i];
                let obj: ObjcId = msg_send![dict, objectForKey: key];
                //let class: ObjcId = msg_send![obj, class]; nsstring_to_string(NSStringFromClass(class)),
                let name = nsstring_to_string(key);
                match name.as_ref() {
                    "manufacturer" => out_state.manufacturer = msg_send![obj, unsignedLongLongValue],
                    "subtype" => out_state.subtype = msg_send![obj, unsignedLongLongValue],
                    "version" => out_state.version = msg_send![obj, unsignedLongLongValue],
                    "type" => out_state.ty = msg_send![obj, unsignedLongLongValue],
                    "name" => out_state.name = nsstring_to_string(obj),
                    "data" => {
                        let len: usize = msg_send![obj, length];
                        if len > 0 {
                            let bytes: *const u8 = msg_send![obj, bytes];
                            out_state.data.extend_from_slice(std::slice::from_raw_parts(bytes, len));
                        }
                    }
                    "vstdata" => {
                        let len: usize = msg_send![obj, length];
                        if len > 0 {
                            let bytes: *const u8 = msg_send![obj, bytes];
                            out_state.vstdata.extend_from_slice(std::slice::from_raw_parts(bytes, len));
                            println!("{}", out_state.vstdata.len());
                        }
                    }
                    _ => {
                        eprintln!("Unexpected key in state dictionary {}", name);
                    }
                }
            }
            out_state
        }
    }
    
    pub fn set_instrument_state(&self, in_state: &AudioInstrumentState) {
        match self.unit_type {
            AudioUnitType::MusicDevice => (),
            _ => panic!("start_audio_output_with_fn on this device")
        }
        unsafe {
            let dict: ObjcId = msg_send![class!(NSMutableDictionary), dictionary];
            let () = msg_send![dict, init];
            
            unsafe fn set_number(dict: ObjcId, name: &str, value: u64) {
                let id: ObjcId = str_to_nsstring(name);
                let num: ObjcId = msg_send![class!(NSNumber), numberWithLongLong: value];
                let () = msg_send![dict, setObject: num forKey: id];
            }
            unsafe fn set_string(dict: ObjcId, name: &str, value: &str) {
                let id: ObjcId = str_to_nsstring(name);
                let value: ObjcId = str_to_nsstring(value);
                let () = msg_send![dict, setObject: value forKey: id];
            }
            unsafe fn set_data(dict: ObjcId, name: &str, data: &[u8]) {
                let id: ObjcId = str_to_nsstring(name);
                let nsdata: ObjcId = msg_send![class!(NSData), dataWithBytes: data.as_ptr() length: data.len()];
                let () = msg_send![dict, setObject: nsdata forKey: id];
            }
            set_number(dict, "manufacturer", in_state.manufacturer);
            set_number(dict, "subtype", in_state.subtype);
            set_number(dict, "version", in_state.version);
            set_number(dict, "type", in_state.ty);
            set_string(dict, "name", &in_state.name);
            set_data(dict, "data", &in_state.data);
            set_data(dict, "vstdata", &in_state.vstdata);
            
            let () = msg_send![self.au_audio_unit, setFullState: dict];
        }
    }
    
    pub fn set_input_callback<F: Fn(AudioTime, &mut dyn AudioOutputBuffer) + Send + 'static>(&self, audio_callback: F) {
        match self.unit_type {
            AudioUnitType::DefaultOutput => (),
            AudioUnitType::Effect => (),
            _ => panic!("set_input_callback on this device")
        }
        unsafe {
            let output_provider = objc_block!(
                move | _flags: *mut u32,
                timestamp: *const CAudioTimeStamp,
                frame_count: u32,
                _input_bus_number: u64,
                buffers: *mut CAudioBufferList |: i32 {
                    let buffers_ref = &*buffers;
                    //println!("IN OUTPUT {} {:?}", buffers_ref.mBuffers[0].mData as u64, *timestamp);
                    let mut output = AudioUnitOutputBuffer {
                        data: [0 as *mut f32; MAX_AUDIO_BUFFERS],
                        frame_count: frame_count as usize,
                        channel_count: buffers_ref.mNumberBuffers as usize
                    };
                    for i in 0..output.channel_count {
                        output.data[i] = buffers_ref.mBuffers[i].mData as *mut f32;
                    }
                    audio_callback(
                        AudioTime {
                            sample_time: (*timestamp).mSampleTime,
                            host_time: (*timestamp).mHostTime,
                            rate_scalar: (*timestamp).mRateScalar
                        },
                        &mut output
                    );
                    0
                }
            );
            let () = msg_send![self.au_audio_unit, setOutputProvider: &output_provider];
        }
    }
    
    pub fn request_ui<F: Fn() + Send + 'static>(&self, view_loaded: F) {
        match self.unit_type {
            AudioUnitType::MusicDevice => (),
            AudioUnitType::Effect => (),
            _ => panic!("request_ui not supported on this device")
        }
        
        let view_controller_arc = self.view_controller.clone();
        unsafe {
            let view_controller_complete = objc_block!(move | view_controller: ObjcId | {
                *view_controller_arc.lock().unwrap() = Some(view_controller);
                view_loaded();
            });
            
            let () = msg_send![self.au_audio_unit, requestViewControllerWithCompletionHandler: &view_controller_complete];
        }
    }
    
    pub fn open_ui(&self) {
        if let Some(view_controller) = self.view_controller.lock().unwrap().as_ref() {
            unsafe {
                let audio_view: ObjcId = msg_send![*view_controller, view];
                let cocoa_app = get_cocoa_app_global();
                let win_view = cocoa_app.cocoa_windows[0].1;
                let () = msg_send![win_view, addSubview: audio_view];
            }
        }
    }
    
    pub fn send_mouse_down(&self) {
        if let Some(_view_controller) = self.view_controller.lock().unwrap().as_ref() {
            unsafe {
                println!("Posting a doubleclick");
                let source = CGEventSourceCreate(1);
                /*
                let pos = NSPoint {x: 600.0, y: 720.0};
                let event = CGEventCreateMouseEvent(source, kCGEventLeftMouseDown, pos, 0);
                CGEventSetIntegerValueField(event, kCGMouseEventClickState, 1);
                CGEventPost(0, event);
                let event = CGEventCreateMouseEvent(source, kCGEventLeftMouseUp, pos, 0);
                CGEventSetIntegerValueField(event, kCGMouseEventClickState, 1);
                CGEventPost(0, event);
                let event = CGEventCreateMouseEvent(source, kCGEventLeftMouseDown, pos, 0);
                CGEventSetIntegerValueField(event, kCGMouseEventClickState, 2);
                CGEventPost(0, event);
                let event = CGEventCreateMouseEvent(source, kCGEventLeftMouseUp, pos, 0);
                CGEventSetIntegerValueField(event, kCGMouseEventClickState, 2);
                CGEventPost(0, event);
                */
                let event = CGEventCreateScrollWheelEvent(source, 0, 1, -24, 0, 0);
                CGEventPost(0, event);
                //CGEventPostToPid(pid, event);
            }
        }
    }
    
    pub fn ocr_ui(&self) {
        unsafe {
            let cocoa_app = get_cocoa_app_global();
            let window = cocoa_app.cocoa_windows[0].0;
            let win_num: u32 = msg_send![window, windowNumber];
            let win_opt = kCGWindowListOptionIncludingWindow;
            let null_rect: NSRect = NSRect {origin: NSPoint {x: f64::INFINITY, y: f64::INFINITY}, size: NSSize {width: 0.0, height: 0.0}};
            let cg_image: ObjcId = CGWindowListCreateImage(null_rect, win_opt, win_num, 1);
            if cg_image == nil {
                println!("Please add 'screen capture' privileges to the compiled binary in the macos settings");
                return
            }
            let handler: ObjcId = msg_send![class!(VNImageRequestHandler), alloc];
            let handler: ObjcId = msg_send![handler, initWithCGImage: cg_image options: nil];
            let start_time = std::time::Instant::now();
            let completion = objc_block!(move | request: ObjcId, error: ObjcId | {
                
                println!("Profile time {}", (start_time.elapsed().as_nanos() as f64) / 1000000f64);
                
                if error != nil {
                    println!("text recognition failed")
                }
                let results: ObjcId = msg_send![request, results];
                let count: usize = msg_send![results, count];
                for i in 0..count {
                    let obj: ObjcId = msg_send![results, objectAtIndex: i];
                    let top_objs: ObjcId = msg_send![obj, topCandidates: 1];
                    let top_obj: ObjcId = msg_send![top_objs, objectAtIndex: 0];
                    let _value: ObjcId = msg_send![top_obj, string];
                    //println!("Found text in UI: {}", nsstring_to_string(value));
                }
            });
            
            let request: ObjcId = msg_send![class!(VNRecognizeTextRequest), alloc];
            let request: ObjcId = msg_send![request, initWithCompletionHandler: &completion];
            let array: ObjcId = msg_send![class!(NSArray), arrayWithObject: request];
            let error: ObjcId = nil;
            let () = msg_send![handler, performRequests: array error: &error];
            if error != nil {
                println!("performRequests failed")
            }
        };
    }
    
}

#[derive(Debug)]
pub enum AudioError {
    System(String),
    NoDevice
}

impl AudioUnitFactory {
    
    pub fn query_audio_units(unit_type: AudioUnitType) -> Vec<AudioUnitInfo> {
        unsafe {
            let desc = match unit_type {
                AudioUnitType::MusicDevice => {
                    CAudioComponentDescription::new_all_manufacturers(
                        CAudioUnitType::MusicDevice,
                        CAudioUnitSubType::Undefined,
                    )
                }
                AudioUnitType::DefaultOutput => {
                    CAudioComponentDescription::new_apple(
                        CAudioUnitType::IO,
                        CAudioUnitSubType::DefaultOutput,
                    )
                }
                AudioUnitType::Effect => {
                    CAudioComponentDescription::new_all_manufacturers(
                        CAudioUnitType::Effect,
                        CAudioUnitSubType::Undefined,
                    )
                }
            };
            
            let manager: ObjcId = msg_send![class!(AVAudioUnitComponentManager), sharedAudioUnitComponentManager];
            let components: ObjcId = msg_send![manager, componentsMatchingDescription: desc];
            let count: usize = msg_send![components, count];
            let mut out = Vec::new();
            for i in 0..count {
                let component: ObjcId = msg_send![components, objectAtIndex: i];
                let name = nsstring_to_string(msg_send![component, name]);
                let desc: CAudioComponentDescription = msg_send!(component, audioComponentDescription);
                out.push(AudioUnitInfo {unit_type, name, desc});
            }
            out
        }
    }
    
    pub fn new_audio_unit<F: Fn(Result<AudioUnit, AudioError>) + Send + 'static>(
        unit_info: &AudioUnitInfo,
        unit_callback: F,
    ) {
        unsafe {
            let unit_type = unit_info.unit_type;
            let instantiation_handler = objc_block!(move | av_audio_unit: ObjcId, error: ObjcId | {
                let () = msg_send![av_audio_unit, retain];
                unsafe fn inner(av_audio_unit: ObjcId, error: ObjcId, unit_type: AudioUnitType) -> Result<AudioUnit, OSError> {
                    OSError::from_nserror(error) ?;
                    let au_audio_unit: ObjcId = msg_send![av_audio_unit, AUAudioUnit];
                    
                    let mut err: ObjcId = nil;
                    let () = msg_send![au_audio_unit, allocateRenderResourcesAndReturnError: &mut err];
                    OSError::from_nserror(err) ?;
                    let mut render_block = None;
                    match unit_type {
                        AudioUnitType::DefaultOutput => {
                            let () = msg_send![au_audio_unit, setOutputEnabled: true];
                            let mut err: ObjcId = nil;
                            let () = msg_send![au_audio_unit, startHardwareAndReturnError: &mut err];
                            OSError::from_nserror(err) ?;
                        }
                        AudioUnitType::MusicDevice => {
                            let block_ptr: ObjcId = msg_send![au_audio_unit, renderBlock];
                            let () = msg_send![block_ptr, retain];
                            render_block = Some(block_ptr);
                        }
                        AudioUnitType::Effect => {
                            let block_ptr: ObjcId = msg_send![au_audio_unit, renderBlock];
                            let input_busses: ObjcId = msg_send![au_audio_unit, inputBusses];
                            let count: usize = msg_send![input_busses, count];
                            if count > 0 {
                                // enable bus 0
                                let bus: ObjcId = msg_send![input_busses, objectAtIndexedSubscript: 0];
                                let () = msg_send![bus, setEnabled: true];
                            }
                            let () = msg_send![block_ptr, retain];
                            render_block = Some(block_ptr);
                        }
                    }
                    
                    Ok(AudioUnit {
                        view_controller: Arc::new(Mutex::new(None)),
                        param_tree_observer: None,
                        render_block,
                        unit_type,
                        av_audio_unit,
                        au_audio_unit
                    })
                }
                
                match inner(av_audio_unit, error, unit_type) {
                    Err(err) => unit_callback(Err(AudioError::System(format!("{:?}", err)))),
                    Ok(device) => unit_callback(Ok(device))
                }
            });
            
            // Instantiate output audio unit
            let () = msg_send![
                class!(AVAudioUnit),
                instantiateWithComponentDescription: unit_info.desc
                options: kAudioComponentInstantiation_LoadOutOfProcess
                completionHandler: &instantiation_handler
            ];
        }
    }
}