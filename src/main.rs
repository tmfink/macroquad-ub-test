//! Simulates `get_internal_gl()` in macroquad

use std::{cell::UnsafeCell, marker::PhantomData, mem::MaybeUninit};

/******************** lib.rs **********************/

// from https://github.com/rust-lang/rust/issues/53639#issuecomment-790091647
struct RacyCell<T>(UnsafeCell<T>);

impl<T> RacyCell<T> {
    const fn new(value: T) -> Self {
        RacyCell(UnsafeCell::new(value))
    }

    unsafe fn get_mut_unchecked(&self) -> &mut T {
        &mut *self.0.get()
    }
}

unsafe impl<T> Sync for RacyCell<T> {}

#[no_mangle]
//static mut CONTEXT: Option<Context> = None;
static CONTEXT: RacyCell<MaybeUninit<Context>> = RacyCell::new(MaybeUninit::uninit());

struct Context {
    quad_context: u32,
    gl: u32,
}

unsafe fn init_context() {
    *CONTEXT.get_mut_unchecked() = MaybeUninit::new(Context {
        quad_context: 0,
        gl: 0,
    })
}

/// # SAFETY
/// Requirements:
/// - `init_context()` must be called before this function.
/// - this function must only be called from the "main" thread
unsafe fn get_context() -> *mut Context {
    CONTEXT.get_mut_unchecked().assume_init_mut()
}

impl Context {
    pub(crate) fn perform_render_passes(&mut self) {
        self.quad_context += 1;
        self.gl += 1;
    }
}

/******************** window.rs **********************/
pub struct InternalGlContext<'a> {
    pub quad_context: *mut u32,
    pub quad_gl: *mut u32,
    _phantom: PhantomData<&'a ()>,
}

impl<'a> InternalGlContext<'a> {
    pub fn flush(&mut self) {
        unsafe {
            (*get_context()).perform_render_passes();
        }
    }
}

pub unsafe fn get_internal_gl<'a>() -> InternalGlContext<'a> {
    let context = get_context();

    InternalGlContext {
        quad_context: &mut (*context).quad_context as *mut u32,
        quad_gl: &mut (*context).gl as *mut u32,
        _phantom: Default::default(),
    }
}

/******************** use of lib **********************/

fn helper() {
    unsafe {
        let mut gl = get_internal_gl();
        gl.flush();
        *gl.quad_context *= 2;
    }
}

fn main() {
    // Simulates Window::from_config()
    unsafe {
        // must be called before `get_context()`
        init_context();
    }

    // Simulate game loop
    {
        let mut gl = unsafe { get_internal_gl() };
        gl.flush();
        unsafe {
            *gl.quad_context += 1;
        }
        helper();
        gl.flush();
    }
}
