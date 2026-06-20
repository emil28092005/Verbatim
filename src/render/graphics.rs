use ash::vk;
use std::ffi::CString;
use std::sync::Arc;

use crate::entity::{EntityManager, EntityKind};
use crate::world::cell::MaterialId;
use crate::world::grid::Grid;
use crate::world::material::MaterialRegistry;

const CHAR_W: u32 = 16;
const CHAR_H: u32 = 16;
const MAX_FRAMES: usize = 2;

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct ColorInstance {
    grid_x: f32,
    grid_y: f32,
    color: [u8; 4],
}

#[repr(C)]
#[derive(bytemuck::NoUninit, Clone, Copy)]
struct PushConstants {
    screen_size: [f32; 2],
    cell_size: [f32; 2],
}

pub struct GraphicsRenderer {
    grid_w: usize,
    grid_h: usize,

    entry: ash::Entry,
    instance: ash::Instance,
    surface: vk::SurfaceKHR,
    device: ash::Device,
    graphics_queue: vk::Queue,

    swapchain_loader: ash::khr::swapchain::Device,
    swapchain: vk::SwapchainKHR,
    swapchain_image_views: Vec<vk::ImageView>,
    swapchain_extent: vk::Extent2D,

    render_pass: vk::RenderPass,
    pipeline: vk::Pipeline,
    pipeline_layout: vk::PipelineLayout,
    framebuffers: Vec<vk::Framebuffer>,

    command_pool: vk::CommandPool,
    command_buffers: Vec<vk::CommandBuffer>,

    image_available: Vec<vk::Semaphore>,
    render_finished: Vec<vk::Semaphore>,
    in_flight: Vec<vk::Fence>,
    frame_index: usize,

    vertex_buffer: vk::Buffer,
    vertex_memory: vk::DeviceMemory,
    index_buffer: vk::Buffer,
    index_memory: vk::DeviceMemory,

    instance_buffer: vk::Buffer,
    instance_memory: vk::DeviceMemory,
    instance_ptr: *mut ColorInstance,
    instance_count: usize,

    _window: Arc<winit::window::Window>,
}

impl GraphicsRenderer {
    pub fn new(window: Arc<winit::window::Window>) -> Result<Self, String> {
    let grid_w = 160usize;
    let grid_h = 50usize;

    let entry = unsafe { ash::Entry::load().map_err(|e| format!("Vulkan load: {e}"))? };

    use raw_window_handle::HasDisplayHandle;
    let dh = window.display_handle().map_err(|e| format!("display_handle: {e}"))?;
    let required_exts = ash_window::enumerate_required_extensions(dh.as_raw())
        .map_err(|e| format!("enumerate_required_extensions: {e:?}"))?;

    let app_name = CString::new("Verbatim").unwrap();
    let app_info = vk::ApplicationInfo::default()
        .application_name(&app_name)
        .api_version(vk::API_VERSION_1_2);
    let mut ext_ptrs: Vec<*const i8> = required_exts.iter().map(|&p| p as *const i8).collect();
    let avail_exts = unsafe { entry.enumerate_instance_extension_properties(None) }.unwrap_or_default();
    let has_debug = avail_exts.iter().any(|e| {
        let name = unsafe { std::ffi::CStr::from_ptr(e.extension_name.as_ptr() as *const i8) };
        name.to_str().unwrap_or("") == "VK_EXT_debug_utils"
    });
    if has_debug { ext_ptrs.push(b"VK_EXT_debug_utils\0".as_ptr() as *const i8); }
    let ci = vk::InstanceCreateInfo::default()
        .application_info(&app_info).enabled_extension_names(&ext_ptrs);
    let instance = unsafe { entry.create_instance(&ci, None) }.map_err(|e| format!("instance: {e:?}"))?;

    let surface = {
        use raw_window_handle::HasWindowHandle;
        let wh = window.window_handle().map_err(|e| format!("wh: {e}"))?;
        let dh = window.display_handle().map_err(|e| format!("dh: {e}"))?;
        unsafe { ash_window::create_surface(&entry, &instance, dh.as_raw(), wh.as_raw(), None) }
            .map_err(|e| format!("surface: {e:?}"))?
    };

    let sl = ash::khr::surface::Instance::new(&entry, &instance);
    let devices = unsafe { instance.enumerate_physical_devices() }.map_err(|e| format!("enum: {e:?}"))?;
    let mut physical_device = vk::PhysicalDevice::null();
    let mut qf = 0u32;
    for &pd in &devices {
        let props = unsafe { instance.get_physical_device_properties(pd) };
        if props.device_type == vk::PhysicalDeviceType::CPU { continue; }
        let qfs = unsafe { instance.get_physical_device_queue_family_properties(pd) };
        for (i, q) in qfs.iter().enumerate() {
            if q.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                let ok = unsafe { sl.get_physical_device_surface_support(pd, i as u32, surface) }.unwrap_or(false);
                if ok { physical_device = pd; qf = i as u32; break; }
            }
        }
        if physical_device != vk::PhysicalDevice::null() { break; }
    }
    if physical_device == vk::PhysicalDevice::null() { return Err("No GPU".to_string()); }

