pub mod vertex;
pub mod camera;
pub mod world;
pub mod thread_pool;

use std::{io::Cursor, sync::{Arc, RwLock}, time::Instant};
use camera::Camera;
use glfw::fail_on_errors;
use image::GenericImageView;
use vertex::Vertex;
use vust::{buffer::Buffer, create_info::VustCreateInfo, pipeline::{DescriptorSetBinding, DescriptorSetLayout, GraphicsPipeline, GraphicsPipelineCreateInfo}, texture::Texture, write_descriptor_info::WriteDescriptorInfo, Vust};
use winapi::um::libloaderapi::GetModuleHandleW;
use world::World;

pub const WINDOW_WIDTH: u32 = 1920;
pub const WINDOW_HEIGHT: u32 = 1080;
pub const WINDOW_TITLE: &str = "Rustcraft";

fn main() {
    let mut glfw = glfw::init(fail_on_errors!()).unwrap();
    glfw.window_hint(glfw::WindowHint::ClientApi(glfw::ClientApiHint::NoApi));
    glfw.window_hint(glfw::WindowHint::Resizable(false));
    glfw.window_hint(glfw::WindowHint::Decorated(false));


    let (mut window, _) = glfw
        .create_window(WINDOW_WIDTH, WINDOW_HEIGHT, WINDOW_TITLE, glfw::WindowMode::Windowed)
        .unwrap();

    let vust = Arc::new(RwLock::new(Vust::new(
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
    )));

    let mut camera = Camera::new(glm::vec3(0.0, 0.0, -2.0), Arc::clone(&vust));

    let mut world = World::new(26, Arc::clone(&vust));

    let mut frames = 0;
    let mut frame_time_instant = Instant::now();
    let mut delta_time_instant = Instant::now();
    while !window.should_close() {
        glfw.poll_events();
        
        let delta_time = delta_time_instant.elapsed().as_secs_f32();
        delta_time_instant = Instant::now();

        if frame_time_instant.elapsed().as_secs_f32() >= 1.0 {
            println!("FPS: {}", frames);
            frames = 0;
            frame_time_instant = Instant::now();
        } else {
            frames += 1;
        }

        camera.inputs(&mut window, delta_time);
        
        vust.write().unwrap().reset_command_buffer();
        world.update_chunks(camera.position(), Arc::clone(&vust));
        world.draw(&*vust.read().unwrap(), camera.position(), camera.buffer_info());
        vust.write().unwrap().render_surface();
    }

    vust.read().unwrap().wait_idle();
}