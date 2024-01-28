use std::{ffi::{CString, CStr}, mem::size_of, ptr::null};
use ash::vk;
use gpu_allocator::vulkan::{Allocator, AllocatorCreateDesc};
use winapi::um::libloaderapi::GetModuleHandleW;
use super::{vertex::Vertex, vulkan_debug_callback, DEBUG};

make_static!(entry, ash::Entry);
make_static!(instance, ash::Instance);

make_static!(debug_utils, ash::extensions::ext::DebugUtils);
make_static!(debug_messenger, vk::DebugUtilsMessengerEXT);

make_static!(gpu, vk::PhysicalDevice);
make_static!(gpu_memory_properties, vk::PhysicalDeviceMemoryProperties);

make_static!(device, ash::Device);
make_static!(graphics_queue_index, u32);
make_static!(graphics_queue, vk::Queue);

make_static!(surface_khr, vk::SurfaceKHR);
make_static!(surface_util, ash::extensions::khr::Surface);

make_static!(swapchain_khr, vk::SwapchainKHR);
make_static!(swapchain_util, ash::extensions::khr::Swapchain);

make_static!(swapchain_format, vk::SurfaceFormatKHR);
make_static!(swapchain_present_mode, vk::PresentModeKHR);
make_static!(extent, vk::Extent2D);

static mut swapchain_image_views: Vec<vk::ImageView> = Vec::new();
static mut swapchain_framebuffers: Vec<vk::Framebuffer> = Vec::new();

make_static!(render_pass, vk::RenderPass);
make_static!(pipeline_layout, vk::PipelineLayout);
make_static!(graphics_pipeline, vk::Pipeline);

make_static!(graphics_command_pool, vk::CommandPool);
make_static!(descriptor_set_layout, vk::DescriptorSetLayout);

make_static!(draw_command_buffer, vk::CommandBuffer);
make_static!(image_available_semaphore, vk::Semaphore);
make_static!(render_finished_semaphore, vk::Semaphore);
make_static!(in_flight_fence, vk::Fence);
static mut image_index: u32 = 0;

make_static!(allocator, gpu_allocator::vulkan::Allocator);

#[derive(Debug, Clone, Copy)]
pub struct DrawCall {
    pub buffer: vk::Buffer,
    pub descriptor_set: vk::DescriptorSet,
    pub vertex_count: u32
}

static mut draw_calls: Vec<DrawCall> = Vec::new();

pub fn init(glfw: &glfw::Glfw, window: &glfw::Window) {
    unsafe {
        create_instance(glfw);
        create_debug_utils();
        choose_physical_device();
        create_logical_device();
        create_surface(window);
        create_swapchain(window);
        create_command_pool();
        create_graphics_pipeline();
        create_framebuffers();
        create_draw_objects();

        set_allocator(Allocator::new(
            &AllocatorCreateDesc {
                instance: get_instance().clone(),
                device: get_device().clone(),
                physical_device: *get_gpu(),
                debug_settings: Default::default(),
                buffer_device_address: false,
                allocation_sizes: Default::default()
            }
        ).unwrap());
    }
}

unsafe fn create_instance(glfw: &glfw::Glfw) {
    set_entry(ash::Entry::load().unwrap());

    let c_name = CString::new("Rustcraft").unwrap();

    let mut supported_extensions = glfw.get_required_instance_extensions().unwrap();
    #[cfg(debug_assertions)]
    supported_extensions.push("VK_EXT_debug_utils".to_string());
    // add \0 to the end of every extension name
    let supported_extensions = supported_extensions.iter().map(|s| format!("{s}\0")).collect::<Vec<_>>();
    let supported_extension_ptrs = supported_extensions.iter().map(|s| s.as_ptr() as *const i8).collect::<Vec<_>>(); 
    
    let enabled_layer = ["VK_LAYER_KHRONOS_validation\0"];
    let enabled_layer_ptrs = enabled_layer.iter().map(|s| s.as_ptr() as *const i8).collect::<Vec<_>>();

    let app_info = vk::ApplicationInfo::builder()
        .application_name(&c_name)
        .application_version(vk::make_api_version(0, 1, 0, 0))
        .engine_name(&c_name)
        .engine_version(vk::make_api_version(0, 1, 0, 0))
        .api_version(vk::make_api_version(0, 1, 3, 0))
        .build();

    let create_info = vk::InstanceCreateInfo::builder()
        .application_info(&app_info)
        .enabled_extension_names(&supported_extension_ptrs)
        .enabled_layer_names(&enabled_layer_ptrs)
        .build();

    set_instance(entry.as_ref().unwrap().create_instance(&create_info, None).unwrap());
    if DEBUG {
        println!("Created Vulkan Instance");
    }
}

