pub mod vertex;
pub mod camera;
pub mod buffer;
pub mod texture;

/// global "Singleton" instance
/// 
/// im doing this cuz i dont wanna pass around references to an engine object
/// 
/// this is highly unsafe according to rust but i know what im doing™
/// 
/// if a type doesnt have a `.null()` function then i wrap it in an `Option`
#[allow(non_upper_case_globals)]
pub mod instance {
    use std::{ffi::{CString, CStr}, ptr::null};
    use ash::vk;
    use winapi::um::libloaderapi::GetModuleHandleW;
    use crate::WINDOW_TITLE;

    use super::vertex::Vertex;

    pub const DEBUG: bool = true;

    static mut entry: Option<ash::Entry> = None;
    static mut instance: Option<ash::Instance> = None;
    
    static mut debug_utils: Option<ash::extensions::ext::DebugUtils> = None;
    static mut debug_messenger: vk::DebugUtilsMessengerEXT = vk::DebugUtilsMessengerEXT::null();

    static mut gpu: vk::PhysicalDevice = vk::PhysicalDevice::null();
    static mut gpu_memory_properties: Option<vk::PhysicalDeviceMemoryProperties> = None;

    static mut device: Option<ash::Device> = None;

    static mut graphics_queue_index: u32 = 0;
    static mut graphics_device_queue: vk::Queue = vk::Queue::null();

    static mut surface_khr: vk::SurfaceKHR = vk::SurfaceKHR::null();
    static mut surface_util: Option<ash::extensions::khr::Surface> = None;

    static mut swapchain_khr: vk::SwapchainKHR = vk::SwapchainKHR::null();
    static mut swapchain_util: Option<ash::extensions::khr::Swapchain> = None;

    static mut swapchain_format: Option<vk::SurfaceFormatKHR> = None;
    static mut swapchain_present_mode: Option<vk::PresentModeKHR> = None;

    static mut extent: Option<vk::Extent2D> = None;

    static mut swapchain_image_views: Vec<vk::ImageView> = Vec::new();
    static mut swapchain_framebuffers: Vec<vk::Framebuffer> = Vec::new();
    static mut depth_image_view: vk::ImageView = vk::ImageView::null();

    static mut render_pass: vk::RenderPass = vk::RenderPass::null();

    static mut pipeline_layout: vk::PipelineLayout = vk::PipelineLayout::null();
    static mut graphics_pipeline: vk::Pipeline = vk::Pipeline::null();

    static mut graphics_command_pool: vk::CommandPool = vk::CommandPool::null();
    static mut descriptor_set_layout: vk::DescriptorSetLayout = vk::DescriptorSetLayout::null();

    //drawing
    static mut draw_command_buffer: vk::CommandBuffer = vk::CommandBuffer::null();
    static mut image_available_semaphore: vk::Semaphore = vk::Semaphore::null();
    static mut render_finished_semaphore: vk::Semaphore = vk::Semaphore::null();
    static mut in_flight_fence: vk::Fence = vk::Fence::null();
    static mut image_index: u32 = 0;

