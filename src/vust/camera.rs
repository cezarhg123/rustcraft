use ash::vk;
use glfw::Window;
use crate::{WINDOW_HEIGHT, WINDOW_WIDTH};
use super::{buffer::Buffer, instance::get_device, UP};

pub struct Camera {
    position: glm::Vec3,
    direction: glm::Vec3,
    projection: glm::Mat4,
    view: glm::Mat4,
    uniform_buffer: Buffer,
    buffer_info: vk::DescriptorBufferInfo,
    rotating: bool,
    accept_input: bool
}

struct CameraUniform {
    projection: glm::Mat4,
    view: glm::Mat4
}

impl Camera {
    pub const SPEED: f32 = 5.0;
    pub const SENSITIVITY: f32 = 10.0;

    pub fn new(position: glm::Vec3) -> Camera {
        let projection = glm::perspective_fov_rh_zo(45.0f32.to_radians(), WINDOW_WIDTH as f32, WINDOW_HEIGHT as f32, 0.1, 1000.0);
        let view = glm::look_at_rh(&position, &(position + glm::vec3(0.0, 0.0, -1.0)), &UP);

        let uniform_buffer = Buffer::new(&[CameraUniform { projection, view }], vk::BufferUsageFlags::UNIFORM_BUFFER, gpu_allocator::MemoryLocation::CpuToGpu);

        Camera {
            position,
            direction: glm::vec3(0.0, 0.0, -1.0),
            projection,
            view,
            buffer_info: vk::DescriptorBufferInfo {
                buffer: uniform_buffer.buffer(),
                offset: 0,
                range: std::mem::size_of::<CameraUniform>() as u64
            },
            uniform_buffer,
            rotating: false,
            accept_input: true
        }
    }

    pub fn buffer_info(&self) -> vk::DescriptorBufferInfo {
        self.buffer_info
    }

    pub fn inputs(&mut self, window: &mut Window, delta_time: f32) {
        if window.get_key(glfw::Key::W) == glfw::Action::Press {
            self.position += (self.direction * Camera::SPEED) * delta_time;
        } else if window.get_key(glfw::Key::S) == glfw::Action::Press {
            self.position -= (self.direction * Camera::SPEED) * delta_time;
        }

        let right = glm::vec3(
            self.view.data.0[0][0],
            self.view.data.0[1][0],
            self.view.data.0[2][0]
        );

        if window.get_key(glfw::Key::A) == glfw::Action::Press {
            self.position -= (right * Camera::SPEED) * delta_time;
        } else if window.get_key(glfw::Key::D) == glfw::Action::Press {
            self.position += (right * Camera::SPEED) * delta_time;
        }

        let local_up = glm::vec3(
            self.view.data.0[0][1],
            self.view.data.0[1][1],
            self.view.data.0[2][1]
        );

        if window.get_key(glfw::Key::E) == glfw::Action::Press {
            self.position -= (local_up * Camera::SPEED) * delta_time;
        } else if window.get_key(glfw::Key::Q) == glfw::Action::Press {
            self.position += (local_up * Camera::SPEED) * delta_time;
        }

        // if right mouse is clicked then go into rotating mode, if clicked again then stop
        if window.get_mouse_button(glfw::MouseButtonRight) == glfw::Action::Press && self.accept_input {
            if self.rotating {
                self.rotating = false;

                window.set_cursor_mode(glfw::CursorMode::Normal);
            } else {
                self.rotating = true;

                window.set_cursor_mode(glfw::CursorMode::Hidden);
                window.set_cursor_pos(WINDOW_WIDTH as f64 / 2.0, WINDOW_HEIGHT as f64 / 2.0);
            }
            
            self.accept_input = false;
        } else if window.get_mouse_button(glfw::MouseButtonRight) == glfw::Action::Release {
            self.accept_input = true;
        }

        if self.rotating {
            let mouse_pos = window.get_cursor_pos();

            let delta_x = mouse_pos.0 - WINDOW_WIDTH as f64 / 2.0;
            let delta_y = mouse_pos.1 - WINDOW_HEIGHT as f64 / 2.0;

            let delta_time = delta_time.min(0.013);
            self.direction = glm::rotate_vec3(&self.direction, ((delta_y).to_radians() as f32 * Camera::SENSITIVITY) * delta_time, &glm::normalize(&glm::cross(&self.direction, &glm::vec3(0.0, 1.0, 0.0))));
            self.direction = glm::rotate_vec3(&self.direction, ((-delta_x).to_radians() as f32 * Camera::SENSITIVITY) * delta_time, &glm::vec3(0.0, 1.0, 0.0));
            window.set_cursor_pos(WINDOW_WIDTH as f64 / 2.0, WINDOW_HEIGHT as f64 / 2.0);
        }

        self.view = glm::look_at_rh(&self.position, &(self.position + self.direction), &UP);

        self.uniform_buffer.change_data(
            &[
                CameraUniform {
                    projection: self.projection.clone(),
                    view: self.view.clone()
                }
            ]
        );
    }
}
