//! Simulates `get_internal_gl()` in macroquad
#![deny(unsafe_code)]

/******************** lib.rs **********************/

#[derive(Debug)]
pub struct Context {
    // exposed to user
    pub quad_context: u32,
    pub gl: u32,

    // only accessed directly from library;
    // fields could be private if the function are moved into a Context impl
    // block
    pub(crate) screen_width: f32,
    pub(crate) screen_height: f32,
    pub(crate) mouse_x: f32,
    pub(crate) mouse_y: f32,
    pub(crate) touches: Vec<Touch>,
    pub(crate) simulate_mouse_with_touch: bool,
    pub(crate) audio_context: AudioContext,
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

    pub(crate) fn mouse_motion_event(&mut self, x: f32, y: f32) {
        self.mouse_x = x;
        self.mouse_y = y;
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

impl Context {
    pub(crate) fn perform_render_passes(&mut self) {
        self.quad_context += 1;
        self.gl += 1;
    }
}

fn resize_event(context: &mut Context, width: f32, height: f32) {
    //mouse_motion_event(0., 0.);

    context.screen_height = height;
    context.screen_width = width;

    //mouse_motion_event(0., 0.);
}

/// Since we call another "macroquad" functions `mouse_motion_event()`, we MUST
/// drop the `&mut Context` before calling those functions.
fn touch_event(context: &mut Context, is_touch_started: bool, x: f32, y: f32) {
    let simulate_mouse_with_touch = {
        context.touches.push(Touch {
            is_touch_started,
            x,
            y,
        });
        context.simulate_mouse_with_touch
    };

    #[allow(clippy::if_same_then_else)]
    if simulate_mouse_with_touch {
        if is_touch_started {
            //context.mouse_button_down_event(MouseButton::Left, x, y);
            context.mouse_motion_event(x, y); // call function that modifies context
        } else {
            // context.mouse_button_up_event(MouseButton::Left, x, y);
            context.mouse_motion_event(x, y); // call function that modifies context
        }
    };

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

pub fn load_sound_from_bytes(context: &mut Context, data: &[u8]) {
    let audio_context = &mut context.audio_context;
    context.mouse_x += 1.0;
    audio_context.sounds.extend_from_slice(data);
    context.mouse_x += 1.0;
    audio_context.sounds.extend_from_slice(data);
}

/******************** use of lib **********************/

fn helper(context: &mut Context) {
    context.gl += 1;
}

fn fake_user_main(context: &mut Context) {
    // Simulate game loop
    for frame in 0..5 {
        context.flush();
        context.quad_context += 1;
        helper(context);

        resize_event(context, 1920., 1080.);
        context.mouse_motion_event(42.0, 84.0);
        touch_event(context, true, frame as f32, frame as f32);
        load_sound_from_bytes(context, &[frame, frame]);
        context.flush();
    }
    dbg!(context);
}

fn main() {
    // construct context before calling user's main function
    let mut context = Context::default();

    fake_user_main(&mut context);
}