unsafe fn create_debug_utils() {
    if DEBUG {
        set_debug_utils(ash::extensions::ext::DebugUtils::new(get_entry(), get_instance()));

        set_debug_messenger(get_debug_utils().create_debug_utils_messenger(
            #[cfg(debug_assertions)]
            &vk::DebugUtilsMessengerCreateInfoEXT::builder()
                .message_severity( // information is empowering
                    vk::DebugUtilsMessageSeverityFlagsEXT::ERROR |
                    vk::DebugUtilsMessageSeverityFlagsEXT::WARNING |
                    vk::DebugUtilsMessageSeverityFlagsEXT::INFO |
                    vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE   
                )
                .message_type(
                    vk::DebugUtilsMessageTypeFlagsEXT::GENERAL |
                    vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE |
                    vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                )
                .pfn_user_callback(Some(vulkan_debug_callback))
                .build(),
            #[cfg(not(debug_assertions))]
            &vk::DebugUtilsMessengerCreateInfoEXT::builder().build(),
            None
        ).unwrap());

        if DEBUG {
            println!("Created Vulkan Debug Utils");
        }
    }
}

unsafe fn choose_physical_device() {
    set_gpu(get_instance().enumerate_physical_devices()
        .unwrap()
        .into_iter()
        .find(|p| {
            let properties = instance.as_ref().unwrap().get_physical_device_properties(*p);

            properties.device_type == vk::PhysicalDeviceType::DISCRETE_GPU ||
            properties.device_type == vk::PhysicalDeviceType::INTEGRATED_GPU
        })
        .expect("No discrete or integrated GPU found."));

    set_gpu_memory_properties(instance.as_ref().unwrap().get_physical_device_memory_properties(*get_gpu()));

    if DEBUG {
        let gpu_properties = get_instance().get_physical_device_properties(*get_gpu());
        
        let name = CStr::from_ptr(gpu_properties.device_name.as_ptr());
        println!("Using GPU: {}", name.to_str().unwrap());
        println!("GPU Type: {:#?}", gpu_properties.device_type);
    }
}

unsafe fn create_logical_device() {
    let queue_families = get_instance().get_physical_device_queue_family_properties(*get_gpu());
    let graphics_queue_family = queue_families
        .iter()
        .enumerate()
        .find(|(_, p)| {
            p.queue_flags.contains(vk::QueueFlags::GRAPHICS)
        })
        .expect("No graphics queue found");

    let mut physical_device_features = get_instance().get_physical_device_features(*get_gpu());
    physical_device_features.sampler_anisotropy = 1;

    let device_extensions = [
        "VK_KHR_swapchain\0"
    ];
    let device_extension_ptrs = device_extensions.iter().map(|s| s.as_ptr() as *const i8).collect::<Vec<_>>();

    let queue_create_infos = vec![
        vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(graphics_queue_family.0 as u32)
            .queue_priorities(&[1.0])
            .build()
    ];

    device = Some(
        instance.as_ref().unwrap().create_device(
            *get_gpu(),
            &vk::DeviceCreateInfo::builder()
                .queue_create_infos(&queue_create_infos)
                .enabled_extension_names(&device_extension_ptrs)
                .enabled_features(&physical_device_features)
                .build(),
            None
        ).unwrap()
    );

    set_graphics_queue_index(graphics_queue_family.0 as u32);
    set_graphics_queue(get_device().get_device_queue(graphics_queue_family.0 as u32, 0));

    if DEBUG {
        println!("Created Vulkan Logical Device");
    }
}

// currently only support win32
unsafe fn create_surface(window: &glfw::Window) {
    surface_util = Some(ash::extensions::khr::Surface::new(entry.as_ref().unwrap(), instance.as_ref().unwrap()));

    let win32_surface = ash::extensions::khr::Win32Surface::new(entry.as_ref().unwrap(), instance.as_ref().unwrap());
    surface_khr = win32_surface.create_win32_surface(
        &vk::Win32SurfaceCreateInfoKHR::builder()
            .hinstance(GetModuleHandleW(null()).cast())
            .hwnd(window.get_win32_window())
            .build(),
        None
    ).ok();

    if window.create_window_surface(instance.as_ref().unwrap().handle(), null(), &mut (get_surface_khr().clone())).result().is_err() {
        panic!("Failed to create window surface");
    }

    if DEBUG {
        println!("Created Vulkan Window Surface");
    }
}