    let qp = [1.0f32];
    let qi = vk::DeviceQueueCreateInfo::default().queue_family_index(qf).queue_priorities(&qp);
    let dev_ext_names: Vec<CString> = vec![CString::new("VK_KHR_swapchain").unwrap()];
    let dev_ext_ptrs: Vec<*const i8> = dev_ext_names.iter().map(|n| n.as_ptr()).collect();
    let dci = vk::DeviceCreateInfo::default()
        .queue_create_infos(std::slice::from_ref(&qi))
        .enabled_extension_names(&dev_ext_ptrs);
    let device = unsafe { instance.create_device(physical_device, &dci, None) }.map_err(|e| format!("device: {e:?}"))?;
    let graphics_queue = unsafe { device.get_device_queue(qf, 0) };

    let swapchain_loader = ash::khr::swapchain::Device::new(&instance, &device);
    let caps = unsafe { sl.get_physical_device_surface_capabilities(physical_device, surface) }.map_err(|e| format!("caps: {e:?}"))?;
    let format = vk::SurfaceFormatKHR { format: vk::Format::B8G8R8A8_UNORM, color_space: vk::ColorSpaceKHR::SRGB_NONLINEAR };
    let extent = if caps.current_extent.width != u32::MAX { caps.current_extent }
        else { vk::Extent2D { width: grid_w as u32 * CHAR_W, height: grid_h as u32 * CHAR_H } };
    let ic = caps.min_image_count.max(2);
    let qf_slice = [qf];
    let sci = vk::SwapchainCreateInfoKHR::default()
        .surface(surface).min_image_count(ic)
        .image_format(format.format).image_color_space(format.color_space)
        .image_extent(extent).image_array_layers(1)
        .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
        .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
        .queue_family_indices(&qf_slice)
        .pre_transform(caps.current_transform)
        .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
        .present_mode(vk::PresentModeKHR::FIFO).clipped(true);
    let swapchain = unsafe { swapchain_loader.create_swapchain(&sci, None) }.map_err(|e| format!("swapchain: {e:?}"))?;
    let swapchain_images = unsafe { swapchain_loader.get_swapchain_images(swapchain) }.map_err(|e| format!("images: {e:?}"))?;
    let swapchain_image_views: Vec<_> = swapchain_images.iter()
        .map(|&img| {
            let vi = vk::ImageViewCreateInfo::default()
                .image(img).view_type(vk::ImageViewType::TYPE_2D).format(format.format)
                .subresource_range(vk::ImageSubresourceRange { aspect_mask: vk::ImageAspectFlags::COLOR, base_mip_level: 0, level_count: 1, base_array_layer: 0, layer_count: 1 });
            unsafe { device.create_image_view(&vi, None).expect("image_view") }
        }).collect();

