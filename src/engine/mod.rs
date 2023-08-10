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

    const DEBUG: bool = true;

    static mut entry: Option<ash::Entry> = None;
    static mut instance: Option<ash::Instance> = None;
    
    static mut debug_utils: Option<ash::extensions::ext::DebugUtils> = None;
    static mut debug_messenger: vk::DebugUtilsMessengerEXT = vk::DebugUtilsMessengerEXT::null();

    static mut gpu: vk::PhysicalDevice = vk::PhysicalDevice::null();
    static mut gpu_memory_properties: Option<vk::PhysicalDeviceMemoryProperties> = None;

    static mut device: Option<ash::Device> = None;
    static mut graphics_device_queue: vk::Queue = vk::Queue::null();
    static mut transfer_device_queue: vk::Queue = vk::Queue::null();

    static mut surface_khr: vk::SurfaceKHR = vk::SurfaceKHR::null();
    static mut surface_util: Option<ash::extensions::khr::Surface> = None;

    static mut swapchain_khr: vk::SwapchainKHR = vk::SwapchainKHR::null();
    static mut swapchain_util: Option<ash::extensions::khr::Swapchain> = None;

    static mut swapchain_format: Option<vk::SurfaceFormatKHR> = None;
    static mut swapchain_present_mode: Option<vk::PresentModeKHR> = None;

    static mut extent: Option<vk::Extent2D> = None;
    static mut viewport: Option<vk::Viewport> = None;
    static mut scissor: Option<vk::Rect2D> = None;

    static mut swapchain_image_views: Vec<vk::ImageView> = Vec::new();
    static mut swapchain_framebuffers: Vec<vk::Framebuffer> = Vec::new();

    static mut render_pass: vk::RenderPass = vk::RenderPass::null();

    static mut pipeline_layout: vk::PipelineLayout = vk::PipelineLayout::null();
    static mut graphics_pipeline: vk::Pipeline = vk::Pipeline::null();

    static mut command_pool: vk::CommandPool = vk::CommandPool::null();
    static mut descriptor_set_layout: vk::DescriptorSetLayout = vk::DescriptorSetLayout::null();

    //drawing
    static mut draw_command_buffer: vk::CommandBuffer = vk::CommandBuffer::null();
    static mut image_available_semaphore: vk::Semaphore = vk::Semaphore::null();
    static mut render_finished_semaphore: vk::Semaphore = vk::Semaphore::null();
    static mut in_flight_fence: vk::Fence = vk::Fence::null();
    static mut image_index: u32 = 0;

    pub fn init(glfw: &glfw::Glfw, window: &glfw::Window) {
        unsafe {
            create_instance(glfw);
            create_debug_utils();
            choose_physical_device();
            create_logical_device();
            create_surface(window);
            create_swapchain(window);
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
        
        // completely unnecessary but i want to work with 2 queues to learn vulkan
        let transfer_queue_family = queue_families
            .iter()
            .enumerate()
            .find(|(_, p)| {
                p.queue_flags.contains(vk::QueueFlags::TRANSFER)
            })
            .expect("No transfer queue found");

        let mut physical_device_features = instance.as_ref().unwrap().get_physical_device_features(gpu);
        physical_device_features.sampler_anisotropy = 1;

        let device_extensions = [
            "VK_KHR_swapchain\0"
        ];
        let device_extension_ptrs = device_extensions.iter().map(|s| s.as_ptr() as *const i8).collect::<Vec<_>>();

        let mut queue_create_infos = vec![
            vk::DeviceQueueCreateInfo::builder()
                .queue_family_index(graphics_queue_family.0 as u32)
                .queue_priorities(&[1.0])
                .build()
        ];
        
        let mut transfer_queue_index = 0;

        if graphics_queue_family.0 != transfer_queue_family.0 {
            queue_create_infos.push(
                vk::DeviceQueueCreateInfo::builder()
                    .queue_family_index(transfer_queue_family.0 as u32)
                    .queue_priorities(&[1.0])
                    .build()
            );

            transfer_queue_index = 1;
        }

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

        graphics_device_queue = device.as_ref().unwrap().get_device_queue(graphics_queue_family.0 as u32, 0);
        transfer_device_queue = device.as_ref().unwrap().get_device_queue(transfer_queue_family.0 as u32, transfer_queue_index);

        if DEBUG {
            println!("Created Vulkan Logical Device");
        }
    }

    /// currently only support win32
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
    fn find_memory_type(
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
}