pub mod engine;
pub mod timer;
pub mod world;

use std::{io::Cursor, mem::size_of, time::Instant};

use ash::vk;
use engine::{camera::Camera, buffer::Buffer, vertex::Vertex, texture::Texture};
use timer::Timer;

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

    let vertex_buffer = Buffer::new(&[
        Vertex::new(glm::vec3(-0.5, -0.5, -2.0), glm::vec2(0.0, 0.0)),
        Vertex::new(glm::vec3(0.5, -0.5, -2.0), glm::vec2(1.0, 0.0)),
        Vertex::new(glm::vec3(0.0, 0.5, -2.0), glm::vec2(0.5, 1.0)),
    ], vk::BufferUsageFlags::VERTEX_BUFFER, vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT);

    let model = Buffer::new(&[glm::Mat4::new_scaling(1.0)], vk::BufferUsageFlags::UNIFORM_BUFFER, vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT);
    let model_info = vk::DescriptorBufferInfo::builder()
        .buffer(model.buffer())
        .range(size_of::<glm::Mat4>() as u64)
        .offset(0)
        .build();

    let descriptor_pool = engine::instance::create_descriptor_pool();
    let descriptor_set = unsafe {
        engine::instance::get_device().allocate_descriptor_sets(
            &vk::DescriptorSetAllocateInfo::builder()
                .descriptor_pool(descriptor_pool)
                .set_layouts(&[engine::instance::get_descriptor_set_layout()])
                .build()
        ).unwrap()[0]
    };

    let texture = Texture::new(image::load(Cursor::new(std::fs::read("textures/atlas.png").unwrap()), image::ImageFormat::Png).unwrap());

    // let mut delta_timer = Timer::new();
    let mut delta_prev_time = Instant::now();

    while !window.should_close() {
        glfw.poll_events();

        let delta_crnt_time = Instant::now();
        let delta_time = (delta_crnt_time - delta_prev_time).as_secs_f32();
        delta_prev_time = delta_crnt_time;

        camera.inputs(&mut window, delta_time);
        
        unsafe {
            engine::instance::get_device().update_descriptor_sets(
                &[
                    vk::WriteDescriptorSet::builder()
                        .dst_set(descriptor_set)
                        .dst_binding(0)
                        .dst_array_element(0)
                        .buffer_info(&[camera.descriptor_buffer_info()])
                        .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                        .build(),
                    vk::WriteDescriptorSet::builder()
                        .dst_set(descriptor_set)
                        .dst_binding(1)
                        .dst_array_element(0)
                        .image_info(&[texture.descriptor_image_info()])
                        .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                        .build(),
                    vk::WriteDescriptorSet::builder()
                        .dst_set(descriptor_set)
                        .dst_binding(2)
                        .dst_array_element(0)
                        .buffer_info(&[model_info])
                        .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                        .build()
                ],
                &[]
            );
        }

        engine::instance::draw(vertex_buffer.buffer(), descriptor_set, vertex_buffer.count());

        engine::instance::render_surface();
    }

    unsafe {
        engine::instance::get_device().device_wait_idle().unwrap();
    }
}
