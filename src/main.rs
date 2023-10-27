pub mod engine;
pub mod timer;
pub mod world;

use std::{io::Cursor, mem::size_of, time::Instant};
use ash::vk;
use engine::{camera::{Camera, CameraUniform}, buffer::Buffer, vertex::Vertex, texture::Texture};
use timer::Timer;
use world::{World, chunk::{Chunk}, block::{Block, BlockType}};

pub const WINDOW_WIDTH: u32 = 1920;
pub const WINDOW_HEIGHT: u32 = 1080;
pub const WINDOW_TITLE: &str = "RustCraft";

fn main() {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    glfw.window_hint(glfw::WindowHint::ClientApi(glfw::ClientApiHint::NoApi));
    glfw.window_hint(glfw::WindowHint::Resizable(false));
    glfw.window_hint(glfw::WindowHint::Decorated(false));

    let (mut window, _) = glfw.create_window(WINDOW_WIDTH, WINDOW_HEIGHT, WINDOW_TITLE, glfw::WindowMode::Windowed).unwrap();

    engine::instance::init(&glfw, &window);

    let mut camera = Camera::new(glm::vec3(0.0, 0.0, 0.0), glm::vec3(0.0, 0.0, -1.0));

    let texture = Texture::new(image::load(Cursor::new(std::fs::read("textures/atlas.png").unwrap()), image::ImageFormat::Png).unwrap().flipv());

    let mut world = World::new(8);
    // let mut chunk = Chunk::new(glm::vec3(0, -8, 0), |pos| {
    //     if pos.y < -2 {
    //         Block::new("Grass Block", "grass_block", BlockType::Solid, glm::vec2(0.0, 0.0), glm::vec2(0.1, 0.0), glm::vec2(0.2, 0.0))
    //     } else {
    //         Block::new("Air", "air", BlockType::Air, glm::vec2(0.9, 0.9), glm::vec2(0.9, 0.9), glm::vec2(0.9, 0.9))
    //     }
    // });
    // build_mesh(&chunk, [None, None, None, None, None, None]);

    let mut delta_timer = Timer::new();
    let mut fps_timer = Timer::new();
    let mut fps_counter = 0;

    while !window.should_close() {
        glfw.poll_events();

        delta_timer.tick();
        let delta_time = delta_timer.elapsed();
        delta_timer.reset();

        fps_timer.tick();
        fps_counter += 1;
        if fps_timer.elapsed() > 1.0 {
            println!("FPS: {}", fps_counter);
            fps_counter = 0;
            fps_timer.reset();
        }

        camera.inputs(&mut window, delta_time);

        // world.update_world(camera.position());

        world.draw(camera.descriptor_buffer_info(), texture.descriptor_image_info());
        // chunk.draw(camera.descriptor_buffer_info(), texture.descriptor_image_info());

        engine::instance::render_surface();
    }

    unsafe {
        engine::instance::get_device().device_wait_idle().unwrap();
    }
}
