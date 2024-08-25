use glfw::fail_on_errors;
use vust::{create_info::VustCreateInfo, Vust};
use winapi::um::libloaderapi::GetModuleHandleW;

pub const WINDOW_WIDTH: u32 = 1920;
pub const WINDOW_HEIGHT: u32 = 1080;
pub const WINDOW_TITLE: &str = "Rustcraft";

fn main() {
    let mut glfw = glfw::init(fail_on_errors!()).unwrap();
    glfw.window_hint(glfw::WindowHint::ClientApi(glfw::ClientApiHint::NoApi));
    glfw.window_hint(glfw::WindowHint::Resizable(false));
    glfw.window_hint(glfw::WindowHint::Decorated(false));


    let (window, _) = glfw
        .create_window(WINDOW_WIDTH, WINDOW_HEIGHT, WINDOW_TITLE, glfw::WindowMode::Windowed)
        .unwrap();

    let mut vust = Vust::new(
        VustCreateInfo::default()
            .with_app_name(WINDOW_TITLE)
            .with_app_version(vust::make_api_version(0, 0, 1, 0))
            .with_extensions(glfw.get_required_instance_extensions().unwrap())
            .with_surface_create_info(
                vust::create_info::SurfaceCreateInfo::Win32 {
                    hinstance: unsafe { GetModuleHandleW(std::ptr::null()).cast() },
                    hwnd: window.get_win32_window(),
                }
            )
            .with_framebuffer_size((window.get_framebuffer_size().0 as usize, window.get_framebuffer_size().1 as usize))
    );

    

    while !window.should_close() {
        glfw.poll_events();
    }
}