use std::marker::PhantomData;
use ash::vk;
use super::instance::{self, find_memory_type, DEBUG};

#[derive(Debug)]
pub struct Buffer<T> {
    buffer: vk::Buffer,
    memory: vk::DeviceMemory,
    memory_type: vk::MemoryPropertyFlags,
    usage: vk::BufferUsageFlags,
    size: u64,
    count: u64,
    _phantom: PhantomData<T>
}

impl<T> Buffer<T> {
    /// if you want to clone the buffer make sure `usage` has `vk::BufferUsageFlags::TRANSFER_SRC`
    /// 
    /// returns None if data has size of 0
    pub fn new(data: &[T], usage: vk::BufferUsageFlags, wanted_memory: vk::MemoryPropertyFlags) -> Option<Buffer<T>> {
        unsafe {
            let device = instance::get_device();

            let data_size = (data.len() * std::mem::size_of::<T>()) as u64;
            if DEBUG {
                println!("Allocating {}B GPU memory", data_size);
            }

            if data_size == 0 {
                return None;
            }

            let buffer = device.create_buffer(
                &vk::BufferCreateInfo::builder()
                    .size(data_size)
                    .usage(usage)
                    .sharing_mode(vk::SharingMode::EXCLUSIVE)
                    .build(),
                None
            ).unwrap();

            let memory_requirements = device.get_buffer_memory_requirements(buffer);

            let memory = device.allocate_memory(
                &vk::MemoryAllocateInfo::builder()
                    .allocation_size(memory_requirements.size)
                    .memory_type_index(find_memory_type(instance::get_physical_memory_properties(), memory_requirements.memory_type_bits, wanted_memory).unwrap())
                    .build(),
                None
            ).unwrap();

            device.bind_buffer_memory(buffer, memory, 0).unwrap();

            let data_ptr = device.map_memory(memory, 0, data_size, vk::MemoryMapFlags::empty()).unwrap() as *mut T;
            data_ptr.copy_from_nonoverlapping(data.as_ptr(), data.len());
            device.unmap_memory(memory);

            Some(Buffer {
                buffer,
                memory,
                memory_type: wanted_memory,
                usage,
                size: data_size,
                count: data.len() as u64,
                _phantom: PhantomData
            })
        }
    }


    /// Errors `vk::Result::ERROR_UNKNOWN` if current buffer is stored in device local memory
    pub fn change_buffer(&mut self, data: &[T]) -> Result<(), vk::Result> {
        if self.memory_type.contains(vk::MemoryPropertyFlags::DEVICE_LOCAL) {
            return Err(vk::Result::ERROR_UNKNOWN);
        }

        unsafe {
            let data_size = (data.len() * std::mem::size_of::<T>()) as u64;

            let device = instance::get_device();

            let new_buffer = device.create_buffer(
                &vk::BufferCreateInfo::builder()
                    .size(data_size)
                    .usage(self.usage)
                    .sharing_mode(vk::SharingMode::EXCLUSIVE)
                    .build(),
                None
            )?;

            device.device_wait_idle().unwrap();
            device.destroy_buffer(self.buffer, None);
            self.buffer = new_buffer;

            // reallocate memory if previous amount was too small
            if data_size > self.size {
                if DEBUG {
                    println!("Reallocating {new_size}B GPU memory. ({old_size}B -> {new_size}B))", new_size=data_size, old_size=self.size);
                }

                let memory_requirements = device.get_buffer_memory_requirements(new_buffer);

                let new_memory = device.allocate_memory(
                    &vk::MemoryAllocateInfo::builder()
                        .allocation_size(memory_requirements.size)
                        .memory_type_index(find_memory_type(instance::get_physical_memory_properties(), memory_requirements.memory_type_bits, vk::MemoryPropertyFlags::HOST_VISIBLE).unwrap())
                        .build(),
                    None
                )?;

                device.free_memory(self.memory, None);
                self.memory = new_memory;
            }
            device.bind_buffer_memory(self.buffer, self.memory, 0)?;

            let data_ptr = device.map_memory(self.memory, 0, data_size, vk::MemoryMapFlags::empty())? as *mut T;
            data_ptr.copy_from_nonoverlapping(data.as_ptr(), data.len());
            device.unmap_memory(self.memory);

            Ok(())
        }
    }

    pub fn buffer(&self) -> vk::Buffer {
        self.buffer
    }

    pub fn usage(&self) -> vk::BufferUsageFlags {
        self.usage
    }

    pub fn size(&self) -> u64 {
        self.size
    }

    pub fn count(&self) -> u64 {
        self.count
    }
}

impl<T> Clone for Buffer<T> {
    /// creates a seperate buffer in vram and copys the data
    fn clone(&self) -> Buffer<T> {
        let device = instance::get_device();

        unsafe {
            let buffer = device.create_buffer(
                &vk::BufferCreateInfo::builder()
                    .size(self.size)
                    .usage(self.usage | vk::BufferUsageFlags::TRANSFER_DST)
                    .sharing_mode(vk::SharingMode::EXCLUSIVE)
                    .build(),
                None
            ).unwrap();
            
            let memory_requirements = device.get_buffer_memory_requirements(buffer);
            
            let memory = device.allocate_memory(
                &vk::MemoryAllocateInfo::builder()
                    .allocation_size(memory_requirements.size)
                    .memory_type_index(find_memory_type(instance::get_physical_memory_properties(), memory_requirements.memory_type_bits, self.memory_type).unwrap())
                    .build(),
                None
            ).unwrap();
            
            device.bind_buffer_memory(buffer, memory, 0).unwrap();

            let copy_command_buffer = instance::begin_single_exec_command();
            device.cmd_copy_buffer(copy_command_buffer, self.buffer, buffer, &[
                vk::BufferCopy {
                    src_offset: 0,
                    dst_offset: 0,
                    size: self.size
                }
            ]);
            instance::end_single_exec_command(copy_command_buffer);

            Buffer {
                buffer,
                memory,
                memory_type: self.memory_type,
                usage: self.usage,
                size: self.size,
                count: self.count,
                _phantom: PhantomData
            }
        }
    }
}

impl<T> Drop for Buffer<T> {
    fn drop(&mut self) {
        unsafe {
            let device = instance::get_device();

            device.destroy_buffer(self.buffer, None);
            device.free_memory(self.memory, None);
            if DEBUG {
                println!("deleted buffer: {}B", self.size);
            }
        }
    }
}
