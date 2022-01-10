//! Simulates `get_internal_gl()` in macroquad
#![deny(unsafe_code)]

use std::ops::{Deref, DerefMut};

/******************** lib.rs **********************/

/// parking_lot Mutext provides Sync and interior mutability
static CONTEXT: parking_lot::Mutex<Option<Context>> = parking_lot::const_mutex(None);

#[derive(Debug)]
struct Context {
    // exposed to user via InternalContext
    pub quad_context: u32,
    pub gl: u32,

    // only accessed directly from library
    screen_width: f32,
    screen_height: f32,
    mouse_x: f32,
    mouse_y: f32,
    touches: Vec<Touch>,
    simulate_mouse_with_touch: bool,
    audio_context: AudioContext,
}

impl Default for Context {
    fn default() -> Self {
        Self {
            quad_context: 0,
            gl: 0,
            screen_height: 0.0,
            screen_width: 0.0,
            mouse_x: 0.0,
            mouse_y: 0.0,
            touches: vec![],
            simulate_mouse_with_touch: true,
            audio_context: Default::default(),
        }
    }
}

impl Context {
    pub fn flush(&mut self) {
        self.perform_render_passes();
    }
}

#[derive(Debug, Default)]
struct AudioContext {
    sounds: Vec<u8>,
}

#[allow(dead_code)]
#[derive(Debug)]
struct Touch {
    is_touch_started: bool,
    x: f32,
    y: f32,
}

fn init_context() {
    *CONTEXT.lock() = Some(Context::default());
}

fn get_context() -> impl Deref<Target = Context> + DerefMut {
    let guard = CONTEXT.lock();
    parking_lot::MutexGuard::map(guard, |opt| match opt {
        None => panic!(),
        Some(ctx) => ctx,
    })
}

impl Context {
    pub(crate) fn perform_render_passes(&mut self) {
        self.quad_context += 1;
        self.gl += 1;
    }
}

fn resize_event(width: f32, height: f32) {
    let ctx = &mut *get_context();

    //mouse_motion_event(0., 0.);

    ctx.screen_height = height;
    ctx.screen_width = width;

    //mouse_motion_event(0., 0.);
}

fn mouse_motion_event(x: f32, y: f32) {
    (*get_context()).mouse_x = x;
    (*get_context()).mouse_y = y;
}

/// Since we call another "macroquad" functions `mouse_motion_event()`, we MUST
/// drop the `&mut Context` before calling those functions.
fn touch_event(is_touch_started: bool, x: f32, y: f32) {
    let simulate_mouse_with_touch = {
        // SAFETY: no calls to other macroquad functions in scope
        let context = &mut *get_context();

        context.touches.push(Touch {
            is_touch_started,
            x,
            y,
        });
        context.simulate_mouse_with_touch
    };

    // SAFETY: no mut ref to Context live
    #[allow(clippy::if_same_then_else)]
    if simulate_mouse_with_touch {
        if is_touch_started {
            //self.mouse_button_down_event(MouseButton::Left, x, y);
            mouse_motion_event(x, y); // call function that modifies context
        } else {
            // self.mouse_button_up_event(MouseButton::Left, x, y);
            mouse_motion_event(x, y); // call function that modifies context
        }
    };

    // SAFETY: no calls to other macroquad functions in scope
    let context = &mut *get_context();

    // context
    //     .input_events
    //     .iter_mut()
    //     .for_each(|arr| arr.push(MiniquadInputEvent::Touch { phase, id, x, y }));
    context.touches.push(Touch {
        is_touch_started,
        x: 100.0 + x,
        y: 100.0 + y,
    });
}

/******************** audio.rs **********************/

pub fn load_sound_from_bytes(data: &[u8]) {
    // SAFETY: no calls to other macroquad functions
    let context = &mut *get_context();

    let audio_context = &mut context.audio_context;
    context.mouse_x += 1.0;
    audio_context.sounds.extend_from_slice(data);
    context.mouse_x += 1.0;
    audio_context.sounds.extend_from_slice(data);
}

/******************** use of lib **********************/

fn helper() {
    // DEADLOCK here because Mutex is not re-entrant; we already locked inside
    // the "frame" loop
    let mut ctx = get_context();
    ctx.gl += 1;
}

fn main() {
    // Simulates Window::from_config()
    // must be called before `get_context()`
    init_context();

    // Simulate game loop
    for frame in 0..5 {
        let mut gl = get_context();
        gl.flush();
        gl.quad_context += 1;
        helper();

        resize_event(1920., 1080.);
        mouse_motion_event(42.0, 84.0);
        touch_event(true, frame as f32, frame as f32);
        load_sound_from_bytes(&[frame, frame]);
        gl.flush();
    }
    dbg!(&*get_context());
}