unsafe fn create_swapchain(window: &glfw::Window) {
    swapchain_util = Some(ash::extensions::khr::Swapchain::new(instance.as_ref().unwrap(), device.as_ref().unwrap()));

    let capabilities = surface_util.as_ref().unwrap().get_physical_device_surface_capabilities(*get_gpu(), *get_surface_khr()).unwrap();

    swapchain_format = Some(
        *surface_util.as_ref().unwrap().get_physical_device_surface_formats(*get_gpu(), *get_surface_khr()).unwrap()
            .iter()
            .find(|f| f.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR && f.format == vk::Format::B8G8R8A8_SRGB)
            .unwrap()
    );

    swapchain_present_mode = Some(
        *surface_util.as_ref().unwrap().get_physical_device_surface_present_modes(*get_gpu(), *get_surface_khr()).unwrap().iter().find(|p| **p == vk::PresentModeKHR::IMMEDIATE).unwrap()
    );

    let framebuffer_size = window.get_framebuffer_size();

    extent = Some(vk::Extent2D {
        width: (framebuffer_size.0 as u32).clamp(capabilities.min_image_extent.width, capabilities.max_image_extent.width),
        height: (framebuffer_size.1 as u32).clamp(capabilities.min_image_extent.height, capabilities.max_image_extent.height),
    });

    set_swapchain_khr(swapchain_util.as_ref().unwrap().create_swapchain(
        &vk::SwapchainCreateInfoKHR::builder()
            .surface(*get_surface_khr())
            .min_image_count(capabilities.min_image_count + 1)
            .image_format(swapchain_format.unwrap().format)
            .image_color_space(swapchain_format.unwrap().color_space)
            .image_extent(extent.unwrap())
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE) // if i understand correctly, this should be the best option since im using the swapchain image on one queue(graphics)
            .pre_transform(capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(swapchain_present_mode.unwrap())
            .clipped(true)
            .old_swapchain(vk::SwapchainKHR::null())
            .build(),
            None
    ).unwrap());

    if DEBUG {
        println!("Created Swapchain");
    }

    let images = swapchain_util.as_ref().unwrap().get_swapchain_images(*get_swapchain_khr()).unwrap();

    swapchain_image_views = images.iter().map(|image| {
        device.as_ref().unwrap().create_image_view(
            &vk::ImageViewCreateInfo::builder()
                .image(*image)
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(swapchain_format.unwrap().format)
                .components(vk::ComponentMapping {
                    r: vk::ComponentSwizzle::IDENTITY,
                    g: vk::ComponentSwizzle::IDENTITY,
                    b: vk::ComponentSwizzle::IDENTITY,
                    a: vk::ComponentSwizzle::IDENTITY
                })
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1
                })
                .build(),
            None
        ).unwrap()
    }).collect::<Vec<_>>();

    if DEBUG {
        println!("Created Swapchain Image Views");
    }
}

unsafe fn create_command_pool() {
    set_graphics_command_pool(device.as_ref().unwrap().create_command_pool(
        &vk::CommandPoolCreateInfo::builder()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(*get_graphics_queue_index())
            .build(),
        None
    ).unwrap());

    if DEBUG {
        println!("Created Command Pool");
    }
}