    type BufferOffset = u64;
    type Count = u64;
    static mut draw_calls: Vec<(vk::Buffer, BufferOffset, vk::DescriptorSet, Count)> = Vec::new();

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
        }
    }

    unsafe fn create_instance(glfw: &glfw::Glfw) {
        entry = Some(ash::Entry::load().unwrap());

        let c_name = CString::new(WINDOW_TITLE).unwrap();

        let mut supported_extensions = glfw.get_required_instance_extensions().unwrap();
        supported_extensions.push("VK_EXT_debug_utils".to_string());
        // add \0 to the end of every extension name
        let supported_extensions = supported_extensions.iter().map(|s| format!("{s}\0")).collect::<Vec<_>>();
        let supported_extension_ptrs = supported_extensions.iter().map(|s| s.as_ptr() as *const i8).collect::<Vec<_>>(); 
        
        let enabled_layer = if DEBUG {
            [
                "VK_LAYER_KHRONOS_validation\0"
            ]
        } else {
            // should work™
            ["\0"]
        };
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

        instance = Some(entry.as_ref().unwrap().create_instance(&create_info, None).unwrap());
        if DEBUG {
            println!("Created Vulkan Instance");
        }
    }

    unsafe fn create_debug_utils() {
        if DEBUG {
            debug_utils = Some(ash::extensions::ext::DebugUtils::new(entry.as_ref().unwrap(), instance.as_ref().unwrap()));

            debug_messenger = debug_utils.as_ref().unwrap().create_debug_utils_messenger(
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
                None
            ).unwrap();

            if DEBUG {
                println!("Created Vulkan Debug Utils");
            }
        }
    }

    unsafe fn choose_physical_device() {
        gpu = instance.as_ref().unwrap().enumerate_physical_devices()
            .unwrap()
            .into_iter()
            .find(|p| {
                let properties = instance.as_ref().unwrap().get_physical_device_properties(*p);

                properties.device_type == vk::PhysicalDeviceType::DISCRETE_GPU ||
                properties.device_type == vk::PhysicalDeviceType::INTEGRATED_GPU
            })
            .expect("No discrete or integrated GPU found.");

        gpu_memory_properties = Some(instance.as_ref().unwrap().get_physical_device_memory_properties(gpu));

        if DEBUG {
            let gpu_properties = instance.as_ref().unwrap().get_physical_device_properties(gpu);
            
            let name = CStr::from_ptr(gpu_properties.device_name.as_ptr());
            println!("Using GPU: {}", name.to_str().unwrap());
            println!("GPU Type: {:#?}", gpu_properties.device_type);
        }
    }

    unsafe fn create_logical_device() {
        let queue_families = instance.as_ref().unwrap().get_physical_device_queue_family_properties(gpu);
        let graphics_queue_family = queue_families
            .iter()
            .enumerate()
            .find(|(_, p)| {
                p.queue_flags.contains(vk::QueueFlags::GRAPHICS)
            })
            .expect("No graphics queue found");

        let mut physical_device_features = instance.as_ref().unwrap().get_physical_device_features(gpu);
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
                gpu,
                &vk::DeviceCreateInfo::builder()
                    .queue_create_infos(&queue_create_infos)
                    .enabled_extension_names(&device_extension_ptrs)
                    .enabled_features(&physical_device_features)
                    .build(),
                None
            ).unwrap()
        );

        graphics_queue_index = graphics_queue_family.0 as u32;
        graphics_device_queue = device.as_ref().unwrap().get_device_queue(graphics_queue_family.0 as u32, 0);

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
        ).unwrap();

        if window.create_window_surface(instance.as_ref().unwrap().handle(), null(), &mut surface_khr).result().is_err() {
            panic!("Failed to create window surface");
        }

        if DEBUG {
            println!("Created Vulkan Window Surface");
        }
    }

    unsafe fn create_swapchain(window: &glfw::Window) {
        swapchain_util = Some(ash::extensions::khr::Swapchain::new(instance.as_ref().unwrap(), device.as_ref().unwrap()));

        let capabilities = surface_util.as_ref().unwrap().get_physical_device_surface_capabilities(gpu, surface_khr).unwrap();

        swapchain_format = Some(
            *surface_util.as_ref().unwrap().get_physical_device_surface_formats(gpu, surface_khr).unwrap()
                .iter()
                .find(|f| f.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR && f.format == vk::Format::B8G8R8A8_SRGB)
                .unwrap()
        );

        swapchain_present_mode = Some(
            *surface_util.as_ref().unwrap().get_physical_device_surface_present_modes(gpu, surface_khr).unwrap().iter().find(|p| **p == vk::PresentModeKHR::IMMEDIATE).unwrap()
        );

        let framebuffer_size = window.get_framebuffer_size();

        extent = Some(vk::Extent2D {
            width: (framebuffer_size.0 as u32).clamp(capabilities.min_image_extent.width, capabilities.max_image_extent.width),
            height: (framebuffer_size.1 as u32).clamp(capabilities.min_image_extent.height, capabilities.max_image_extent.height),
        });

        swapchain_khr = swapchain_util.as_ref().unwrap().create_swapchain(
            &vk::SwapchainCreateInfoKHR::builder()
                .surface(surface_khr)
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
        ).unwrap();

        if DEBUG {
            println!("Created Swapchain");
        }

        let images = swapchain_util.as_ref().unwrap().get_swapchain_images(swapchain_khr).unwrap();

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
        graphics_command_pool = device.as_ref().unwrap().create_command_pool(
            &vk::CommandPoolCreateInfo::builder()
                .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
                .queue_family_index(graphics_queue_index)
                .build(),
            None
        ).unwrap();

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
            .cull_mode(vk::CullModeFlags::BACK)
            .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
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
                .blend_enable(false)
                .build()
        ];

        let color_blend_info = vk::PipelineColorBlendStateCreateInfo::builder()
            .attachments(&color_blend_attachments)
            .logic_op_enable(false)
            .build();
        if DEBUG {
            println!("Created Color Blend State Info");
        }

        let camera_descriptor_binding = vk::DescriptorSetLayoutBinding::builder()
            .binding(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::VERTEX)
            .build();

        let texture_atlas_descriptor_binding = vk::DescriptorSetLayoutBinding::builder()
            .binding(1)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT)
            .build();

        let model_descriptor_binding = vk::DescriptorSetLayoutBinding::builder()
            .binding(2)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::VERTEX)
            .build();

        descriptor_set_layout = device.as_ref().unwrap().create_descriptor_set_layout(
            &vk::DescriptorSetLayoutCreateInfo::builder()
                .bindings(&[
                    camera_descriptor_binding,
                    texture_atlas_descriptor_binding,
                    model_descriptor_binding
                ])
                .build(),
            None
        ).unwrap();

        pipeline_layout = device.as_ref().unwrap().create_pipeline_layout(
            &vk::PipelineLayoutCreateInfo::builder()
                .set_layouts(&[descriptor_set_layout])
                .push_constant_ranges(&[])
                .build(),
            None
        ).unwrap();

        let depth_format = {
            let wanted_formats = [vk::Format::D24_UNORM_S8_UINT, vk::Format::D32_SFLOAT_S8_UINT];

            let mut selected_format = None;

            for format in wanted_formats {
                let props = instance.as_ref().unwrap().get_physical_device_format_properties(gpu, format);

                if props.optimal_tiling_features.contains(vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT) {
                    selected_format = Some(format);
                }
            }

            selected_format.unwrap()
        };

        let depth_image = device.as_ref().unwrap().create_image(
            &vk::ImageCreateInfo::builder()
                .image_type(vk::ImageType::TYPE_2D)
                .extent(vk::Extent3D {
                    width: extent.unwrap().width,
                    height: extent.unwrap().height,
                    depth: 1,
                })
                .mip_levels(1)
                .array_layers(1)
                .format(depth_format)
                .samples(vk::SampleCountFlags::TYPE_1)
                .tiling(vk::ImageTiling::OPTIMAL)
                .usage(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT)
                .sharing_mode(vk::SharingMode::EXCLUSIVE)
                .initial_layout(vk::ImageLayout::UNDEFINED)
                .build(),
            None
        ).unwrap();

        let depth_image_memory = {
            let memory_requirements = device.as_ref().unwrap().get_image_memory_requirements(depth_image);

            device.as_ref().unwrap().allocate_memory(
                &vk::MemoryAllocateInfo::builder()
                    .allocation_size(memory_requirements.size)
                    .memory_type_index(find_memory_type(gpu_memory_properties.unwrap(), memory_requirements.memory_type_bits, vk::MemoryPropertyFlags::DEVICE_LOCAL).unwrap())
                    .build(),
                None
            ).unwrap()
        };
        device.as_ref().unwrap().bind_image_memory(depth_image, depth_image_memory, 0).unwrap();

        depth_image_view = device.as_ref().unwrap().create_image_view(
            &vk::ImageViewCreateInfo::builder()
                .image(depth_image)
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(depth_format)
                .components(vk::ComponentMapping {
                    r: vk::ComponentSwizzle::IDENTITY,
                    g: vk::ComponentSwizzle::IDENTITY,
                    b: vk::ComponentSwizzle::IDENTITY,
                    a: vk::ComponentSwizzle::IDENTITY
                })
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::DEPTH,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1
                })
                .build(),
            None
        ).unwrap();

        let transition_command_buffer = begin_single_exec_command();
        transition_image_layout(depth_image, vk::ImageLayout::UNDEFINED, vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);
        end_single_exec_command(transition_command_buffer);

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

            let depth_attachment = vk::AttachmentDescription::builder()
                .format(depth_format)
                .samples(vk::SampleCountFlags::TYPE_1)
                .load_op(vk::AttachmentLoadOp::CLEAR)
                .store_op(vk::AttachmentStoreOp::DONT_CARE)
                .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
                .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
                .initial_layout(vk::ImageLayout::UNDEFINED)
                .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
                .build();

            let color_attachment_ref = vk::AttachmentReference::builder()
                .attachment(0)
                .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                .build();

            let depth_attachment_ref = vk::AttachmentReference::builder()
                .attachment(1)
                .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
                .build();

            let color_attachments = [color_attachment_ref];
            let subpass = vk::SubpassDescription::builder()
                .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
                .color_attachments(&color_attachments)
                .depth_stencil_attachment(&depth_attachment_ref)
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
                    .attachments(&[color_attachment, depth_attachment])
                    .subpasses(&[subpass])
                    .dependencies(&[subpass_dependency])
                    .build(),
                None
            ).unwrap()
        };
        if DEBUG {
            println!("Created Render Pass");
        }

        let depth_stencil_info = vk::PipelineDepthStencilStateCreateInfo::builder()
            .depth_test_enable(true)
            .depth_write_enable(true)
            .depth_compare_op(vk::CompareOp::LESS)
            .depth_bounds_test_enable(false)
            .stencil_test_enable(false)
            .build();
        if DEBUG {
            println!("Created Depth Stencil Info");
        }

        graphics_pipeline = device.as_ref().unwrap().create_graphics_pipelines(vk::PipelineCache::null(), &[
            vk::GraphicsPipelineCreateInfo::builder()
                .stages(&shader_stages)
                .vertex_input_state(&vertex_input_info)
                .input_assembly_state(&input_assembly_info)
                .viewport_state(&viewport_state_info)
                .rasterization_state(&rasterizer_info)
                .multisample_state(&multisample_info)
                .depth_stencil_state(&depth_stencil_info)
                .color_blend_state(&color_blend_info)
                .layout(pipeline_layout)
                .render_pass(render_pass)
                .subpass(0)
                .build()
        ], None).unwrap()[0];

        if DEBUG {
            println!("Created Graphics Pipeline");
        }
    }

    unsafe fn create_framebuffers() {
        swapchain_framebuffers = swapchain_image_views.iter().map(|image_view| {
            device.as_ref().unwrap().create_framebuffer(
                &vk::FramebufferCreateInfo::builder()
                    .render_pass(render_pass)
                    .attachments(&[*image_view, depth_image_view])
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
        draw_command_buffer = device.as_ref().unwrap().allocate_command_buffers(
            &vk::CommandBufferAllocateInfo::builder()
                .command_pool(graphics_command_pool)
                .level(vk::CommandBufferLevel::PRIMARY)
                .command_buffer_count(1)
                .build(),
        ).unwrap()[0];

        let semaphore_create_info = vk::SemaphoreCreateInfo::builder().build();

        image_available_semaphore = device.as_ref().unwrap().create_semaphore(&semaphore_create_info, None).unwrap();
        render_finished_semaphore = device.as_ref().unwrap().create_semaphore(&semaphore_create_info, None).unwrap();

        in_flight_fence = device.as_ref().unwrap().create_fence(&vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED).build(), None).unwrap();
    }

    /// the command buffer returned will be submitted to a transfer queue
    pub fn begin_single_exec_command() -> vk::CommandBuffer {
        unsafe {
            let command_buffer = device.as_ref().unwrap().allocate_command_buffers(
                &vk::CommandBufferAllocateInfo::builder()
                    .command_pool(graphics_command_pool)
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
                graphics_device_queue,
                &[
                    vk::SubmitInfo::builder()
                        .command_buffers(&[command_buffer])
                        .build(),
                ],
                vk::Fence::null()
            ).unwrap();

            device.as_ref().unwrap().queue_wait_idle(graphics_device_queue).unwrap();

            device.as_ref().unwrap().free_command_buffers(
                graphics_command_pool,
                &[command_buffer],
            );
        }
    }

    /// pushes a draw "command" to a vector
    /// 
    /// note: writing of descriptor sets is not handled by engine
    pub fn draw(vertex_buffer: vk::Buffer, offset: u64, descriptor_set: vk::DescriptorSet, vertex_count: u64) {
        unsafe {
            draw_calls.push((vertex_buffer, offset, descriptor_set, vertex_count));
        }
    }


    /// note: again i repeat, writing of descriptor sets is not handled by engine
    pub fn render_surface() {
        unsafe {
            device.as_ref().unwrap().wait_for_fences(&[in_flight_fence], true, std::u64::MAX).unwrap();
            device.as_ref().unwrap().reset_fences(&[in_flight_fence]).unwrap();

            image_index = swapchain_util.as_ref().unwrap().acquire_next_image(swapchain_khr, std::u64::MAX, image_available_semaphore, vk::Fence::null()).unwrap().0;

            device.as_ref().unwrap().begin_command_buffer(draw_command_buffer, &vk::CommandBufferBeginInfo::builder().build()).unwrap();

            device.as_ref().unwrap().cmd_begin_render_pass(
                draw_command_buffer,
                &vk::RenderPassBeginInfo::builder()
                    .render_pass(render_pass)
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

            device.as_ref().unwrap().cmd_bind_pipeline(draw_command_buffer, vk::PipelineBindPoint::GRAPHICS, graphics_pipeline);

            for call in draw_calls.iter() {
                device.as_ref().unwrap().cmd_bind_descriptor_sets(
                    draw_command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    pipeline_layout,
                    0,
                    &[call.2],
                    &[]
                );

                device.as_ref().unwrap().cmd_bind_vertex_buffers(
                    draw_command_buffer,
                    0,
                    &[call.0],
                    &[call.1]
                );

                device.as_ref().unwrap().cmd_draw(
                    draw_command_buffer,
                    call.3 as u32,
                    1,
                    0,
                    0
                );
            }
            draw_calls.clear();

            device.as_ref().unwrap().cmd_end_render_pass(draw_command_buffer);
            device.as_ref().unwrap().end_command_buffer(draw_command_buffer).unwrap();

            device.as_ref().unwrap().queue_submit(
                graphics_device_queue,
                &[
                    vk::SubmitInfo::builder()
                        .command_buffers(&[draw_command_buffer])
                        .wait_dst_stage_mask(&[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT])
                        .wait_semaphores(&[image_available_semaphore])
                        .signal_semaphores(&[render_finished_semaphore])
                        .build()
                ],
                in_flight_fence
            ).unwrap();

            swapchain_util.as_ref().unwrap().queue_present(
                graphics_device_queue,
                &vk::PresentInfoKHR::builder()
                    .wait_semaphores(&[render_finished_semaphore])
                    .swapchains(&[swapchain_khr])
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
                        },
                        vk::DescriptorPoolSize {
                            ty: vk::DescriptorType::UNIFORM_BUFFER,
                            descriptor_count: 1
                        }
                    ])
                    .max_sets(1)
                    .build(),
                None
            ).unwrap()
        }
    }

    pub fn get_device() -> ash::Device {
        unsafe {
            device.clone().unwrap()
        }
    }

    pub fn get_physical_memory_properties() -> vk::PhysicalDeviceMemoryProperties {
        unsafe {
            gpu_memory_properties.unwrap()
        }
    }

    pub fn get_descriptor_set_layout() -> vk::DescriptorSetLayout {
        unsafe {
            descriptor_set_layout
        }
    }

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
            device.as_ref().unwrap().cmd_pipeline_barrier(
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
}