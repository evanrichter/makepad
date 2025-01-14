

pub mod platform;

#[macro_use]
mod live_prims;

#[macro_use]
mod cx;
mod cx_api;
mod cx_draw_shaders;

pub mod live_traits;
pub mod live_cx;

mod thread;
mod event;
mod area;
mod font;
mod window;
mod pass;
mod texture;
mod cursor;
mod menu;
mod state;
mod gpu_info;
mod draw_vars;
mod geometry;
mod draw_2d;
mod draw_3d;
mod draw_list;
mod shader;
pub mod audio;
pub mod midi;

#[cfg(target_arch = "wasm32")]
pub use makepad_wasm_bridge::{self, console_log};

pub use {
    makepad_shader_compiler,
    makepad_shader_compiler::makepad_derive_live,
    makepad_shader_compiler::makepad_math,
    makepad_shader_compiler::makepad_live_tokenizer,
    makepad_shader_compiler::makepad_micro_serde,
    makepad_shader_compiler::makepad_live_compiler,
    makepad_shader_compiler::makepad_live_id,
    makepad_derive_live::*,
    makepad_math::*,
    makepad_live_id::*,
    makepad_live_compiler::{
        vec4_ext::*,
        live_error_origin,
        LiveErrorOrigin,
        LiveNodeOrigin,
        LiveRegistry,
        LiveDocNodes,
        LiveId,
        LiveIdMap,
        LiveFileId,
        LivePtr,
        LiveRef,
        LiveNode,
        LiveType,
        LiveTypeInfo,
        LiveTypeField,
        LiveFieldKind,
        LiveComponentInfo,
        LiveComponentRegistry,
        LivePropType,
        LiveProp,
        LiveIdAsProp,
        //LiveTypeKind,
        LiveValue,
        FittedString,
        InlineString,
        LiveModuleId,
        LiveNodeSlice,
        LiveNodeVec,
    },
    makepad_shader_compiler::{
        ShaderRegistry,
        ShaderEnum,
        DrawShaderPtr,
        ShaderTy,
    },
    crate::{
        cx_api::{
            CxPlatformApi,
            profile_start,
            profile_end
        },
        /*cx_registries::{
            CxRegistries,
            CxRegistryNew,
        },*/
        cx_draw_shaders::{
        },
        cx::{
            Cx,
            PlatformType
        },
        area::{
            Area,
            DrawListArea,
            InstanceArea
        },
        event::{
            KeyCode,
            Event,
            HitEvent,
            DragEvent,
            Signal,
            WebSocket,
            WebSocketAutoReconnect,
            Timer,
            NextFrame,
            KeyModifiers,
            FingerInputType,
            FingerDownEvent,
            FingerMoveEvent,
            FingerUpEvent,
            HoverState,
            FingerHoverEvent,
            FingerScrollEvent,
            WindowGeomChangeEvent,
            WindowMovedEvent,
            NextFrameEvent,
            TimerEvent,
            SignalEvent,
            KeyEvent,
            KeyFocusEvent,
            TextInputEvent,
            TextCopyEvent,
            WindowCloseRequestedEvent,
            WindowClosedEvent,
            WindowResizeLoopEvent,
            WindowDragQueryResponse,
            WindowDragQueryEvent,
            XRButton,
            XRInput,
            XRUpdateEvent,
            FingerDragEvent,
            FingerDropEvent,
            DragState,
            DragAction,
            DraggedItem,
            HitOptions,
            FingerScrollHitEvent,
            FingerMoveHitEvent,
            FingerHoverHitEvent,
            FingerDownHitEvent,
            FingerUpHitEvent,
            FingerDragHitEvent,
            FingerDropHitEvent,
        },
        cursor::MouseCursor,
        menu::Menu,
        font::Font,
        draw_2d::{
            turtle::{
                Axis,
                Layout,
                Walk,
                Align,
                Margin,
                Padding,
                Flow,
                Size,
                DeferWalk
            },
            view::{
                View,
                ManyInstances,
                ViewRedraw
            },
            cx_2d::{
                Cx2d
            },
        },
        audio::{
            AudioTime,
            AudioOutputBuffer,
            AudioBuffer
        },
        midi::{
            Midi1InputData,
            Midi1Data,
            Midi1Note,
            Midi1Event
        },
        window::Window,
        pass::{
            Pass,
            PassClearColor,
            PassClearDepth
        },
        thread::{
            FromUIReceiver,
            FromUISender,
            ToUISender,
            ToUIReceiver,
            ThreadPool
        },
        texture::{
            Texture,
            TextureFormat,
            TextureDesc
        },
        live_traits::{
            LiveNew,
            LiveApply,
            LiveHook,
            LiveApplyValue,
            ToLiveValue,
            ApplyFrom,
            LiveBody,
        },
        state::{
            Ease,
            Play,
            Animate,
            LiveState,
            State,
            StateAction,
            StatePair
        },
        draw_vars::{
            DrawVars
        },
        geometry::{
            GeometryField,
            Geometry,
        },
        gpu_info::{
            GpuPerformance
        },
        draw_2d::{
            draw_shape::{DrawShape, Shape},
            draw_quad::DrawQuad,
            draw_text::{
                DrawText,
            },
            draw_color::DrawColor,
        },
        shader::{
            geometry_gen::{
                GeometryGen,
                GeometryQuad2D,
            },
        },
    },
};

