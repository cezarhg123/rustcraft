pub mod vertex;
pub mod camera;
pub mod world;

use std::{io::Cursor, sync::{Arc, RwLock}, time::Instant};
use camera::Camera;
use glfw::fail_on_errors;
use image::GenericImageView;
use vertex::Vertex;
use vust::{buffer::Buffer, create_info::VustCreateInfo, pipeline::{DescriptorSetBinding, DescriptorSetLayout, GraphicsPipeline, GraphicsPipelineCreateInfo}, texture::Texture, write_descriptor_info::WriteDescriptorInfo, Vust};
use winapi::um::libloaderapi::GetModuleHandleW;
use world::{chunk, World};

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

    let mut camera = Camera::new(glm::vec3(0.0, 10.0, -10.0), &vust);

    let mut world = World::new(8, &vust);

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
        
        vust.reset_command_buffer();
        let chunk_update_time = Instant::now();
        world.update_chunks(camera.position(), &vust);
        let chunk_update_time = chunk_update_time.elapsed().as_millis();
        if chunk_update_time > 10 {
            println!("Chunk update time: {}ms", chunk_update_time);
        }
        world.draw(&vust, camera.position(), camera.buffer_info());
        vust.render_surface();
    }

    vust.wait_idle();
}