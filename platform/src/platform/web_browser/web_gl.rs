use {
    crate::{
        makepad_shader_compiler::{
            generate_glsl,
        },
        makepad_wasm_bridge::*,
        makepad_math::*,
        platform::{
            web_browser::{
                from_wasm::*
            }
        },
        cx::Cx,
        pass::{PassClearColor, PassClearDepth},
    },
};

impl Cx {
    pub fn render_view(
        &mut self,
        pass_id: usize,
        draw_list_id: usize,
        scroll: Vec2,
        clip: (Vec2, Vec2),
        zbias: &mut f32,
        zbias_step: f32
    ) {
        // tad ugly otherwise the borrow checker locks 'self' and we can't recur
        let draw_items_len = self.draw_lists[draw_list_id].draw_items_len;
        //self.views[view_id].set_clipping_uniforms();
        self.draw_lists[draw_list_id].uniform_view_transform(&Mat4::identity());
        self.draw_lists[draw_list_id].parent_scroll = scroll;
        let local_scroll = self.draw_lists[draw_list_id].get_local_scroll();
        let clip = self.draw_lists[draw_list_id].intersect_clip(clip);
        
        for draw_item_id in 0..draw_items_len {
            if let Some(sub_view_id) = self.draw_lists[draw_list_id].draw_items[draw_item_id].sub_view_id {
                self.render_view(
                    pass_id,
                    sub_view_id,
                    Vec2 {x: local_scroll.x + scroll.x, y: local_scroll.y + scroll.y},
                    clip,
                    zbias,
                    zbias_step,
                );
            }
            else {
                let draw_list = &mut self.draw_lists[draw_list_id];
                //view.platform.uni_vw.update_with_f32_data(device, &view.uniforms);
                let draw_call = draw_list.draw_items[draw_item_id].draw_call.as_mut().unwrap();
                let sh = &self.draw_shaders[draw_call.draw_shader.draw_shader_id];
                if sh.platform.is_none() { // shader didnt compile somehow
                    continue;
                }
                //let shp = &self.draw_shaders.platform[sh.platform.unwrap()];
                
                if draw_call.instance_dirty || draw_call.platform.inst_vb_id.is_none() {
                    draw_call.instance_dirty = false;
                    if draw_call.platform.inst_vb_id.is_none() {
                        draw_call.platform.inst_vb_id = Some(self.platform.vertex_buffers);
                        self.platform.vertex_buffers += 1;
                    }
                    
                    let slots = sh.mapping.instances.total_slots;
                    let instances = draw_call.instances.as_ref().unwrap().len() / slots;
                    
                    // lets patch up integer enums to floats because we dont support integers in attributes
                    for offset in &sh.mapping.instance_enums{
                        for i in 0..instances{
                            let instances = draw_call.instances.as_mut().unwrap();
                            let float = instances[i * sh.mapping.instances.total_slots + offset];
                            let integer : u32 = unsafe{std::mem::transmute(float)};
                            instances[i * sh.mapping.instances.total_slots + offset] = integer as f32;
                        }
                    }
                    
                    self.platform.from_wasm(FromWasmAllocArrayBuffer {
                        buffer_id: draw_call.platform.inst_vb_id.unwrap(),
                        data: WasmDataF32::new(draw_call.instances.as_ref().unwrap())
                    });
                    draw_call.instance_dirty = false;
                }
                
                draw_call.draw_uniforms.set_zbias(*zbias);
                draw_call.draw_uniforms.set_local_scroll(
                    scroll,
                    local_scroll,
                    &draw_call.options
                );
                draw_call.draw_uniforms.set_clip(clip);
                *zbias += zbias_step;
                
                // update/alloc textures?
                for i in 0..sh.mapping.textures.len() {
                    let texture_id = if let Some(texture_id) = draw_call.texture_slots[i] {
                        texture_id
                    }else {0};
                    
                    let cxtexture = &mut self.textures[texture_id as usize];
                    if cxtexture.update_image {
                        cxtexture.update_image = false;
                        self.platform.from_wasm(FromWasmAllocTextureImage2D {
                            texture_id,
                            width: cxtexture.desc.width.unwrap(),
                            height: cxtexture.desc.height.unwrap(),
                            data: WasmDataU32::new(&cxtexture.image_u32)
                        });
                    }
                }
                
                let geometry_id = if let Some(geometry_id) = draw_call.geometry_id {geometry_id}
                else {
                    continue;
                };
                
                let geometry = &mut self.geometries[geometry_id];
                
                if geometry.dirty || geometry.platform.vb_id.is_none() || geometry.platform.ib_id.is_none() {
                    if geometry.platform.vb_id.is_none() {
                        geometry.platform.vb_id = Some(self.platform.vertex_buffers);
                        self.platform.vertex_buffers += 1;
                    }
                    if geometry.platform.ib_id.is_none() {
                        geometry.platform.ib_id = Some(self.platform.index_buffers);
                        self.platform.index_buffers += 1;
                    }
                    self.platform.from_wasm(FromWasmAllocArrayBuffer {
                        buffer_id: geometry.platform.vb_id.unwrap(),
                        data: WasmDataF32::new(&geometry.vertices)
                    });
                    
                    self.platform.from_wasm(FromWasmAllocIndexBuffer {
                        buffer_id: geometry.platform.ib_id.unwrap(),
                        data: WasmDataU32::new(&geometry.indices)
                    });
                    
                    geometry.dirty = false;
                }
                
                // lets check if our vao is still valid
                if draw_call.platform.vao.is_none() {
                    draw_call.platform.vao = Some(CxPlatformDrawCallVao {
                        vao_id: self.platform.vaos,
                        shader_id: None,
                        inst_vb_id: None,
                        geom_vb_id: None,
                        geom_ib_id: None,
                    });
                    self.platform.vaos += 1;
                }
                
                let vao = draw_call.platform.vao.as_mut().unwrap();
                
                if vao.inst_vb_id != draw_call.platform.inst_vb_id
                    || vao.geom_vb_id != geometry.platform.vb_id
                    || vao.geom_ib_id != geometry.platform.ib_id
                    || vao.shader_id != Some(draw_call.draw_shader.draw_shader_id) {
                    
                    vao.shader_id = Some(draw_call.draw_shader.draw_shader_id);
                    vao.inst_vb_id = draw_call.platform.inst_vb_id;
                    vao.geom_vb_id = geometry.platform.vb_id;
                    vao.geom_ib_id = geometry.platform.ib_id;
                    
                    self.platform.from_wasm(FromWasmAllocVao {
                        vao_id: vao.vao_id,
                        shader_id: vao.shader_id.unwrap(),
                        geom_ib_id: vao.geom_ib_id.unwrap(),
                        geom_vb_id: vao.geom_vb_id.unwrap(),
                        inst_vb_id: draw_call.platform.inst_vb_id.unwrap()
                    });
                }
                
                let pass_uniforms = &self.passes[pass_id].pass_uniforms;
                
                self.platform.from_wasm(FromWasmDrawCall {
                    shader_id: draw_call.draw_shader.draw_shader_id,
                    vao_id: draw_call.platform.vao.as_ref().unwrap().vao_id,
                    pass_uniforms: WasmDataF32::new(pass_uniforms.as_slice()),
                    view_uniforms: WasmDataF32::new(draw_list.draw_list_uniforms.as_slice()),
                    draw_uniforms: WasmDataF32::new(draw_call.draw_uniforms.as_slice()),
                    user_uniforms: WasmDataF32::new(draw_call.user_uniforms.as_slice()),
                    live_uniforms: WasmDataF32::new(&sh.mapping.live_uniforms_buf),
                    const_table: WasmDataF32::new(&sh.mapping.const_table.table),
                    textures: draw_call.texture_slots.clone()
                });
            }
        }
        /*
        if let Some(_) = &self.views[view_id].debug {
            let mut s = String::new();
            self.debug_draw_tree_recur(false, &mut s, view_id, 0);
            console_log(&s);
        }*/
    }
    
