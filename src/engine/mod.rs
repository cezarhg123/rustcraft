/// global "Singleton" instance
/// 
/// im doing this cuz i dont wanna pass around references to an engine object
/// 
/// this is highly unsafe according to rust but i know what im doing™
/// 
/// if a type doesnt have a `null()` function then i wrap it in an `Option`
#[allow(non_upper_case_globals)]
pub mod instance {
    use std::ffi::CString;

    use ash::vk;

    use crate::WINDOW_TITLE;

    const DEBUG: bool = true;

    static mut entry: Option<ash::Entry> = None;
    static mut instance: Option<ash::Instance> = None;

    static mut gpu: vk::PhysicalDevice = vk::PhysicalDevice::null();
    static mut gpu_memory_properties: Option<vk::PhysicalDeviceMemoryProperties> = None;

    static mut device: Option<ash::Device> = None;
    static mut device_queue: vk::Queue = vk::Queue::null();

    static mut debug_utils: Option<ash::extensions::ext::DebugUtils> = None;
    static mut debug_messenger: vk::DebugUtilsMessengerEXT = vk::DebugUtilsMessengerEXT::null();

    static mut surface_khr: vk::SurfaceKHR = vk::SurfaceKHR::null();
    static mut surface_util: Option<ash::extensions::khr::Surface> = None;

    static mut swapchain_khr: vk::SwapchainKHR = vk::SwapchainKHR::null();
    static mut swapchain_util: Option<ash::extensions::khr::Swapchain> = None;

    static mut swapchain_format: Option<vk::Format> = None;
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

    pub fn init(glfw: &glfw::Glfw) {
        unsafe {
            create_instance(glfw);
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

        let enabled_layer = [
            "VK_LAYER_KHRONOS_validation\0"
        ];
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
}