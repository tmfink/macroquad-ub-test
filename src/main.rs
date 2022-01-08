//! Simulates `get_internal_gl()` in macroquad

/******************** lib.rs **********************/

#[no_mangle]
static mut CONTEXT: Option<Context> = None;

struct Context {
    quad_context: u32,
    gl: u32,
}

fn get_context() -> &'static mut Context {
    unsafe { CONTEXT.as_mut().unwrap_or_else(|| panic!()) }
}

impl Context {
    pub(crate) fn perform_render_passes(&mut self) {
        self.quad_context += 1;
        self.gl += 1;
    }
}

/******************** window.rs **********************/
pub struct InternalGlContext<'a> {
    pub quad_context: &'a mut u32,
    pub quad_gl: &'a mut u32,
}

impl<'a> InternalGlContext<'a> {
    pub fn flush(&mut self) {
        get_context().perform_render_passes();
    }
}

pub unsafe fn get_internal_gl<'a>() -> InternalGlContext<'a> {
    let context = get_context();

    InternalGlContext {
        quad_context: &mut context.quad_context,
        quad_gl: &mut context.gl,
    }
}

/******************** use of lib **********************/
fn main() {
    // Simulates Window::from_config()
    unsafe {
        CONTEXT = Some(Context {
            quad_context: 0,
            gl: 0,
        })
    };

    // Simulate game loop
    {
        let mut gl = unsafe { get_internal_gl() };
        gl.flush();
        *gl.quad_context += 1; // miri complains here
    }
}
