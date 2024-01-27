pub mod vust;

use std::mem::size_of;
use ash::vk;
use glfw::fail_on_errors;
use gpu_allocator::vulkan::AllocationCreateDesc;
use vust::{instance::{get_allocator, get_device, get_mut_allocator, DrawCall}, vertex::Vertex};

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

    let test_buffer = unsafe {
        let buffer = get_device().create_buffer(
            &vk::BufferCreateInfo::builder()
                .size(size_of::<Vertex>() as u64 * 6)
                .usage(vk::BufferUsageFlags::VERTEX_BUFFER)
                .sharing_mode(vk::SharingMode::EXCLUSIVE)
                .build(),
            None
        ).unwrap();

        let requirements = get_device().get_buffer_memory_requirements(buffer);

        let allocation = get_mut_allocator().allocate(
            &AllocationCreateDesc {
                name: "test",
                requirements,
                location: gpu_allocator::MemoryLocation::CpuToGpu,
                linear: true,
                allocation_scheme: gpu_allocator::vulkan::AllocationScheme::GpuAllocatorManaged
            }
        ).unwrap();

        get_device().bind_buffer_memory(buffer, allocation.memory(), allocation.offset()).unwrap();

        let vertices = [
            Vertex {
                x: -0.5,
                y: -0.5,
                z: 0.0
            },
            Vertex {
                x: 0.5,
                y: 0.5,
                z: 0.0
            },
            Vertex {
                x: -0.5,
                y: 0.5,
                z: 0.0
            },

            Vertex {
                x: -0.5,
                y: -0.5,
                z: 0.0
            },
            Vertex {
                x: 0.5,
                y: -0.5,
                z: 0.0
            },
            Vertex {
                x: 0.5,
                y: 0.5,
                z: 0.0
            }
        ];

        let ptr = allocation.mapped_ptr().unwrap().as_ptr() as *mut Vertex;
        ptr.copy_from_nonoverlapping(vertices.as_ptr(), 6);

        (buffer, allocation)
    };

    while !window.should_close() {
        glfw.poll_events();

        vust::instance::reset_command_buffer();
        vust::instance::draw(DrawCall {
            buffer: test_buffer.0,
            vertex_count: 6
        });
        vust::instance::render_surface();
    }
}