//! Simulates `get_internal_gl()` in macroquad

use std::cell::UnsafeCell;

/******************** lib.rs **********************/

/// Cell type that should be preferred over a `static mut` is better to use in a
/// `static`
///
/// Based on [@Nemo157 comment](issue-53639)
/// [issue-53639]: https://github.com/rust-lang/rust/issues/53639#issuecomment-790091647
#[repr(transparent)]
#[derive(Debug)]
pub struct RacyCell<T>(UnsafeCell<T>);

impl<T> RacyCell<T> {
    #[inline(always)]
    pub const fn new(value: T) -> Self {
        RacyCell(UnsafeCell::new(value))
    }

    /// Get a shared reference to the inner type.
    #[inline(always)]
    pub unsafe fn get_ref(&self) -> &T {
        &*self.0.get()
    }

    /// Get a mutable reference to the inner type.
    ///
    /// # Safety
    ///
    /// Mutable references must never alias (point to the same memory location).
    /// Any previous result from calling this method on a specific instance
    /// **must** be dropped before calling this function again. This applies
    /// even if the mutable refernces lives in the stack frame of another function.
    ///
    /// Callers may want to convert this result to a `*mut T` immediately.
    #[allow(clippy::mut_from_ref)]
    #[inline(always)]
    pub unsafe fn get_ref_mut(&self) -> &mut T {
        &mut *self.0.get()
    }

    /// Get a const pointer to the inner type
    #[inline(always)]
    pub unsafe fn get_ptr(&self) -> *const T {
        self.0.get()
    }

    /// Get a mutable pointer to the inner type
    #[inline(always)]
    pub unsafe fn get_ptr_mut(&self) -> *mut T {
        self.0.get()
    }
}

unsafe impl<T> Sync for RacyCell<T> {}

#[no_mangle]
static CONTEXT: RacyCell<Option<Context>> = RacyCell::new(None);

#[derive(Default, Debug)]
struct Context {
    // exposed to user via InternalContext
    quad_context: u32,
    gl: u32,

    // only accessed directly from library
    screen_width: f32,
    screen_height: f32,
    mouse_x: f32,
    mouse_y: f32,
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
    for _ in 0..5 {
        let mut gl = unsafe { get_internal_gl() };
        gl.flush();
        unsafe {
            *gl.quad_context += 1;
        }
        helper();

        resize_event(1920., 1080.);
        mouse_motion_event(42.0, 84.0);
        gl.flush();
    }
    unsafe {
        dbg!(&*get_context());
    }
}
