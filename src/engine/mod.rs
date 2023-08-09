/// global "Singleton" instance
/// 
/// im doing this cuz i dont wanna pass around references to an engine object
/// 
/// this is highly unsafe according to rust but i know what im doing™
/// 
/// if a type doesnt have a `null()` function then i wrap it in an `Option`
#[allow(non_upper_case_globals)]
pub mod instance {
    use ash::vk;

    const DEBUG: bool = true;

    static entry: Option<ash::Entry> = None;
    static instance: Option<ash::Instance> = None;

    static gpu: vk::PhysicalDevice = vk::PhysicalDevice::null();
    static gpu_memory_properties: Option<vk::PhysicalDeviceMemoryProperties> = None;

    static device: Option<ash::Device> = None;
    static device_queue: vk::Queue = vk::Queue::null();

    static debug_utils: Option<ash::extensions::ext::DebugUtils> = None;
    static debug_messenger: vk::DebugUtilsMessengerEXT = vk::DebugUtilsMessengerEXT::null();

    static surface_khr: vk::SurfaceKHR = vk::SurfaceKHR::null();
    static surface_util: Option<ash::extensions::khr::Surface> = None;

    static swapchain_khr: vk::SwapchainKHR = vk::SwapchainKHR::null();
    static swapchain_util: Option<ash::extensions::khr::Swapchain> = None;

    static swapchain_format: Option<vk::Format> = None;
    static swapchain_present_mode: Option<vk::PresentModeKHR> = None;

    static extent: Option<vk::Extent2D> = None;
    static viewport: Option<vk::Viewport> = None;
    static scissor: Option<vk::Rect2D> = None;

    static swapchain_image_views: Vec<vk::ImageView> = Vec::new();
    static swapchain_framebuffers: Vec<vk::Framebuffer> = Vec::new();

    static render_pass: vk::RenderPass = vk::RenderPass::null();

    static pipeline_layout: vk::PipelineLayout = vk::PipelineLayout::null();
    static graphics_pipeline: vk::Pipeline = vk::Pipeline::null();

    static command_pool: vk::CommandPool = vk::CommandPool::null();
    static descriptor_set_layout: vk::DescriptorSetLayout = vk::DescriptorSetLayout::null();

    //drawing
    static draw_command_buffer: vk::CommandBuffer = vk::CommandBuffer::null();
    static image_available_semaphore: vk::Semaphore = vk::Semaphore::null();
    static render_finished_semaphore: vk::Semaphore = vk::Semaphore::null();
    static in_flight_fence: vk::Fence = vk::Fence::null();
    static image_index: u32 = 0;
}