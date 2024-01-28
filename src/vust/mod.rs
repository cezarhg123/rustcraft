macro_rules! make_static {
    ($name: ident, $type: ty) => {
        static mut $name: Option<$type> = None;
        
        paste::paste! {
            #[inline(always)]
            pub fn [<get_ $name>]() -> &'static $type {
                unsafe { $name.as_ref().unwrap() }
            }

            #[inline(always)]
            pub fn [<get_mut_ $name>]() -> &'static mut $type {
                unsafe { $name.as_mut().unwrap() }
            }
            
            #[inline(always)]
            pub fn [<set_ $name>](value: $type) {
                unsafe {
                    $name = Some(value);
                }
            }
        }
    };
}

#[allow(non_upper_case_globals)]
pub mod instance;
pub mod camera;
pub mod vertex;
pub mod buffer;
pub mod texture;

use ash::vk;
use self::instance::{begin_single_exec_command, end_single_exec_command, get_device};

pub const DEBUG: bool = cfg!(debug_assertions);
pub const UP: glm::Vec3 = glm::Vec3::new(0.0, 1.0, 0.0);

/// yoinked from ash examples
unsafe extern "system" fn vulkan_debug_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _user_data: *mut std::os::raw::c_void,
) -> vk::Bool32 {
    let callback_data = *p_callback_data;
    let message_id_number = callback_data.message_id_number;

    let message_id_name = if callback_data.p_message_id_name.is_null() {
        std::borrow::Cow::from("")
    } else {
        std::ffi::CStr::from_ptr(callback_data.p_message_id_name).to_string_lossy()
    };

    let message = if callback_data.p_message.is_null() {
        std::borrow::Cow::from("")
    } else {
        std::ffi::CStr::from_ptr(callback_data.p_message).to_string_lossy()
    };

    println!(
        "{message_severity:?}:\n{message_type:?} [{message_id_name} ({message_id_number})] : {message}\n",
    );

    vk::FALSE
}

/// ported from https://vulkan-tutorial.com
pub fn find_memory_type(
    memory_properties: vk::PhysicalDeviceMemoryProperties,
    type_filter: u32,
    properties: vk::MemoryPropertyFlags
) -> Option<u32> {
    for i in 0..memory_properties.memory_type_count {
        // dont really know how this works ¯\_(ツ)_/¯
        if (type_filter & (1 << i)) > 0 && ((memory_properties.memory_types[i as usize].property_flags & properties) == properties) {
            return Some(i as u32);
        }
    }

    None
}

pub fn transition_image_layout(
    image: vk::Image,
    old_layout: vk::ImageLayout,
    new_layout: vk::ImageLayout
) {
    let transition_command_buffer = begin_single_exec_command();

    let mut barrier = vk::ImageMemoryBarrier::builder()
        .old_layout(old_layout)
        .new_layout(new_layout)
        .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .image(image)
        .subresource_range(vk::ImageSubresourceRange::builder()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .base_mip_level(0)
            .level_count(1)
            .base_array_layer(0)
            .layer_count(1)
            .build())
        .build();
    
    let (src_stage, dst_stage) = if old_layout == vk::ImageLayout::UNDEFINED  && new_layout == vk::ImageLayout::TRANSFER_DST_OPTIMAL {
        barrier.src_access_mask = vk::AccessFlags::empty();
        barrier.dst_access_mask = vk::AccessFlags::TRANSFER_WRITE;

        (vk::PipelineStageFlags::TOP_OF_PIPE, vk::PipelineStageFlags::TRANSFER)
    } else if old_layout == vk::ImageLayout::TRANSFER_DST_OPTIMAL && new_layout == vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL {
        barrier.src_access_mask = vk::AccessFlags::TRANSFER_WRITE;
        barrier.dst_access_mask = vk::AccessFlags::SHADER_READ;

        (vk::PipelineStageFlags::TRANSFER, vk::PipelineStageFlags::FRAGMENT_SHADER)
    } else if old_layout == vk::ImageLayout::UNDEFINED && new_layout == vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL {
        barrier.subresource_range.aspect_mask = vk::ImageAspectFlags::DEPTH | vk::ImageAspectFlags::STENCIL;
        barrier.src_access_mask = vk::AccessFlags::empty();
        barrier.dst_access_mask = vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE;

        (vk::PipelineStageFlags::TOP_OF_PIPE, vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS)
    } else {
        unreachable!()
    };

    unsafe {
        get_device().cmd_pipeline_barrier(
            transition_command_buffer,
            src_stage,
            dst_stage,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &[barrier],
        );
    }

    end_single_exec_command(transition_command_buffer);
}