unsafe fn create_graphics_pipeline() {
    let vert_shader_bin = include_bytes!("../../shaders/default.vert.spv");
    let frag_shader_bin = include_bytes!("../../shaders/default.frag.spv");

    let vertex_shader_module = device.as_ref().unwrap().create_shader_module(
        &vk::ShaderModuleCreateInfo {
            s_type: vk::StructureType::SHADER_MODULE_CREATE_INFO,
            code_size: vert_shader_bin.len(),
            p_code: vert_shader_bin.as_ptr() as *const u32,
            ..Default::default()
        },
        None
    ).unwrap();

    let fragment_shader_module = device.as_ref().unwrap().create_shader_module(
        &vk::ShaderModuleCreateInfo {
            s_type: vk::StructureType::SHADER_MODULE_CREATE_INFO,
            code_size: frag_shader_bin.len(),
            p_code: frag_shader_bin.as_ptr() as *const u32,
            ..Default::default()
        },
        None
    ).unwrap();

    let entry_point_name = CString::new("main").unwrap();
    let shader_stages = [
        vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::VERTEX)
            .module(vertex_shader_module)
            .name(&entry_point_name)
            .build(),
        vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::FRAGMENT)
            .module(fragment_shader_module)
            .name(&entry_point_name)
            .build()
    ];
    if DEBUG {
        println!("Created Shader Stages");
    }

    let vertex_binding_description = Vertex::get_binding_description();
    let vertex_attribute_descriptions = Vertex::get_attribute_descriptions();

    let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::builder()
        .vertex_binding_descriptions(&vertex_binding_description)
        .vertex_attribute_descriptions(&vertex_attribute_descriptions)
        .build();
    if DEBUG {
        println!("Created Vertex Input State Info");
    }

    let input_assembly_info = vk::PipelineInputAssemblyStateCreateInfo::builder()
        .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
        .primitive_restart_enable(false)
        .build();
    if DEBUG {
        println!("Created Input Assembly State Info");
    }

    let viewport = [vk::Viewport {
        x: 0.0,
        y: 0.0,
        width: extent.unwrap().width as f32,
        height: extent.unwrap().height as f32,
        min_depth: 0.0,
        max_depth: 1.0,
    }];

    let scissor = [vk::Rect2D {
        offset: vk::Offset2D { x: 0, y: 0 },
        extent: extent.unwrap()
    }];

    let viewport_state_info = vk::PipelineViewportStateCreateInfo::builder()
        .viewports(&viewport)
        .scissors(&scissor)
        .build();
    if DEBUG {
        println!("Created Viewport State Info");
    }

    let rasterizer_info = vk::PipelineRasterizationStateCreateInfo::builder()
        .depth_clamp_enable(false)
        .rasterizer_discard_enable(false)
        .polygon_mode(vk::PolygonMode::FILL)
        .line_width(1.0)
        .cull_mode(vk::CullModeFlags::NONE)
        .front_face(vk::FrontFace::CLOCKWISE)
        .depth_bias_enable(false)
        .build();
    if DEBUG {
        println!("Created Rasterizer State Info");
    }

    let multisample_info = vk::PipelineMultisampleStateCreateInfo::builder()
        .sample_shading_enable(false)
        .rasterization_samples(vk::SampleCountFlags::TYPE_1)
        .build();
    if DEBUG {
        println!("Created Multisample State Info");
    }

    let color_blend_attachments = [
        vk::PipelineColorBlendAttachmentState::builder()
            .color_write_mask(vk::ColorComponentFlags::R | vk::ColorComponentFlags::G | vk::ColorComponentFlags::B | vk::ColorComponentFlags::A)
            .blend_enable(true)
            .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
            .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
            .color_blend_op(vk::BlendOp::ADD)
            .src_alpha_blend_factor(vk::BlendFactor::ONE)
            .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
            .alpha_blend_op(vk::BlendOp::ADD)
            .build()
    ];

    let color_blend_info = vk::PipelineColorBlendStateCreateInfo::builder()
        .attachments(&color_blend_attachments)
        .logic_op_enable(true)
        .logic_op(vk::LogicOp::COPY)
        .blend_constants([0.0, 0.0, 0.0, 0.0])
        .build();
    if DEBUG {
        println!("Created Color Blend State Info");
    }

    let camera_binding = vk::DescriptorSetLayoutBinding::builder()
        .binding(0)
        .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
        .descriptor_count(1)
        .stage_flags(vk::ShaderStageFlags::VERTEX)
        .build();

    let texture_binding = vk::DescriptorSetLayoutBinding::builder()
        .binding(1)
        .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
        .descriptor_count(1)
        .stage_flags(vk::ShaderStageFlags::FRAGMENT)
        .build();

    let bindings = [camera_binding, texture_binding];

    let local_descriptor_set_layout = device.as_ref().unwrap().create_descriptor_set_layout(
        &vk::DescriptorSetLayoutCreateInfo::builder()
            .bindings(&bindings)
            .build(),
        None
    ).unwrap();


    let local_pipeline_layout = device.as_ref().unwrap().create_pipeline_layout(
        &vk::PipelineLayoutCreateInfo::builder()
            .set_layouts(&[local_descriptor_set_layout])
            .push_constant_ranges(&[])
            .build(),
        None
    ).unwrap();

    render_pass = {
        let color_attachment = vk::AttachmentDescription::builder()
            .format(swapchain_format.unwrap().format)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::PRESENT_SRC_KHR)
            .build();

        let color_attachment_ref = vk::AttachmentReference::builder()
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .build();

        let color_attachments = [color_attachment_ref];
        let subpass = vk::SubpassDescription::builder()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(&color_attachments)
            .build();

        let subpass_dependency = vk::SubpassDependency::builder()
            .src_subpass(vk::SUBPASS_EXTERNAL)
            .dst_subpass(0)
            .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS)
            .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS)
            .src_access_mask(vk::AccessFlags::empty())
            .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE)
            .build();

        device.as_ref().unwrap().create_render_pass(
            &vk::RenderPassCreateInfo::builder()
                .attachments(&[color_attachment])
                .subpasses(&[subpass])
                .dependencies(&[subpass_dependency])
                .build(),
            None
        ).ok()
    };
    if DEBUG {
        println!("Created Render Pass");
    }
    set_graphics_pipeline(device.as_ref().unwrap().create_graphics_pipelines(vk::PipelineCache::null(), &[
        vk::GraphicsPipelineCreateInfo::builder()
            .stages(&shader_stages)
            .vertex_input_state(&vertex_input_info)
            .input_assembly_state(&input_assembly_info)
            .viewport_state(&viewport_state_info)
            .rasterization_state(&rasterizer_info)
            .multisample_state(&multisample_info)
            .color_blend_state(&color_blend_info)
            .layout(local_pipeline_layout)
            .render_pass(get_render_pass().clone())
            .subpass(0)
            .build()
    ], None).unwrap()[0]);

    set_descriptor_set_layout(local_descriptor_set_layout);
    set_pipeline_layout(local_pipeline_layout);

    if DEBUG {
        println!("Created Graphics Pipeline");
    }
}