    let att = vk::AttachmentDescription::default()
        .format(format.format).samples(vk::SampleCountFlags::TYPE_1)
        .load_op(vk::AttachmentLoadOp::CLEAR).store_op(vk::AttachmentStoreOp::STORE)
        .initial_layout(vk::ImageLayout::UNDEFINED).final_layout(vk::ImageLayout::PRESENT_SRC_KHR);
    let att_ref = vk::AttachmentReference::default().attachment(0).layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);
    let subpass = vk::SubpassDescription::default()
        .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
        .color_attachments(std::slice::from_ref(&att_ref));
    let dep = vk::SubpassDependency::default()
        .src_subpass(vk::SUBPASS_EXTERNAL).dst_subpass(0)
        .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
        .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
        .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE);
    let rpci = vk::RenderPassCreateInfo::default()
        .attachments(std::slice::from_ref(&att))
        .subpasses(std::slice::from_ref(&subpass))
        .dependencies(std::slice::from_ref(&dep));
    let render_pass = unsafe { device.create_render_pass(&rpci, None) }.map_err(|e| format!("render_pass: {e:?}"))?;

    // Pipeline — no atlas, no descriptor set, just colored quads
    let vert_spv = include_bytes!("../../assets/shaders/graphics_vert.spv");
    let frag_spv = include_bytes!("../../assets/shaders/graphics_frag.spv");
    let vert_code: Vec<u32> = vert_spv.chunks_exact(4).map(|c| u32::from_ne_bytes([c[0], c[1], c[2], c[3]])).collect();
    let frag_code: Vec<u32> = frag_spv.chunks_exact(4).map(|c| u32::from_ne_bytes([c[0], c[1], c[2], c[3]])).collect();
    let vm = unsafe { device.create_shader_module(&vk::ShaderModuleCreateInfo::default().code(&vert_code), None) }.map_err(|e| format!("vert: {e:?}"))?;
    let fm = unsafe { device.create_shader_module(&vk::ShaderModuleCreateInfo::default().code(&frag_code), None) }.map_err(|e| format!("frag: {e:?}"))?;
    let main = CString::new("main").unwrap();
    let vs = vk::PipelineShaderStageCreateInfo::default().stage(vk::ShaderStageFlags::VERTEX).module(vm).name(&main);
    let fs = vk::PipelineShaderStageCreateInfo::default().stage(vk::ShaderStageFlags::FRAGMENT).module(fm).name(&main);

    let bindings = [
        vk::VertexInputBindingDescription { binding: 0, stride: 8, input_rate: vk::VertexInputRate::VERTEX },
        vk::VertexInputBindingDescription { binding: 1, stride: std::mem::size_of::<ColorInstance>() as u32, input_rate: vk::VertexInputRate::INSTANCE },
    ];
    let attrs = [
        vk::VertexInputAttributeDescription { location: 0, binding: 0, format: vk::Format::R32G32_SFLOAT, offset: 0 },
        vk::VertexInputAttributeDescription { location: 1, binding: 1, format: vk::Format::R32G32_SFLOAT, offset: 0 },
        vk::VertexInputAttributeDescription { location: 2, binding: 1, format: vk::Format::R8G8B8A8_UNORM, offset: 8 },
    ];
    let vi = vk::PipelineVertexInputStateCreateInfo::default()
        .vertex_binding_descriptions(&bindings).vertex_attribute_descriptions(&attrs);
    let ia = vk::PipelineInputAssemblyStateCreateInfo::default().topology(vk::PrimitiveTopology::TRIANGLE_LIST);
    let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
    let dynamic_state = vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_states);
    let vs_state = vk::PipelineViewportStateCreateInfo::default().viewport_count(1).scissor_count(1);
    let rs = vk::PipelineRasterizationStateCreateInfo::default().line_width(1.0).cull_mode(vk::CullModeFlags::NONE);
    let ms = vk::PipelineMultisampleStateCreateInfo::default().rasterization_samples(vk::SampleCountFlags::TYPE_1);
    let cba = vk::PipelineColorBlendAttachmentState::default().color_write_mask(vk::ColorComponentFlags::RGBA);
    let cb = vk::PipelineColorBlendStateCreateInfo::default().attachments(std::slice::from_ref(&cba));
    let pcr = vk::PushConstantRange { stage_flags: vk::ShaderStageFlags::VERTEX, offset: 0, size: std::mem::size_of::<PushConstants>() as u32 };
    let pli = vk::PipelineLayoutCreateInfo::default().push_constant_ranges(std::slice::from_ref(&pcr));
    let pipeline_layout = unsafe { device.create_pipeline_layout(&pli, None) }.map_err(|e| format!("pipeline_layout: {e:?}"))?;
    let stages = [vs, fs];
    let gpci = vk::GraphicsPipelineCreateInfo::default()
        .stages(&stages).vertex_input_state(&vi).input_assembly_state(&ia)
        .viewport_state(&vs_state).rasterization_state(&rs).multisample_state(&ms)
        .color_blend_state(&cb).dynamic_state(&dynamic_state)
        .layout(pipeline_layout).render_pass(render_pass).subpass(0);
    let pipes = unsafe { device.create_graphics_pipelines(vk::PipelineCache::null(), std::slice::from_ref(&gpci), None) }
        .map_err(|(_, e)| format!("pipeline: {e:?}"))?;
    unsafe { device.destroy_shader_module(vm, None); device.destroy_shader_module(fm, None); }
    let pipeline = pipes[0];

    let framebuffers: Vec<_> = swapchain_image_views.iter()
        .map(|&view| {
            let fci = vk::FramebufferCreateInfo::default()
                .render_pass(render_pass).attachments(std::slice::from_ref(&view))
                .width(extent.width).height(extent.height).layers(1);
            unsafe { device.create_framebuffer(&fci, None).expect("fb") }
        }).collect();

    let command_pool = {
        let cpci = vk::CommandPoolCreateInfo::default().queue_family_index(qf).flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);
        unsafe { device.create_command_pool(&cpci, None) }.map_err(|e| format!("cmd_pool: {e:?}"))?
    };
    let command_buffers = {
        let cai = vk::CommandBufferAllocateInfo::default().command_pool(command_pool).level(vk::CommandBufferLevel::PRIMARY).command_buffer_count(framebuffers.len() as u32);
        unsafe { device.allocate_command_buffers(&cai) }.map_err(|e| format!("cmd_bufs: {e:?}"))?
    };

    let mut image_available = Vec::new();
    let mut render_finished = Vec::new();
    let mut in_flight = Vec::new();
    for _ in 0..MAX_FRAMES {
        unsafe {
            image_available.push(device.create_semaphore(&vk::SemaphoreCreateInfo::default(), None).map_err(|e| format!("sem: {e:?}"))?);
            render_finished.push(device.create_semaphore(&vk::SemaphoreCreateInfo::default(), None).map_err(|e| format!("sem: {e:?}"))?);
            in_flight.push(device.create_fence(&vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED), None).map_err(|e| format!("fence: {e:?}"))?);
        }
    }

    // Vertex + index buffers
    let find_mem = |filter: u32, props: vk::MemoryPropertyFlags| -> Result<u32, String> {
        let mp = unsafe { instance.get_physical_device_memory_properties(physical_device) };
        for (i, mt) in mp.memory_types.iter().enumerate() {
            if (filter & (1 << i)) != 0 && mt.property_flags.contains(props) { return Ok(i as u32); }
        }
        Err("No memory type".to_string())
    };
    let make_buf = |data: &[u8], usage: vk::BufferUsageFlags| -> Result<(vk::Buffer, vk::DeviceMemory), String> {
        let sz = data.len() as vk::DeviceSize;
        let bi = vk::BufferCreateInfo::default().size(sz).usage(usage).sharing_mode(vk::SharingMode::EXCLUSIVE);
        let buf = unsafe { device.create_buffer(&bi, None) }.map_err(|e| format!("buf: {e:?}"))?;
        let req = unsafe { device.get_buffer_memory_requirements(buf) };
        let mt = find_mem(req.memory_type_bits, vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT)?;
        let mem = unsafe { device.allocate_memory(&vk::MemoryAllocateInfo::default().allocation_size(req.size).memory_type_index(mt), None) }.map_err(|e| format!("mem: {e:?}"))?;
        unsafe {
            device.bind_buffer_memory(buf, mem, 0).expect("bind");
            let ptr = device.map_memory(mem, 0, sz, vk::MemoryMapFlags::default()).expect("map");
            std::ptr::copy_nonoverlapping(data.as_ptr(), ptr as *mut u8, data.len());
            device.unmap_memory(mem);
        }
        Ok((buf, mem))
    };
    let verts: [f32; 8] = [0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, 1.0];
    let indices: [u16; 6] = [0, 1, 2, 1, 3, 2];
    let (vertex_buffer, vertex_memory) = make_buf(bytemuck::cast_slice(&verts), vk::BufferUsageFlags::VERTEX_BUFFER)?;
    let (index_buffer, index_memory) = make_buf(bytemuck::cast_slice(&indices), vk::BufferUsageFlags::INDEX_BUFFER)?;

    // Instance buffer
    let instance_count = grid_w * grid_h;
    let inst_sz = (instance_count * std::mem::size_of::<ColorInstance>()) as vk::DeviceSize;
    let ibi = vk::BufferCreateInfo::default().size(inst_sz).usage(vk::BufferUsageFlags::VERTEX_BUFFER).sharing_mode(vk::SharingMode::EXCLUSIVE);
    let instance_buffer = unsafe { device.create_buffer(&ibi, None) }.map_err(|e| format!("inst buf: {e:?}"))?;
    let ireq = unsafe { device.get_buffer_memory_requirements(instance_buffer) };
    let imt = find_mem(ireq.memory_type_bits, vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT)?;
    let instance_memory = unsafe { device.allocate_memory(&vk::MemoryAllocateInfo::default().allocation_size(ireq.size).memory_type_index(imt), None) }.map_err(|e| format!("inst mem: {e:?}"))?;
    let instance_ptr = unsafe {
        device.bind_buffer_memory(instance_buffer, instance_memory, 0).expect("bind inst");
        let ptr = device.map_memory(instance_memory, 0, inst_sz, vk::MemoryMapFlags::default()).expect("map inst");
        ptr as *mut ColorInstance
    };

    Ok(Self {
        grid_w, grid_h,
        entry, instance, surface, device, graphics_queue,
        swapchain_loader, swapchain, swapchain_image_views, swapchain_extent: extent,
        render_pass, pipeline, pipeline_layout, framebuffers,
        command_pool, command_buffers,
        image_available, render_finished, in_flight, frame_index: 0,
        vertex_buffer, vertex_memory, index_buffer, index_memory,
        instance_buffer, instance_memory, instance_ptr, instance_count,
        _window: window,
    })
}

    pub fn render(&mut self, grid: &Grid, entities: &EntityManager, cam_x: i32, cam_y: i32) {
        let reg = MaterialRegistry::instance();

        let mut entity_map: std::collections::HashMap<(i32, i32), [u8; 4]> = std::collections::HashMap::new();
        for e in entities.all() {
            for b in &e.bodies {
                if !b.alive { continue; }
                let sx = b.x as i32 - cam_x;
                let sy = b.y as i32 - cam_y;
                if sx >= 0 && sx < self.grid_w as i32 && sy >= 0 && sy < self.grid_h as i32 {
                    let fg = if e.on_fire { [255, 160, 40, 255] }
                    else if !e.alive { [100, 60, 60, 255] }
                    else {
                        match e.kind {
                            EntityKind::Player => [255, 255, 100, 255],
                            EntityKind::Goblin => [100, 220, 100, 255],
                            _ => [180, 50, 50, 255],
                        }
                    };
                    entity_map.insert((sx, sy), fg);
                }
            }
        }

        let instances = unsafe { std::slice::from_raw_parts_mut(self.instance_ptr, self.instance_count) };
        let bg_default = [10u8, 10, 15, 255];

        for dy in 0..self.grid_h {
            for dx in 0..self.grid_w {
                let idx = dy * self.grid_w + dx;
                let wx = cam_x + dx as i32;
                let wy = cam_y + dy as i32;

                let color = if let Some(&ec) = entity_map.get(&(dx as i32, dy as i32)) {
                    ec
                } else if !grid.in_bounds(wx, wy) {
                    [40, 40, 40, 255]
                } else {
                    let cell = grid.get(wx, wy);
                    let mat = reg.get(cell.material);
                    if cell.is_empty() {
                        bg_default
                    } else {
                        if cell.material == MaterialId::Lava {
                            let r = 200u8.saturating_add(cell.variant / 2);
                            [r, 60, 20, 255]
                        } else {
                            [mat.color_fg.0, mat.color_fg.1, mat.color_fg.2, 255]
                        }
                    }
                };

                instances[idx] = ColorInstance {
                    grid_x: dx as f32,
                    grid_y: dy as f32,
                    color,
                };
            }
        }

        let frame = self.frame_index;
        let device = &self.device;

        unsafe {
            let _ = device.wait_for_fences(&[self.in_flight[frame]], true, u64::MAX);
            let _ = device.reset_fences(&[self.in_flight[frame]]);

            let image_index = match self.swapchain_loader.acquire_next_image(
                self.swapchain, u64::MAX, self.image_available[frame], vk::Fence::null()
            ) {
                Ok((idx, _)) => idx as usize,
                Err(e) => { eprintln!("acquire: {e:?}"); return; }
            };

            let cmd = self.command_buffers[frame];
            let _ = device.reset_command_buffer(cmd, vk::CommandBufferResetFlags::default());
            let _ = device.begin_command_buffer(cmd, &vk::CommandBufferBeginInfo::default());

            let clear = vk::ClearValue {
                color: vk::ClearColorValue { float32: [10.0/255.0, 10.0/255.0, 15.0/255.0, 1.0] },
            };
            let rp_info = vk::RenderPassBeginInfo::default()
                .render_pass(self.render_pass)
                .framebuffer(self.framebuffers[image_index])
                .render_area(vk::Rect2D { offset: vk::Offset2D::default(), extent: self.swapchain_extent })
                .clear_values(std::slice::from_ref(&clear));

            device.cmd_begin_render_pass(cmd, &rp_info, vk::SubpassContents::INLINE);
            device.cmd_bind_pipeline(cmd, vk::PipelineBindPoint::GRAPHICS, self.pipeline);

            let viewport = vk::Viewport {
                x: 0.0, y: 0.0,
                width: self.swapchain_extent.width as f32,
                height: self.swapchain_extent.height as f32,
                min_depth: 0.0, max_depth: 1.0,
            };
            let scissor = vk::Rect2D { offset: vk::Offset2D::default(), extent: self.swapchain_extent };
            device.cmd_set_viewport(cmd, 0, std::slice::from_ref(&viewport));
            device.cmd_set_scissor(cmd, 0, std::slice::from_ref(&scissor));

            device.cmd_bind_vertex_buffers(cmd, 0, &[self.vertex_buffer, self.instance_buffer], &[0, 0]);
            device.cmd_bind_index_buffer(cmd, self.index_buffer, 0, vk::IndexType::UINT16);

            let pc = PushConstants {
                screen_size: [self.swapchain_extent.width as f32, self.swapchain_extent.height as f32],
                cell_size: [CHAR_W as f32, CHAR_H as f32],
            };
            device.cmd_push_constants(cmd, self.pipeline_layout, vk::ShaderStageFlags::VERTEX,
                0, bytemuck::bytes_of(&pc));

            device.cmd_draw_indexed(cmd, 6, self.instance_count as u32, 0, 0, 0);
            device.cmd_end_render_pass(cmd);
            let _ = device.end_command_buffer(cmd);

            let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
            let submit_info = vk::SubmitInfo::default()
                .wait_semaphores(std::slice::from_ref(&self.image_available[frame]))
                .wait_dst_stage_mask(&wait_stages)
                .command_buffers(std::slice::from_ref(&cmd))
                .signal_semaphores(std::slice::from_ref(&self.render_finished[frame]));
            let _ = device.queue_submit(self.graphics_queue, std::slice::from_ref(&submit_info), self.in_flight[frame]);

            let img_idx = image_index as u32;
            let present_info = vk::PresentInfoKHR::default()
                .wait_semaphores(std::slice::from_ref(&self.render_finished[frame]))
                .swapchains(std::slice::from_ref(&self.swapchain))
                .image_indices(std::slice::from_ref(&img_idx));
            let _ = self.swapchain_loader.queue_present(self.graphics_queue, &present_info);
        }

        self.frame_index = (self.frame_index + 1) % MAX_FRAMES;
    }

    pub fn grid_w(&self) -> usize { self.grid_w }
    pub fn grid_h(&self) -> usize { self.grid_h }
}