    pub fn setup_render_pass(&mut self, pass_id: usize, inherit_dpi_factor: f32) {
        let pass_size = self.passes[pass_id].pass_size;
        self.passes[pass_id].set_matrix(Vec2::default(), pass_size);
        self.passes[pass_id].paint_dirty = false;
        
        let dpi_factor = if let Some(override_dpi_factor) = self.passes[pass_id].override_dpi_factor {
            override_dpi_factor
        }
        else {
            inherit_dpi_factor
        };
        self.passes[pass_id].set_dpi_factor(dpi_factor);
    }
    
    pub fn draw_pass_to_canvas(
        &mut self,
        pass_id: usize,
        dpi_factor: f32
    ) {
        let view_id = self.passes[pass_id].main_draw_list_id.unwrap();
        
        // get the color and depth
        let clear_color = if self.passes[pass_id].color_textures.len() == 0 {
            self.passes[pass_id].clear_color
        }
        else {
            match self.passes[pass_id].color_textures[0].clear_color {
                PassClearColor::InitWith(color) => color,
                PassClearColor::ClearWith(color) => color
            }
        };
        let clear_depth = match self.passes[pass_id].clear_depth {
            PassClearDepth::InitWith(depth) => depth,
            PassClearDepth::ClearWith(depth) => depth
        };
        
        self.platform.from_wasm(FromWasmBeginRenderCanvas {
            clear_color: clear_color.into(),
            clear_depth,
        });
        
        self.setup_render_pass(pass_id, dpi_factor);
        
        self.platform.from_wasm(FromWasmSetDefaultDepthAndBlendMode {});
        
        let mut zbias = 0.0;
        let zbias_step = self.passes[pass_id].zbias_step;

        self.render_view(
            pass_id,
            view_id,
            Vec2::default(),
            (Vec2 {x: -50000., y: -50000.}, Vec2 {x: 50000., y: 50000.}),
            &mut zbias,
            zbias_step
        );
    }
    
