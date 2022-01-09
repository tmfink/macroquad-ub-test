//! Simulates `get_internal_gl()` in macroquad

use std::cell::UnsafeCell;

/******************** lib.rs **********************/

/// Cell type that should be preferred over a `static mut` is better to use in a
/// `static`
///
/// Based on [@Nemo157 comment](issue-53639)
/// [issue-53639]: https://github.com/rust-lang/rust/issues/53639#issuecomment-790091647
///
/// # Safety
///
/// Mutable references must never alias (point to the same memory location).
/// Any previous result from calling this method on a specific instance
/// **must** be dropped before calling this function again. This applies
/// even if the mutable refernces lives in the stack frame of another function.
#[repr(transparent)]
#[derive(Debug)]
pub struct RacyCell<T>(UnsafeCell<T>);

impl<T> RacyCell<T> {
    #[inline(always)]
    pub const fn new(value: T) -> Self {
        RacyCell(UnsafeCell::new(value))
    }

    /// Get a shared reference to the inner type.
    ///
    /// # Safety
    /// See [RacyCell]
    #[inline(always)]
    pub unsafe fn get_ref(&self) -> &T {
        &*self.0.get()
    }

    /// Get a mutable reference to the inner type.
    /// Callers should try to restrict the lifetime as much as possible.
    ///
    /// Callers may want to convert this result to a `*mut T` immediately.
    ///
    /// # Safety
    /// See [RacyCell]
    #[allow(clippy::mut_from_ref)]
    #[inline(always)]
    pub unsafe fn get_ref_mut(&self) -> &mut T {
        &mut *self.0.get()
    }

    /// Get a const pointer to the inner type
    ///
    /// # Safety
    /// See [RacyCell]
    #[inline(always)]
    pub unsafe fn get_ptr(&self) -> *const T {
        self.0.get()
    }

    /// Get a mutable pointer to the inner type
    ///
    /// # Safety
    /// See [RacyCell]
    #[inline(always)]
    pub unsafe fn get_ptr_mut(&self) -> *mut T {
        self.0.get()
    }
}

unsafe impl<T> Sync for RacyCell<T> {}

#[no_mangle]
static CONTEXT: RacyCell<Option<Context>> = RacyCell::new(None);

#[derive(Debug)]
struct Context {
    // exposed to user via InternalContext
    quad_context: u32,
    gl: u32,

    // only accessed directly from library
    screen_width: f32,
    screen_height: f32,
    mouse_x: f32,
    mouse_y: f32,
    touches: Vec<Touch>,
    simulate_mouse_with_touch: bool,
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
        }
    }
}

#[allow(dead_code)]
#[derive(Debug)]
struct Touch {
    is_touch_started: bool,
    x: f32,
    y: f32,
}

/// # Safety
/// Requirements:
/// - this function must only be called once from the "main" thread
unsafe fn init_context() {
    *CONTEXT.get_ref_mut() = Some(Context::default())
}

/// # Safety
/// Requirements:
/// - `init_context()` must be called before this function.
/// - this function must only be called from the "main" thread
unsafe fn get_context() -> *mut Context {
    match CONTEXT.get_ref_mut() {
        None => panic!(),
        Some(x) => x,
    }
}

impl Context {
    pub(crate) fn perform_render_passes(&mut self) {
        self.quad_context += 1;
        self.gl += 1;
    }
}

fn resize_event(width: f32, height: f32) {
    unsafe {
        let ctx = &mut *get_context();

        // This would be UB because mouse_motion_event() mutates the underlying Context ctx mutable reference is live
        //mouse_motion_event(0., 0.);

        ctx.screen_height = height;
        ctx.screen_width = width;
    }
    // This is **not** UB because ctx mutable reference in this stack frame is no longer live
    //mouse_motion_event(0., 0.);
}

fn mouse_motion_event(x: f32, y: f32) {
    unsafe {
        (*get_context()).mouse_x = x;
        (*get_context()).mouse_y = y;
    }
}

/// Since we call another "macroquad" functions `mouse_motion_event()`, we MUST
/// drop the `&mut Context` before calling those functions.
fn touch_event(is_touch_started: bool, x: f32, y: f32) {
    let simulate_mouse_with_touch = {
        // SAFETY: no calls to other macroquad functions in scope
        let context = unsafe { &mut *get_context() };

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
    let context = unsafe { &mut *get_context() };

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

/******************** window.rs **********************/
pub struct InternalGlContext {
    pub quad_context: *mut u32,
    pub quad_gl: *mut u32,
}

impl InternalGlContext {
    pub fn flush(&mut self) {
        unsafe {
            let c = &mut *get_context();
            c.perform_render_passes();
        }
    }
}

/// # Safety
/// Requirements:
/// - this function must only be called from the "main" thread
pub unsafe fn get_internal_gl() -> InternalGlContext {
    let context = get_context();

    InternalGlContext {
        quad_context: &mut (*context).quad_context as *mut u32,
        quad_gl: &mut (*context).gl as *mut u32,
    }
}

/******************** use of lib **********************/

fn helper() {
    unsafe {
        let gl = get_internal_gl();
        *gl.quad_gl += 1;
    }
}

fn main() {
    // Simulates Window::from_config()
    unsafe {
        // must be called before `get_context()`
        init_context();
    }

    // Simulate game loop
    for frame in 0..5 {
        let mut gl = unsafe { get_internal_gl() };
        gl.flush();
        unsafe {
            *gl.quad_context += 1;
        }
        helper();

        resize_event(1920., 1080.);
        mouse_motion_event(42.0, 84.0);
        touch_event(true, frame as f32, frame as f32);
        gl.flush();
    }
    unsafe {
        dbg!(&*get_context());
    }
}
