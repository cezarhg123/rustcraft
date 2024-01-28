use ash::vk;
use gpu_allocator::vulkan::{Allocation, AllocationCreateDesc, AllocationScheme};
use image::GenericImageView;
use super::{buffer::Buffer, instance::{self, get_device, get_mut_allocator}, transition_image_layout};

pub struct Texture {
    image: vk::Image,
    allocation: Allocation,
    view: vk::ImageView,
    sampler: vk::Sampler,
    descriptor_image_info: vk::DescriptorImageInfo
}

impl Texture {
    pub fn new(dynamic_image: image::DynamicImage) -> Texture {
        unsafe {
            let image_bytes = dynamic_image.as_bytes();
            let dimensions = dynamic_image.dimensions();

            let raw_byte_buffer = Buffer::new(image_bytes, vk::BufferUsageFlags::TRANSFER_SRC, gpu_allocator::MemoryLocation::CpuToGpu);

            let image = get_device().create_image(
                &vk::ImageCreateInfo::builder()
                    .image_type(vk::ImageType::TYPE_2D)
                    .extent(
                        vk::Extent3D {
                            width: dimensions.0,
                            height: dimensions.1,
                            depth: 1
                        }
                    )
                    .mip_levels(1)
                    .array_layers(1)
                    .format(vk::Format::R8G8B8A8_SRGB)
                    .tiling(vk::ImageTiling::OPTIMAL)
                    .initial_layout(vk::ImageLayout::UNDEFINED)
                    .usage(vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED)
                    .sharing_mode(vk::SharingMode::EXCLUSIVE)
                    .samples(vk::SampleCountFlags::TYPE_1)
                    .build(),
                None
            ).unwrap();

            let requirements = get_device().get_image_memory_requirements(image);

            let allocation = get_mut_allocator().allocate(
                &AllocationCreateDesc {
                    name: "texture",
                    requirements,
                    location: gpu_allocator::MemoryLocation::GpuOnly,
                    linear: true,
                    allocation_scheme: AllocationScheme::GpuAllocatorManaged
                }
            ).unwrap();

            get_device().bind_image_memory(image, allocation.memory(), allocation.offset()).unwrap();

            transition_image_layout(image, vk::ImageLayout::UNDEFINED, vk::ImageLayout::TRANSFER_DST_OPTIMAL);
            let copy_command_buffer = instance::begin_single_exec_command();
            get_device().cmd_copy_buffer_to_image(
                copy_command_buffer,
                raw_byte_buffer.buffer(),
                image,
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
                            vk::Offset3D {
                                x: 0,
                                y: 0,
                                z: 0
                            }
                        )
                        .image_extent(
                            vk::Extent3D {
                                width: dimensions.0,
                                height: dimensions.1,
                                depth: 1
                            }
                        )
                        .build()
                ]
            );
            instance::end_single_exec_command(copy_command_buffer);
            transition_image_layout(image, vk::ImageLayout::TRANSFER_DST_OPTIMAL, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL);

            let view = get_device().create_image_view(
                &vk::ImageViewCreateInfo::builder()
                    .image(image)
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

            let sampler = get_device().create_sampler(
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
                image,
                allocation,
                view,
                sampler,
                descriptor_image_info
            }
        }
    }

    pub fn descirptor_image_info(&self) -> vk::DescriptorImageInfo {
        self.descriptor_image_info
    }
}