impl Drop for GraphicsRenderer {
    fn drop(&mut self) {
        unsafe {
            let _ = self.device.device_wait_idle();
            for &f in &self.in_flight { self.device.destroy_fence(f, None); }
            for &s in &self.image_available { self.device.destroy_semaphore(s, None); }
            for &s in &self.render_finished { self.device.destroy_semaphore(s, None); }
            self.device.destroy_command_pool(self.command_pool, None);
            for &fb in &self.framebuffers { self.device.destroy_framebuffer(fb, None); }
            self.device.destroy_pipeline(self.pipeline, None);
            self.device.destroy_pipeline_layout(self.pipeline_layout, None);
            self.device.destroy_render_pass(self.render_pass, None);
            self.device.destroy_buffer(self.instance_buffer, None);
            self.device.free_memory(self.instance_memory, None);
            self.device.destroy_buffer(self.vertex_buffer, None);
            self.device.free_memory(self.vertex_memory, None);
            self.device.destroy_buffer(self.index_buffer, None);
            self.device.free_memory(self.index_memory, None);
            for &v in &self.swapchain_image_views { self.device.destroy_image_view(v, None); }
            self.swapchain_loader.destroy_swapchain(self.swapchain, None);
            let sl = ash::khr::surface::Instance::new(&self.entry, &self.instance);
            sl.destroy_surface(self.surface, None);
            self.device.destroy_device(None);
            self.instance.destroy_instance(None);
        }
    }
}