unsafe fn create_framebuffers() {
    swapchain_framebuffers = swapchain_image_views.iter().map(|image_view| {
        device.as_ref().unwrap().create_framebuffer(
            &vk::FramebufferCreateInfo::builder()
                .render_pass(get_render_pass().clone())
                .attachments(&[*image_view])
                .width(extent.unwrap().width)
                .height(extent.unwrap().height)
                .layers(1)
                .build(),
            None
        ).unwrap()
    }).collect();

    if DEBUG {
        println!("Created Framebuffers");
    }
}

unsafe fn create_draw_objects() {
    set_draw_command_buffer(device.as_ref().unwrap().allocate_command_buffers(
        &vk::CommandBufferAllocateInfo::builder()
            .command_pool(get_graphics_command_pool().clone())
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1)
            .build(),
    ).unwrap()[0]);

    let semaphore_create_info = vk::SemaphoreCreateInfo::builder().build();

    image_available_semaphore = device.as_ref().unwrap().create_semaphore(&semaphore_create_info, None).ok();
    render_finished_semaphore = device.as_ref().unwrap().create_semaphore(&semaphore_create_info, None).ok();

    in_flight_fence = device.as_ref().unwrap().create_fence(&vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED).build(), None).ok();
}

pub fn begin_single_exec_command() -> vk::CommandBuffer {
    unsafe {
        let command_buffer = device.as_ref().unwrap().allocate_command_buffers(
            &vk::CommandBufferAllocateInfo::builder()
                .command_pool(get_graphics_command_pool().clone())
                .level(vk::CommandBufferLevel::PRIMARY)
                .command_buffer_count(1)
                .build(),
        ).unwrap()[0];

        device.as_ref().unwrap().begin_command_buffer(
            command_buffer,
            &vk::CommandBufferBeginInfo::builder()
                .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT)
                .build(),
        ).unwrap();

        command_buffer
    }
}

pub fn end_single_exec_command(command_buffer: vk::CommandBuffer) {
    unsafe {
        device.as_ref().unwrap().end_command_buffer(command_buffer).unwrap();

        device.as_ref().unwrap().queue_submit(
            get_graphics_queue().clone(),
            &[
                vk::SubmitInfo::builder()
                    .command_buffers(&[command_buffer])
                    .build(),
            ],
            vk::Fence::null()
        ).unwrap();

        device.as_ref().unwrap().queue_wait_idle(get_graphics_queue().clone()).unwrap();

        device.as_ref().unwrap().free_command_buffers(
            get_graphics_command_pool().clone(),
            &[command_buffer],
        );
    }
}

