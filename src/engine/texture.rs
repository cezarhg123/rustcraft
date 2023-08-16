use ash::vk;
use image::{DynamicImage, GenericImageView, EncodableLayout};

use super::{instance::{self, transition_image_layout}, buffer::Buffer};

pub struct Texture {
    image: vk::Image,
    memory: vk::DeviceMemory,
    view: vk::ImageView,
    sampler: vk::Sampler,
    descriptor_image_info: vk::DescriptorImageInfo
}

impl Texture {
    pub fn new(image: DynamicImage) -> Texture {
        unsafe {
            let device = instance::get_device();

            let image_dims = image.dimensions();
            let image_buffer = Buffer::new(image.as_bytes(), vk::BufferUsageFlags::TRANSFER_SRC, vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT);

            let vk_image = device.create_image(
                &vk::ImageCreateInfo::builder()
                    .image_type(vk::ImageType::TYPE_2D)
                    .extent(vk::Extent3D {
                        width: image_dims.0,
                        height: image_dims.1,
                        depth: 1
                    })
                    .mip_levels(1)
                    .array_layers(1)
                    .format(vk::Format::R8G8B8A8_SRGB)
                    .tiling(vk::ImageTiling::OPTIMAL)
                    .initial_layout(vk::ImageLayout::UNDEFINED)
                    .usage(vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::TRANSFER_DST)
                    .samples(vk::SampleCountFlags::TYPE_1)
                    .sharing_mode(vk::SharingMode::EXCLUSIVE)
                    .build(),
                None
            ).unwrap();

            let memory_requirements = device.get_image_memory_requirements(vk_image);

            let memory = device.allocate_memory(
                &vk::MemoryAllocateInfo::builder()
                    .allocation_size(memory_requirements.size)
                    .memory_type_index(
                        instance::find_memory_type(
                            instance::get_physical_memory_properties(),
                            memory_requirements.memory_type_bits,
                            vk::MemoryPropertyFlags::DEVICE_LOCAL
                        ).unwrap()
                    )
                    .build(),
                None
            ).unwrap();

            device.bind_image_memory(vk_image, memory, 0).unwrap();

            transition_image_layout(vk_image, vk::ImageLayout::UNDEFINED, vk::ImageLayout::TRANSFER_DST_OPTIMAL);
            let copy_command_buffer = instance::begin_single_exec_command();
            device.cmd_copy_buffer_to_image(
                copy_command_buffer,
                image_buffer.buffer(),
                vk_image,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &[
                    vk::BufferImageCopy::builder()
                        .buffer_offset(0)
                        .buffer_row_length(0)
                        .buffer_image_height(0)
                        .image_subresource(
                            vk::ImageSubresourceLayers::builder()
                                .aspect_mask(vk::ImageAspectFlags::COLOR)
                                .mip_level(0)
                                .base_array_layer(0)
                                .layer_count(1)
                                .build()
                        )
                        .image_offset(
                            vk::Offset3D::builder()
                                .x(0)
                                .y(0)
                                .z(0)
                                .build()
                        )
                        .image_extent(
                            vk::Extent3D::builder()
                                .width(image_dims.0)
                                .height(image_dims.1)
                                .depth(1)
                                .build()
                        )
                        .build()
                ]
            );
            instance::end_single_exec_command(copy_command_buffer);

            transition_image_layout(vk_image, vk::ImageLayout::TRANSFER_DST_OPTIMAL, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL);

            let view = device.create_image_view(
                &vk::ImageViewCreateInfo::builder()
                    .image(vk_image)
                    .view_type(vk::ImageViewType::TYPE_2D)
                    .format(vk::Format::R8G8B8A8_SRGB)
                    .subresource_range(
                        vk::ImageSubresourceRange::builder()
                            .aspect_mask(vk::ImageAspectFlags::COLOR)
                            .base_mip_level(0)
                            .level_count(1)
                            .base_array_layer(0)
                            .layer_count(1)
                            .build()
                    )
                    .build(),
                None
            ).unwrap();

            let sampler = device.create_sampler(
                &vk::SamplerCreateInfo::builder()
                    .mag_filter(vk::Filter::NEAREST)
                    .min_filter(vk::Filter::NEAREST)
                    .mipmap_mode(vk::SamplerMipmapMode::NEAREST)
                    .address_mode_u(vk::SamplerAddressMode::REPEAT)
                    .address_mode_v(vk::SamplerAddressMode::REPEAT)
                    .address_mode_w(vk::SamplerAddressMode::REPEAT)
                    .anisotropy_enable(true)
                    .max_anisotropy(16.0)
                    .border_color(vk::BorderColor::INT_OPAQUE_BLACK)
                    .unnormalized_coordinates(false)
                    .compare_enable(false)
                    .compare_op(vk::CompareOp::ALWAYS)
                    .mip_lod_bias(0.0)
                    .min_lod(0.0)
                    .max_lod(0.0)
                    .build(),
                None
            ).unwrap();

            let descriptor_image_info = vk::DescriptorImageInfo::builder()
                .image_view(view)
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .sampler(sampler)
                .build();

            Texture {
                image: vk_image,
                memory,
                view,
                sampler,
                descriptor_image_info
            }
        }
    }

    pub fn descriptor_image_info(&self) -> vk::DescriptorImageInfo {
        self.descriptor_image_info
    }
}
