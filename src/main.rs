pub mod vust;
pub mod timer;
pub mod world;
pub mod ptr_wrapper;

use std::{io::Cursor, mem::size_of};
use ash::vk;
use glfw::fail_on_errors;
use gpu_allocator::vulkan::AllocationCreateDesc;
use timer::Timer;
use vust::{buffer::Buffer, camera::Camera, instance::{get_allocator, get_device, get_mut_allocator, DrawCall}, texture::Texture, vertex::Vertex};
use world::World;

pub const WINDOW_WIDTH: u32 = 1920;
pub const WINDOW_HEIGHT: u32 = 1080;

fn main() {
    let mut glfw = glfw::init(fail_on_errors!()).unwrap();
    glfw.window_hint(glfw::WindowHint::ClientApi(glfw::ClientApiHint::NoApi));
    glfw.window_hint(glfw::WindowHint::Decorated(false));
    glfw.window_hint(glfw::WindowHint::Resizable(false));

    let (mut window, _) = glfw.create_window(WINDOW_WIDTH, WINDOW_HEIGHT, "Rustcraft", glfw::WindowMode::Windowed).unwrap();
    window.set_all_polling(true);

    vust::instance::init(&glfw, &window);

    let mut camera = Camera::new(glm::vec3(-2.0, -10.0, 100.0));

    let world = World::new();

    let mut delta_timer = Timer::new();
    let mut fps_timer = Timer::new();
    let mut fps_frames = 0;
    while !window.should_close() {
        glfw.poll_events();

        fps_timer.tick();
        fps_frames += 1;
        if fps_timer.elapsed() >= 1.0 {
            println!("FPS: {}", fps_frames);
            fps_timer.reset();
            fps_frames = 0;
        }

        delta_timer.tick();
        let delta_time = delta_timer.elapsed();
        delta_timer.reset();

        camera.inputs(&mut window, delta_time);

        vust::instance::reset_command_buffer();
        world.draw(camera.buffer_info());
        vust::instance::render_surface();
    }

    unsafe { get_device().device_wait_idle().unwrap(); }
}