pub fn reset_command_buffer() {
    unsafe {
        device.as_ref().unwrap().wait_for_fences(&[get_in_flight_fence().clone()], true, std::u64::MAX).unwrap();
        device.as_ref().unwrap().reset_fences(&[get_in_flight_fence().clone()]).unwrap();

        device.as_ref().unwrap().reset_command_buffer(get_draw_command_buffer().clone(), vk::CommandBufferResetFlags::empty()).unwrap();
    }
}

pub fn draw(draw_call: DrawCall) {
    unsafe {
        draw_calls.push(draw_call);
    }
}

pub fn render_surface() {
    unsafe {
        image_index = swapchain_util.as_ref().unwrap().acquire_next_image(get_swapchain_khr().clone(), std::u64::MAX, get_image_available_semaphore().clone(), vk::Fence::null()).unwrap().0;

        device.as_ref().unwrap().begin_command_buffer(get_draw_command_buffer().clone(), &vk::CommandBufferBeginInfo::builder().build()).unwrap();

        device.as_ref().unwrap().cmd_begin_render_pass(
            get_draw_command_buffer().clone(),
            &vk::RenderPassBeginInfo::builder()
                .render_pass(get_render_pass().clone())
                .framebuffer(swapchain_framebuffers[image_index as usize])
                .render_area(vk::Rect2D {
                    offset: vk::Offset2D { x: 0, y: 0 },
                    extent: extent.unwrap()
                })
                .clear_values(&[
                    vk::ClearValue {
                        color: vk::ClearColorValue {
                            float32: [0.0, 0.0, 0.0, 1.0]
                        }
                    },
                    vk::ClearValue {
                        depth_stencil: vk::ClearDepthStencilValue {
                            depth: 1.0,
                            stencil: 0
                        }
                    }
                ])
                .build(),
            vk::SubpassContents::INLINE
        );

        device.as_ref().unwrap().cmd_bind_pipeline(get_draw_command_buffer().clone(), vk::PipelineBindPoint::GRAPHICS, get_graphics_pipeline().clone());

        for draw_call in draw_calls.iter() {
            get_device().cmd_bind_descriptor_sets(
                get_draw_command_buffer().clone(),
                vk::PipelineBindPoint::GRAPHICS,
                get_pipeline_layout().clone(),
                0,
                &[draw_call.descriptor_set],
                &[],
            );

            get_device().cmd_bind_vertex_buffers(
                get_draw_command_buffer().clone(),
                0,
                &[draw_call.buffer],
                &[0],
            );

            get_device().cmd_draw(
                get_draw_command_buffer().clone(),
                draw_call.vertex_count,
                1,
                0,
                0
            );
        }
        draw_calls.clear();

        device.as_ref().unwrap().cmd_end_render_pass(get_draw_command_buffer().clone());
        device.as_ref().unwrap().end_command_buffer(get_draw_command_buffer().clone()).unwrap();

        device.as_ref().unwrap().queue_submit(
            get_graphics_queue().clone(),
            &[
                vk::SubmitInfo::builder()
                    .command_buffers(&[get_draw_command_buffer().clone()])
                    .wait_dst_stage_mask(&[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT])
                    .wait_semaphores(&[get_image_available_semaphore().clone()])
                    .signal_semaphores(&[get_render_finished_semaphore().clone()])
                    .build()
            ],
            get_in_flight_fence().clone()
        ).unwrap();

        swapchain_util.as_ref().unwrap().queue_present(
            get_graphics_queue().clone(),
            &vk::PresentInfoKHR::builder()
                .wait_semaphores(&[get_render_finished_semaphore().clone()])
                .swapchains(&[get_swapchain_khr().clone()])
                .image_indices(&[image_index])
                .build()
        ).unwrap();
    }
}

pub fn create_descriptor_pool() -> vk::DescriptorPool {
    unsafe {
        device.as_ref().unwrap().create_descriptor_pool(
            &vk::DescriptorPoolCreateInfo::builder()
                .pool_sizes(&[
                    vk::DescriptorPoolSize {
                        ty: vk::DescriptorType::UNIFORM_BUFFER,
                        descriptor_count: 1
                    },
                    vk::DescriptorPoolSize {
                        ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                        descriptor_count: 1
                    }
                ])
                .max_sets(1)
                .build(),
            None
        ).unwrap()
    }
}
