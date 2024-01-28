pub mod vust;
pub mod timer;

use std::{io::Cursor, mem::size_of};
use ash::vk;
use glfw::fail_on_errors;
use gpu_allocator::vulkan::AllocationCreateDesc;
use timer::Timer;
use vust::{buffer::Buffer, camera::Camera, instance::{get_allocator, get_device, get_mut_allocator, DrawCall}, texture::Texture, vertex::Vertex};

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

    let vertices = [
            Vertex {
                x: -0.5,
                y: -0.5,
                z: 0.0,
                uv_x: 0.0,
                uv_y: 0.0
            },
            Vertex {
                x: 0.5,
                y: 0.5,
                z: 0.0,
                uv_x: 1.0,
                uv_y: 1.0
            },
            Vertex {
                x: -0.5,
                y: 0.5,
                z: 0.0,
                uv_x: 0.0,
                uv_y: 1.0
            },

            Vertex {
                x: -0.5,
                y: -0.5,
                z: 0.0,
                uv_x: 0.0,
                uv_y: 0.0
            },
            Vertex {
                x: 0.5,
                y: -0.5,
                z: 0.0,
                uv_x: 1.0,
                uv_y: 0.0
            },
            Vertex {
                x: 0.5,
                y: 0.5,
                z: 0.0,
                uv_x: 1.0,
                uv_y: 1.0
            }
        ];
    
    let test_buffer = Buffer::new(&vertices, vk::BufferUsageFlags::VERTEX_BUFFER, gpu_allocator::MemoryLocation::CpuToGpu);

    let test_texture = Texture::new(
        image::load(
            Cursor::new(include_bytes!("../textures/bruh.png")),
            image::ImageFormat::Png
        ).unwrap()
    );

    let mut camera = Camera::new(glm::vec3(0.0, 0.0, 3.0));

    let descriptor_pool = vust::instance::create_descriptor_pool();
    let descriptor_set = unsafe {
        get_device().allocate_descriptor_sets(
            &vk::DescriptorSetAllocateInfo::builder()
                .descriptor_pool(descriptor_pool)
                .set_layouts(&[*vust::instance::get_descriptor_set_layout()])
                .build()
        ).unwrap()[0]
    };

    let mut delta_timer = Timer::new();
    while !window.should_close() {
        glfw.poll_events();

        delta_timer.tick();
        let delta_time = delta_timer.elapsed();
        delta_timer.reset();

        camera.inputs(&mut window, delta_time);

        vust::instance::reset_command_buffer();
        unsafe {
            get_device().update_descriptor_sets(
                &[
                    vk::WriteDescriptorSet::builder()
                        .buffer_info(&[camera.buffer_info()])
                        .dst_binding(0)
                        .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                        .dst_set(descriptor_set)
                        .dst_array_element(0)
                        .build(),
                    vk::WriteDescriptorSet::builder()
                        .image_info(&[test_texture.descirptor_image_info()])
                        .dst_binding(1)
                        .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                        .dst_set(descriptor_set)
                        .dst_array_element(0)
                        .build()
                ],
                &[]
            );
        }
        vust::instance::draw(DrawCall {
            buffer: test_buffer.buffer(),
            descriptor_set,
            vertex_count: 6
        });
        vust::instance::render_surface();
    }

    unsafe { get_device().device_wait_idle().unwrap(); }
}