    pub fn draw_pass_to_texture(&mut self, pass_id: usize, dpi_factor: f32) {
        let pass_size = self.passes[pass_id].pass_size;
        let view_id = self.passes[pass_id].main_draw_list_id.unwrap();
        
        self.setup_render_pass(pass_id, dpi_factor);
        
        /*
        self.platform.from_wasm(FromWasmBeginRenderTargets {
            pass_id,
            width: (pass_size.x * dpi_factor) as usize,
            height: (pass_size.y * dpi_factor) as usize
        });*/
        
        let mut color_targets = [WColorTarget::default()];
        let mut depth_target = WDepthTarget::default();
        
        for (index, color_texture) in self.passes[pass_id].color_textures.iter().enumerate() {
            match color_texture.clear_color {
                PassClearColor::InitWith(clear_color) => {
                    color_targets[index] = WColorTarget{
                        texture_id: color_texture.texture_id,
                        init_only: true,
                        clear_color: clear_color.into()
                    };
                },
                PassClearColor::ClearWith(clear_color) => {
                    color_targets[index] = WColorTarget{
                        texture_id: color_texture.texture_id,
                        init_only: false,
                        clear_color: clear_color.into()
                    };
                }
            }
        }
        
        // attach/clear depth buffers, if any
        if let Some(depth_texture_id) = self.passes[pass_id].depth_texture {
            match self.passes[pass_id].clear_depth {
                PassClearDepth::InitWith(clear_depth) => {
                    depth_target = WDepthTarget{
                        texture_id: depth_texture_id,
                        init_only: true,
                        clear_depth
                    };
                },
                PassClearDepth::ClearWith(clear_depth) => {
                    depth_target = WDepthTarget{
                        texture_id: depth_texture_id,
                        init_only: false,
                        clear_depth
                    };
                }
            }
        }
        
        self.platform.from_wasm(FromWasmBeginRenderTexture {
            pass_id,
            width: (pass_size.x * dpi_factor) as usize,
            height: (pass_size.y * dpi_factor) as usize,
            color_targets,
            depth_target
        });
        
        // set the default depth and blendmode
        self.platform.from_wasm(FromWasmSetDefaultDepthAndBlendMode {});
        let mut zbias = 0.0;
        let zbias_step = self.passes[pass_id].zbias_step;
        
        self.render_view(
            pass_id,
            view_id,
            Vec2::default(),
            (Vec2 {x: -50000., y: -50000.}, Vec2 {x: 50000., y: 50000.}),
            &mut zbias,
            zbias_step
        );
    }
    
