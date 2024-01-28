use ash::vk;
use gpu_allocator::vulkan::{Allocation, AllocationCreateDesc, AllocationScheme};
use super::instance::{get_device, get_mut_allocator};

pub struct Buffer {
    usage: vk::BufferUsageFlags,
    buffer: vk::Buffer,
    memory: Allocation
}

impl Buffer {
    pub fn new<T>(data: &[T], usage: vk::BufferUsageFlags) -> Buffer {
        unsafe {
            let buffer = get_device().create_buffer(
                &vk::BufferCreateInfo::builder()
                    .size(std::mem::size_of_val(data) as u64)
                    .usage(usage)
                    .sharing_mode(vk::SharingMode::EXCLUSIVE)
                    .build(),
                None
            ).unwrap();

            let requirements = get_device().get_buffer_memory_requirements(buffer);

            let mut memory = get_mut_allocator().allocate(
                &AllocationCreateDesc {
                    name: format!("buffer// usage: {usage:?}, size: {}", requirements.size).as_str(),
                    requirements,
                    location: gpu_allocator::MemoryLocation::CpuToGpu,
                    linear: true,
                    allocation_scheme: AllocationScheme::GpuAllocatorManaged
                }
            ).unwrap();

            get_device().bind_buffer_memory(buffer, memory.memory(), memory.offset()).unwrap();

            let mapped = memory.mapped_slice_mut().unwrap().as_ptr() as *mut T;
            mapped.copy_from_nonoverlapping(data.as_ptr(), data.len());

            Buffer {
                usage,
                buffer,
                memory
            }
        }
    }

    pub fn change_data<T>(&mut self, data: &[T]) {
        unsafe {
            let data_size = std::mem::size_of_val(data) as u64;

            if data_size > self.memory.size() {
                self.memory = get_mut_allocator().allocate(
                    &AllocationCreateDesc {
                        name: format!("buffer// usage: {:?}, size: {}", self.usage, data_size).as_str(),
                        requirements: get_device().get_buffer_memory_requirements(self.buffer),
                        location: gpu_allocator::MemoryLocation::CpuToGpu,
                        linear: true,
                        allocation_scheme: AllocationScheme::GpuAllocatorManaged
                    }
                ).unwrap();
            }

            let mapped = self.memory.mapped_slice_mut().unwrap().as_ptr() as *mut T;
            mapped.copy_from_nonoverlapping(data.as_ptr(), data.len());
        }
    }

    pub fn buffer(&self) -> vk::Buffer {
        self.buffer
    }

    pub fn memory(&self) -> &Allocation {
        &self.memory
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe {
            get_device().destroy_buffer(self.buffer, None);
        }
    }
}