    pub fn webgl_compile_shaders(&mut self) {
        for draw_shader_ptr in &self.draw_shaders.compile_set {
            if let Some(item) = self.draw_shaders.ptr_to_item.get(&draw_shader_ptr) {
                let cx_shader = &mut self.draw_shaders.shaders[item.draw_shader_id];
                let draw_shader_def = self.shader_registry.draw_shader_defs.get(&draw_shader_ptr);
                
                let vertex = generate_glsl::generate_vertex_shader(
                    draw_shader_def.as_ref().unwrap(),
                    &cx_shader.mapping.const_table,
                    &self.shader_registry
                );
                let pixel = generate_glsl::generate_pixel_shader(
                    draw_shader_def.as_ref().unwrap(),
                    &cx_shader.mapping.const_table,
                    &self.shader_registry
                );
                
                if cx_shader.mapping.flags.debug {
                    println!("{}\n{}", vertex,pixel);
                }
                // lets see if we have the shader already
                for (index, ds) in self.draw_shaders.platform.iter().enumerate() {
                    if ds.vertex == vertex && ds.pixel == pixel {
                        cx_shader.platform = Some(index);
                        break;
                    }
                }
                if cx_shader.platform.is_none() {
                    let shp = CxPlatformDrawShader::new(vertex.clone(), pixel.clone());
                    self.platform.from_wasm(FromWasmCompileWebGLShader{
                        shader_id: item.draw_shader_id,
                        vertex: shp.vertex.clone(), 
                        pixel: shp.pixel.clone(),
                        geometry_slots: cx_shader.mapping.geometries.total_slots,
                        instance_slots: cx_shader.mapping.instances.total_slots,
                        /*
                        pass_uniforms_slots: cx_shader.mapping.pass_uniforms.total_slots,
                        view_uniforms_slots: cx_shader.mapping.view_uniforms.total_slots,
                        draw_uniforms_slots: cx_shader.mapping.draw_uniforms.total_slots,
                        user_uniforms_slots: cx_shader.mapping.user_uniforms.total_slots,
                        live_uniforms_slots: cx_shader.mapping.live_uniforms.total_slots,
                        const_table_slots:cx_shader.mapping.const_table.table.len(),
                        */
                        textures:cx_shader.mapping.textures.iter().map(|v| v.to_from_wasm_texture_input()).collect()
                    });
                    cx_shader.platform = Some(self.draw_shaders.platform.len());
                    self.draw_shaders.platform.push(shp);
                }
            }
        }
        self.draw_shaders.compile_set.clear();
    }
}

impl CxPlatformDrawShader{
    pub fn new(
        vertex: String,
        pixel: String,
    ) -> Self {
        
        let vertex = format!("
            precision highp float;
            precision highp int;
            vec4 sample2d(sampler2D sampler, vec2 pos){{return texture2D(sampler, vec2(pos.x, 1.0-pos.y));}}
            mat4 transpose(mat4 m){{return mat4(m[0][0],m[1][0],m[2][0],m[3][0],m[0][1],m[1][1],m[2][1],m[3][1],m[0][2],m[1][2],m[2][2],m[3][3], m[3][0], m[3][1], m[3][2], m[3][3]);}}
            mat3 transpose(mat3 m){{return mat3(m[0][0],m[1][0],m[2][0],m[0][1],m[1][1],m[2][1],m[0][2],m[1][2],m[2][2]);}}
            mat2 transpose(mat2 m){{return mat2(m[0][0],m[1][0],m[0][1],m[1][1]);}}
            {}", vertex);
            
        let pixel = format!("
            #extension GL_OES_standard_derivatives : enable
            precision highp float;
            precision highp int;
            vec4 sample2d(sampler2D sampler, vec2 pos){{return texture2D(sampler, vec2(pos.x, 1.0-pos.y));}}
            mat4 transpose(mat4 m){{return mat4(m[0][0],m[1][0],m[2][0],m[3][0],m[0][1],m[1][1],m[2][1],m[3][1],m[0][2],m[1][2],m[2][2],m[3][3], m[3][0], m[3][1], m[3][2], m[3][3]);}}
            mat3 transpose(mat3 m){{return mat3(m[0][0],m[1][0],m[2][0],m[0][1],m[1][1],m[2][1],m[0][2],m[1][2],m[2][2]);}}
            mat2 transpose(mat2 m){{return mat2(m[0][0],m[1][0],m[0][1],m[1][1]);}}
            {}", pixel);
        
        
        Self{
            vertex,
            pixel,
        }
    }
    
}

#[derive(Default, Clone, Debug)]
pub struct CxPlatformPass {
}

#[derive(Clone, Default)]
pub struct CxPlatformView {
}

#[derive(Default, Clone)]
pub struct CxPlatformDrawCallVao {
    pub vao_id: usize,
    pub shader_id: Option<usize>,
    pub inst_vb_id: Option<usize>,
    pub geom_vb_id: Option<usize>,
    pub geom_ib_id: Option<usize>,
}

#[derive(Default, Clone)]
pub struct CxPlatformDrawCall {
    pub vao: Option<CxPlatformDrawCallVao>,
    pub inst_vb_id: Option<usize>,
}

#[derive(Clone)]
pub struct CxPlatformDrawShader {
    pub vertex: String,
    pub pixel: String,
}

#[derive(Clone, Default)]
pub struct CxPlatformTexture {
}

#[derive(Clone, Default)]
pub struct CxPlatformGeometry {
    pub vb_id: Option<usize>,
    pub ib_id: Option<usize>
}

impl CxPlatformDrawCall {
}

use std::process::{Child};
pub fn spawn_process_command(_cmd: &str, _args: &[&str], _current_dir: &str) -> Result<Child, std::io::Error> {
    Err(std::io::Error::new(std::io::ErrorKind::NotFound, ""))
}
