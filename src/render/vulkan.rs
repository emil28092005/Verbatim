use ash::vk;
use fontdue::{Font, FontSettings};
use std::ffi::CString;
use std::sync::Arc;

use crate::entity::{EntityManager, EntityKind};
use crate::world::cell::MaterialId;
use crate::world::grid::Grid;

const CHAR_W: u32 = 16;
const CHAR_H: u32 = 16;
const ATLAS_COLS: usize = 16;
const ATLAS_ROWS: usize = 16;
const ATLAS_W: u32 = (ATLAS_COLS as u32) * CHAR_W;
const ATLAS_H: u32 = (ATLAS_ROWS as u32) * CHAR_H;
const MAX_FRAMES: usize = 2;

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct CellInstance {
    grid_x: f32,
    grid_y: f32,
    atlas_u: f32,
    atlas_v: f32,
    atlas_w: f32,
    atlas_h: f32,
    fg: [u8; 4],
    bg: [u8; 4],
}

#[repr(C)]
#[derive(bytemuck::NoUninit, Clone, Copy)]
struct PushConstants {
    screen_size: [f32; 2],
    cell_size: [f32; 2],
}

pub struct VulkanRenderer {
    grid_w: usize,
    grid_h: usize,

    entry: ash::Entry,
    instance: ash::Instance,
    surface: vk::SurfaceKHR,
    physical_device: vk::PhysicalDevice,
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

    atlas_image: vk::Image,
    atlas_memory: vk::DeviceMemory,
    atlas_view: vk::ImageView,
    atlas_sampler: vk::Sampler,
    atlas_map: std::collections::HashMap<char, (f32, f32, f32, f32)>,

    instance_buffer: vk::Buffer,
    instance_memory: vk::DeviceMemory,
    instance_ptr: *mut CellInstance,
    instance_count: usize,

    descriptor_pool: vk::DescriptorPool,
    descriptor_set: vk::DescriptorSet,
    descriptor_set_layout: vk::DescriptorSetLayout,
    window: Arc<winit::window::Window>,
}

impl VulkanRenderer {
    pub fn new(window: Arc<winit::window::Window>) -> Result<Self, String> {
        let grid_w = 160usize;
        let grid_h = 50usize;
        let pixel_w = (grid_w as u32) * CHAR_W;
        let pixel_h = (grid_h as u32) * CHAR_H;

        let entry = unsafe { ash::Entry::load().map_err(|e| format!("Vulkan load: {e}"))? };

        // Get required instance extensions from the window's display handle (platform-agnostic)
        use raw_window_handle::HasDisplayHandle;
        let dh = window.display_handle().map_err(|e| format!("display_handle: {e}"))?;
        let required_exts = ash_window::enumerate_required_extensions(dh.as_raw())
            .map_err(|e| format!("enumerate_required_extensions: {e:?}"))?;

        let instance = create_instance(&entry, required_exts)?;
        let surface = create_surface(&entry, &instance, &window)?;

        let surface_loader = ash::khr::surface::Instance::new(&entry, &instance);
        let (physical_device, queue_family) =
            pick_physical_device(&instance, &surface_loader, surface)?;
        let (device, graphics_queue) =
            create_device(&instance, physical_device, queue_family)?;

        let swapchain_loader = ash::khr::swapchain::Device::new(&instance, &device);
        let (swapchain, swapchain_images, swapchain_format, swapchain_extent) =
            create_swapchain(&device, &swapchain_loader, &surface_loader,
                physical_device, surface, queue_family, pixel_w, pixel_h)?;

        let swapchain_image_views: Vec<_> = swapchain_images.iter()
            .map(|&img| create_image_view(&device, img, swapchain_format))
            .collect();

        let render_pass = create_render_pass(&device, swapchain_format)?;
        let (descriptor_set_layout, descriptor_pool, descriptor_set) =
            create_descriptor(&device)?;
        let (pipeline_layout, pipeline) = create_pipeline(&device, render_pass, descriptor_set_layout)?;
        let framebuffers: Vec<_> = swapchain_image_views.iter()
            .map(|&view| create_framebuffer(&device, render_pass, view, swapchain_extent))
            .collect();
        let command_pool = create_command_pool(&device, queue_family)?;
        let command_buffers = create_command_buffers(&device, command_pool, framebuffers.len())?;
        let (image_available, render_finished, in_flight) = create_sync(&device)?;

        let (vertex_buffer, vertex_memory, index_buffer, index_memory) =
            create_vertex_index_buffers(&device, &instance, physical_device)?;

        let (atlas_image, atlas_memory, atlas_view, atlas_sampler, atlas_map) =
            create_atlas_texture(&device, &instance, physical_device, &graphics_queue, command_pool)?;

        let instance_count = grid_w * grid_h;
        let (instance_buffer, instance_memory, instance_ptr) =
            create_instance_buffer(&device, &instance, physical_device, instance_count)?;

        update_descriptor_set(&device, descriptor_set, atlas_view, atlas_sampler);

        Ok(Self {
            grid_w, grid_h,
            entry, instance, surface, physical_device, device,
            graphics_queue,
            swapchain_loader, swapchain, swapchain_image_views,
            swapchain_extent,
            render_pass, pipeline, pipeline_layout, framebuffers,
            command_pool, command_buffers,
            image_available, render_finished, in_flight, frame_index: 0,
            vertex_buffer, vertex_memory, index_buffer, index_memory,
            atlas_image, atlas_memory, atlas_view, atlas_sampler, atlas_map,
            instance_buffer, instance_memory, instance_ptr, instance_count,
            descriptor_pool, descriptor_set, descriptor_set_layout,
            window,
        })
    }

    pub fn render(&mut self, grid: &Grid, entities: &EntityManager, cam_x: i32, cam_y: i32) {
        self.check_resize();

        let mut entity_map: std::collections::HashMap<(i32, i32), (char, [u8; 4])> = std::collections::HashMap::new();
        for e in entities.all() {
            for b in &e.bodies {
                if !b.alive { continue; }
                let sx = b.x as i32 - cam_x;
                let sy = b.y as i32 - cam_y;
                if sx >= 0 && sx < self.grid_w as i32 && sy >= 0 && sy < self.grid_h as i32 {
                    let ch = match e.kind {
                        EntityKind::Player if e.alive => '@',
                        EntityKind::Goblin if e.alive => 'g',
                        _ => '%',
                    };
                    let fg = if e.on_fire { [255, 160, 40, 255] }
                    else if !e.alive { [100, 60, 60, 255] }
                    else {
                        match e.kind {
                            EntityKind::Player => [255, 255, 100, 255],
                            EntityKind::Goblin => [100, 220, 100, 255],
                            _ => [180, 50, 50, 255],
                        }
                    };
                    entity_map.insert((sx, sy), (ch, fg));
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

                let (ch, fg, bg) = if let Some(&(ec, ef)) = entity_map.get(&(dx as i32, dy as i32)) {
                    (ec, ef, bg_default)
                } else if !grid.in_bounds(wx, wy) {
                    ('?', [80, 80, 80, 255], bg_default)
                } else {
                    let cell = grid.get(wx, wy);
                    if cell.is_empty() {
                        (' ', [10, 10, 15, 255], bg_default)
                    } else {
                        let fg = if cell.material == MaterialId::Lava {
                            let r = 200u8.saturating_add(cell.variant / 2);
                            [r, 60, 20, 255]
                        } else {
                            [cell.fg[0], cell.fg[1], cell.fg[2], 255]
                        };
                        let bg = [cell.bg[0], cell.bg[1], cell.bg[2], 255];
                        (cell.material.display_char(), fg, bg)
                    }
                };

                let (au, av, aw, ah) = self.atlas_map.get(&ch).copied().unwrap_or((0.0, 0.0, 0.0, 0.0));
                instances[idx] = CellInstance {
                    grid_x: dx as f32, grid_y: dy as f32,
                    atlas_u: au, atlas_v: av, atlas_w: aw, atlas_h: ah,
                    fg, bg,
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
            let scissor = vk::Rect2D {
                offset: vk::Offset2D::default(),
                extent: self.swapchain_extent,
            };
            device.cmd_set_viewport(cmd, 0, std::slice::from_ref(&viewport));
            device.cmd_set_scissor(cmd, 0, std::slice::from_ref(&scissor));

            device.cmd_bind_vertex_buffers(cmd, 0, &[self.vertex_buffer, self.instance_buffer], &[0, 0]);
            device.cmd_bind_index_buffer(cmd, self.index_buffer, 0, vk::IndexType::UINT16);
            device.cmd_bind_descriptor_sets(cmd, vk::PipelineBindPoint::GRAPHICS,
                self.pipeline_layout, 0, &[self.descriptor_set], &[]);

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

    fn check_resize(&mut self) {
        let sl = ash::khr::surface::Instance::new(&self.entry, &self.instance);
        let caps = match unsafe { sl.get_physical_device_surface_capabilities(self.physical_device, self.surface) } {
            Ok(c) => c,
            Err(_) => return,
        };

        let new_extent = if caps.current_extent.width != u32::MAX {
            caps.current_extent
        } else {
            // Wayland: surface extent is undefined, use window inner size
            let inner = self.window.inner_size();
            vk::Extent2D {
                width: inner.width.max(1),
                height: inner.height.max(1),
            }
        };

        if new_extent.width == self.swapchain_extent.width
            && new_extent.height == self.swapchain_extent.height {
            return;
        }

        if new_extent.width == 0 || new_extent.height == 0 {
            return;
        }

        unsafe { let _ = self.device.device_wait_idle(); }

        for &fb in &self.framebuffers { unsafe { self.device.destroy_framebuffer(fb, None); } }
        for &v in &self.swapchain_image_views { unsafe { self.device.destroy_image_view(v, None); } }

        let sci = vk::SwapchainCreateInfoKHR::default()
            .surface(self.surface)
            .min_image_count(caps.min_image_count.max(2))
            .image_format(vk::Format::B8G8R8A8_UNORM)
            .image_color_space(vk::ColorSpaceKHR::SRGB_NONLINEAR)
            .image_extent(new_extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .pre_transform(caps.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(vk::PresentModeKHR::FIFO)
            .clipped(true)
            .old_swapchain(self.swapchain);

        let new_swapchain = match unsafe { self.swapchain_loader.create_swapchain(&sci, None) } {
            Ok(s) => s,
            Err(_) => return,
        };

        let new_images = match unsafe { self.swapchain_loader.get_swapchain_images(new_swapchain) } {
            Ok(i) => i,
            Err(_) => return,
        };

        let new_views: Vec<_> = new_images.iter()
            .map(|&img| {
                let vi = vk::ImageViewCreateInfo::default()
                    .image(img).view_type(vk::ImageViewType::TYPE_2D)
                    .format(vk::Format::B8G8R8A8_UNORM)
                    .subresource_range(vk::ImageSubresourceRange {
                        aspect_mask: vk::ImageAspectFlags::COLOR,
                        base_mip_level: 0, level_count: 1,
                        base_array_layer: 0, layer_count: 1,
                    });
                unsafe { self.device.create_image_view(&vi, None).expect("image_view") }
            }).collect();

        let new_framebuffers: Vec<_> = new_views.iter()
            .map(|&view| {
                let fci = vk::FramebufferCreateInfo::default()
                    .render_pass(self.render_pass)
                    .attachments(std::slice::from_ref(&view))
                    .width(new_extent.width).height(new_extent.height).layers(1);
                unsafe { self.device.create_framebuffer(&fci, None).expect("fb") }
            }).collect();

        unsafe { self.device.free_command_buffers(self.command_pool, &self.command_buffers); }
        let cai = vk::CommandBufferAllocateInfo::default()
            .command_pool(self.command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(new_framebuffers.len() as u32);
        let new_cmd_bufs = unsafe { self.device.allocate_command_buffers(&cai).expect("cmd_bufs") };

        let new_grid_w = (new_extent.width / CHAR_W) as usize;
        let new_grid_h = (new_extent.height / CHAR_H) as usize;
        let new_count = new_grid_w * new_grid_h;

        if new_count != self.instance_count {
            unsafe {
                self.device.unmap_memory(self.instance_memory);
                self.device.destroy_buffer(self.instance_buffer, None);
                self.device.free_memory(self.instance_memory, None);
            }

            let inst_sz = (new_count * std::mem::size_of::<CellInstance>()) as vk::DeviceSize;
            let ibi = vk::BufferCreateInfo::default()
                .size(inst_sz).usage(vk::BufferUsageFlags::VERTEX_BUFFER)
                .sharing_mode(vk::SharingMode::EXCLUSIVE);
            self.instance_buffer = unsafe { self.device.create_buffer(&ibi, None) }.expect("inst buf");
            let ireq = unsafe { self.device.get_buffer_memory_requirements(self.instance_buffer) };

            let find_mem = |filter: u32, props: vk::MemoryPropertyFlags| -> u32 {
                let mp = unsafe { self.instance.get_physical_device_memory_properties(self.physical_device) };
                for (i, mt) in mp.memory_types.iter().enumerate() {
                    if (filter & (1 << i)) != 0 && mt.property_flags.contains(props) { return i as u32; }
                }
                0
            };
            let imt = find_mem(ireq.memory_type_bits, vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT);
            self.instance_memory = unsafe { self.device.allocate_memory(
                &vk::MemoryAllocateInfo::default().allocation_size(ireq.size).memory_type_index(imt), None)
            }.expect("inst mem");
            self.instance_ptr = unsafe {
                self.device.bind_buffer_memory(self.instance_buffer, self.instance_memory, 0).expect("bind");
                let ptr = self.device.map_memory(self.instance_memory, 0, inst_sz, vk::MemoryMapFlags::default()).expect("map");
                ptr as *mut CellInstance
            };
            self.instance_count = new_count;
        }

        unsafe { self.swapchain_loader.destroy_swapchain(self.swapchain, None); }

        self.swapchain = new_swapchain;
        self.swapchain_image_views = new_views;
        self.framebuffers = new_framebuffers;
        self.command_buffers = new_cmd_bufs;
        self.swapchain_extent = new_extent;
        self.grid_w = new_grid_w;
        self.grid_h = new_grid_h;
    }
}

impl Drop for VulkanRenderer {
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
            self.device.destroy_sampler(self.atlas_sampler, None);
            self.device.destroy_image_view(self.atlas_view, None);
            self.device.destroy_image(self.atlas_image, None);
            self.device.free_memory(self.atlas_memory, None);
            self.device.destroy_buffer(self.instance_buffer, None);
            self.device.free_memory(self.instance_memory, None);
            self.device.destroy_buffer(self.vertex_buffer, None);
            self.device.free_memory(self.vertex_memory, None);
            self.device.destroy_buffer(self.index_buffer, None);
            self.device.free_memory(self.index_memory, None);
            self.device.destroy_descriptor_pool(self.descriptor_pool, None);
            self.device.destroy_descriptor_set_layout(self.descriptor_set_layout, None);
            for &v in &self.swapchain_image_views { self.device.destroy_image_view(v, None); }
            self.swapchain_loader.destroy_swapchain(self.swapchain, None);
            let sl = ash::khr::surface::Instance::new(&self.entry, &self.instance);
            sl.destroy_surface(self.surface, None);
            self.device.destroy_device(None);
            self.instance.destroy_instance(None);
        }
    }
}

fn create_instance(entry: &ash::Entry, required_exts: &'static [*const std::ffi::c_char]) -> Result<ash::Instance, String> {
    let app_name = CString::new("Verbatim").unwrap();
    let app_info = vk::ApplicationInfo::default()
        .application_name(&app_name)
        .api_version(vk::API_VERSION_1_2);

    // Start with platform-required extensions (from ash_window)
    let mut ext_ptrs: Vec<*const i8> = required_exts.iter().map(|&p| p as *const i8).collect();

    // Add debug utils extension if available
    let avail_exts = unsafe { entry.enumerate_instance_extension_properties(None) }.unwrap_or_default();
    let has_debug_utils = avail_exts.iter().any(|e| {
        let name = unsafe { std::ffi::CStr::from_ptr(e.extension_name.as_ptr() as *const i8) };
        name.to_str().unwrap_or("") == "VK_EXT_debug_utils"
    });
    if has_debug_utils {
        ext_ptrs.push(b"VK_EXT_debug_utils\0".as_ptr() as *const i8);
    }

    let create_info = vk::InstanceCreateInfo::default()
        .application_info(&app_info)
        .enabled_extension_names(&ext_ptrs);
    unsafe { entry.create_instance(&create_info, None).map_err(|e| format!("instance: {e:?}")) }
}

fn create_surface(entry: &ash::Entry, instance: &ash::Instance, window: &winit::window::Window) -> Result<vk::SurfaceKHR, String> {
    use raw_window_handle::{HasWindowHandle, HasDisplayHandle};
    let wh = window.window_handle().map_err(|e| format!("wh: {e}"))?;
    let dh = window.display_handle().map_err(|e| format!("dh: {e}"))?;
    let wh_raw = wh.as_raw();
    let dh_raw = dh.as_raw();
    unsafe {
        ash_window::create_surface(entry, instance, dh_raw, wh_raw, None)
            .map_err(|e| format!("surface: {e:?}"))
    }
}

fn pick_physical_device(
    instance: &ash::Instance,
    sl: &ash::khr::surface::Instance,
    surface: vk::SurfaceKHR,
) -> Result<(vk::PhysicalDevice, u32), String> {
    let devices = unsafe { instance.enumerate_physical_devices().map_err(|e| format!("enum: {e:?}"))? };
    for &pd in &devices {
        let props = unsafe { instance.get_physical_device_properties(pd) };
        if props.device_type == vk::PhysicalDeviceType::CPU { continue; }
        let qfs = unsafe { instance.get_physical_device_queue_family_properties(pd) };
        for (i, qf) in qfs.iter().enumerate() {
            if qf.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                let ok = unsafe { sl.get_physical_device_surface_support(pd, i as u32, surface) }.unwrap_or(false);
                if ok { return Ok((pd, i as u32)); }
            }
        }
    }
    Err("No GPU".to_string())
}

fn create_device(instance: &ash::Instance, pd: vk::PhysicalDevice, qf: u32) -> Result<(ash::Device, vk::Queue), String> {
    let qp = [1.0f32];
    let qi = vk::DeviceQueueCreateInfo::default().queue_family_index(qf).queue_priorities(&qp);
    let ext_names: Vec<CString> = vec![CString::new("VK_KHR_swapchain").unwrap()];
    let ext_ptrs: Vec<*const i8> = ext_names.iter().map(|n| n.as_ptr()).collect();
    let ci = vk::DeviceCreateInfo::default()
        .queue_create_infos(std::slice::from_ref(&qi))
        .enabled_extension_names(&ext_ptrs);
    unsafe {
        let device = instance.create_device(pd, &ci, None).map_err(|e| format!("device: {e:?}"))?;
        let queue = device.get_device_queue(qf, 0);
        Ok((device, queue))
    }
}

fn create_swapchain(
    _device: &ash::Device,
    sl: &ash::khr::swapchain::Device,
    surface_loader: &ash::khr::surface::Instance,
    pd: vk::PhysicalDevice,
    surface: vk::SurfaceKHR,
    qf: u32,
    pw: u32, ph: u32,
) -> Result<(vk::SwapchainKHR, Vec<vk::Image>, vk::Format, vk::Extent2D), String> {
    let caps = unsafe { surface_loader.get_physical_device_surface_capabilities(pd, surface) }
        .map_err(|e| format!("caps: {e:?}"))?;
    let format = vk::SurfaceFormatKHR {
        format: vk::Format::B8G8R8A8_UNORM,
        color_space: vk::ColorSpaceKHR::SRGB_NONLINEAR,
    };
    let extent = if caps.current_extent.width != u32::MAX { caps.current_extent }
        else { vk::Extent2D { width: pw, height: ph } };
    let ic = caps.min_image_count.max(2);
    let qf_slice = [qf];
    let ci = vk::SwapchainCreateInfoKHR::default()
        .surface(surface)
        .min_image_count(ic)
        .image_format(format.format)
        .image_color_space(format.color_space)
        .image_extent(extent)
        .image_array_layers(1)
        .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
        .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
        .queue_family_indices(&qf_slice)
        .pre_transform(caps.current_transform)
        .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
        .present_mode(vk::PresentModeKHR::FIFO)
        .clipped(true);
    let swapchain = unsafe { sl.create_swapchain(&ci, None) }.map_err(|e| format!("swapchain: {e:?}"))?;
    let images = unsafe { sl.get_swapchain_images(swapchain) }.map_err(|e| format!("images: {e:?}"))?;
    Ok((swapchain, images, format.format, extent))
}

fn create_image_view(device: &ash::Device, image: vk::Image, format: vk::Format) -> vk::ImageView {
    let ci = vk::ImageViewCreateInfo::default()
        .image(image).view_type(vk::ImageViewType::TYPE_2D).format(format)
        .subresource_range(vk::ImageSubresourceRange {
            aspect_mask: vk::ImageAspectFlags::COLOR, base_mip_level: 0,
            level_count: 1, base_array_layer: 0, layer_count: 1,
        });
    unsafe { device.create_image_view(&ci, None).expect("image_view") }
}

fn create_render_pass(device: &ash::Device, format: vk::Format) -> Result<vk::RenderPass, String> {
    let att = vk::AttachmentDescription::default()
        .format(format).samples(vk::SampleCountFlags::TYPE_1)
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
    let ci = vk::RenderPassCreateInfo::default()
        .attachments(std::slice::from_ref(&att))
        .subpasses(std::slice::from_ref(&subpass))
        .dependencies(std::slice::from_ref(&dep));
    unsafe { device.create_render_pass(&ci, None).map_err(|e| format!("render_pass: {e:?}")) }
}

fn create_descriptor(device: &ash::Device) -> Result<(vk::DescriptorSetLayout, vk::DescriptorPool, vk::DescriptorSet), String> {
    let binding = vk::DescriptorSetLayoutBinding::default()
        .binding(0).descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
        .descriptor_count(1).stage_flags(vk::ShaderStageFlags::FRAGMENT);
    let li = vk::DescriptorSetLayoutCreateInfo::default().bindings(std::slice::from_ref(&binding));
    let layout = unsafe { device.create_descriptor_set_layout(&li, None) }.map_err(|e| format!("ds_layout: {e:?}"))?;
    let ps = vk::DescriptorPoolSize { ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER, descriptor_count: 1 };
    let pi = vk::DescriptorPoolCreateInfo::default().pool_sizes(std::slice::from_ref(&ps)).max_sets(1);
    let pool = unsafe { device.create_descriptor_pool(&pi, None) }.map_err(|e| format!("ds_pool: {e:?}"))?;
    let ai = vk::DescriptorSetAllocateInfo::default().descriptor_pool(pool).set_layouts(std::slice::from_ref(&layout));
    let sets = unsafe { device.allocate_descriptor_sets(&ai) }.map_err(|e| format!("alloc_ds: {e:?}"))?;
    Ok((layout, pool, sets[0]))
}

fn create_pipeline(device: &ash::Device, rp: vk::RenderPass, ds_layout: vk::DescriptorSetLayout) -> Result<(vk::PipelineLayout, vk::Pipeline), String> {
    let vert_spv = include_bytes!("../../assets/shaders/cell_vert.spv");
    let frag_spv = include_bytes!("../../assets/shaders/cell_frag.spv");
    let vert_code: Vec<u32> = vert_spv.chunks_exact(4).map(|c| u32::from_ne_bytes([c[0], c[1], c[2], c[3]])).collect();
    let frag_code: Vec<u32> = frag_spv.chunks_exact(4).map(|c| u32::from_ne_bytes([c[0], c[1], c[2], c[3]])).collect();
    let vm = unsafe { device.create_shader_module(&vk::ShaderModuleCreateInfo::default().code(&vert_code), None) }.map_err(|e| format!("vert: {e:?}"))?;
    let fm = unsafe { device.create_shader_module(&vk::ShaderModuleCreateInfo::default().code(&frag_code), None) }.map_err(|e| format!("frag: {e:?}"))?;
    let main = CString::new("main").unwrap();
    let vs = vk::PipelineShaderStageCreateInfo::default().stage(vk::ShaderStageFlags::VERTEX).module(vm).name(&main);
    let fs = vk::PipelineShaderStageCreateInfo::default().stage(vk::ShaderStageFlags::FRAGMENT).module(fm).name(&main);

    let bindings = [
        vk::VertexInputBindingDescription { binding: 0, stride: 8, input_rate: vk::VertexInputRate::VERTEX },
        vk::VertexInputBindingDescription { binding: 1, stride: std::mem::size_of::<CellInstance>() as u32, input_rate: vk::VertexInputRate::INSTANCE },
    ];
    let attrs = [
        vk::VertexInputAttributeDescription { location: 0, binding: 0, format: vk::Format::R32G32_SFLOAT, offset: 0 },
        vk::VertexInputAttributeDescription { location: 1, binding: 1, format: vk::Format::R32G32_SFLOAT, offset: 0 },
        vk::VertexInputAttributeDescription { location: 2, binding: 1, format: vk::Format::R32G32B32A32_SFLOAT, offset: 8 },
        vk::VertexInputAttributeDescription { location: 3, binding: 1, format: vk::Format::R8G8B8A8_UNORM, offset: 24 },
        vk::VertexInputAttributeDescription { location: 4, binding: 1, format: vk::Format::R8G8B8A8_UNORM, offset: 28 },
    ];
    let vi = vk::PipelineVertexInputStateCreateInfo::default()
        .vertex_binding_descriptions(&bindings).vertex_attribute_descriptions(&attrs);
    let ia = vk::PipelineInputAssemblyStateCreateInfo::default().topology(vk::PrimitiveTopology::TRIANGLE_LIST);
    let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
    let dynamic_state = vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_states);
    let vs_state = vk::PipelineViewportStateCreateInfo::default()
        .viewport_count(1)
        .scissor_count(1);
    let rs = vk::PipelineRasterizationStateCreateInfo::default().line_width(1.0).cull_mode(vk::CullModeFlags::NONE);
    let ms = vk::PipelineMultisampleStateCreateInfo::default().rasterization_samples(vk::SampleCountFlags::TYPE_1);
    let cba = vk::PipelineColorBlendAttachmentState::default().color_write_mask(vk::ColorComponentFlags::RGBA);
    let cb = vk::PipelineColorBlendStateCreateInfo::default().attachments(std::slice::from_ref(&cba));
    let pcr = vk::PushConstantRange { stage_flags: vk::ShaderStageFlags::VERTEX, offset: 0, size: std::mem::size_of::<PushConstants>() as u32 };
    let li = vk::PipelineLayoutCreateInfo::default()
        .set_layouts(std::slice::from_ref(&ds_layout))
        .push_constant_ranges(std::slice::from_ref(&pcr));
    let layout = unsafe { device.create_pipeline_layout(&li, None) }.map_err(|e| format!("pipeline_layout: {e:?}"))?;
    let stages = [vs, fs];
    let pi = vk::GraphicsPipelineCreateInfo::default()
        .stages(&stages).vertex_input_state(&vi).input_assembly_state(&ia)
        .viewport_state(&vs_state).rasterization_state(&rs).multisample_state(&ms)
        .color_blend_state(&cb).dynamic_state(&dynamic_state)
        .layout(layout).render_pass(rp).subpass(0);
    let pipes = unsafe { device.create_graphics_pipelines(vk::PipelineCache::null(), std::slice::from_ref(&pi), None) }
        .map_err(|(_, e)| format!("pipeline: {e:?}"))?;
    unsafe { device.destroy_shader_module(vm, None); device.destroy_shader_module(fm, None); }
    Ok((layout, pipes[0]))
}

fn create_framebuffer(device: &ash::Device, rp: vk::RenderPass, view: vk::ImageView, ext: vk::Extent2D) -> vk::Framebuffer {
    let ci = vk::FramebufferCreateInfo::default().render_pass(rp).attachments(std::slice::from_ref(&view))
        .width(ext.width).height(ext.height).layers(1);
    unsafe { device.create_framebuffer(&ci, None).expect("fb") }
}

fn create_command_pool(device: &ash::Device, qf: u32) -> Result<vk::CommandPool, String> {
    let ci = vk::CommandPoolCreateInfo::default().queue_family_index(qf).flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);
    unsafe { device.create_command_pool(&ci, None).map_err(|e| format!("cmd_pool: {e:?}")) }
}

fn create_command_buffers(device: &ash::Device, pool: vk::CommandPool, count: usize) -> Result<Vec<vk::CommandBuffer>, String> {
    let ai = vk::CommandBufferAllocateInfo::default().command_pool(pool).level(vk::CommandBufferLevel::PRIMARY).command_buffer_count(count as u32);
    unsafe { device.allocate_command_buffers(&ai).map_err(|e| format!("cmd_bufs: {e:?}")) }
}

fn create_sync(device: &ash::Device) -> Result<(Vec<vk::Semaphore>, Vec<vk::Semaphore>, Vec<vk::Fence>), String> {
    let mut ia = Vec::new(); let mut rf = Vec::new(); let mut ifl = Vec::new();
    for _ in 0..MAX_FRAMES {
        unsafe {
            ia.push(device.create_semaphore(&vk::SemaphoreCreateInfo::default(), None).map_err(|e| format!("sem: {e:?}"))?);
            rf.push(device.create_semaphore(&vk::SemaphoreCreateInfo::default(), None).map_err(|e| format!("sem: {e:?}"))?);
            ifl.push(device.create_fence(&vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED), None).map_err(|e| format!("fence: {e:?}"))?);
        }
    }
    Ok((ia, rf, ifl))
}

fn find_mem_type(instance: &ash::Instance, pd: vk::PhysicalDevice, filter: u32, props: vk::MemoryPropertyFlags) -> Result<u32, String> {
    let mp = unsafe { instance.get_physical_device_memory_properties(pd) };
    for (i, mt) in mp.memory_types.iter().enumerate() {
        if (filter & (1 << i)) != 0 && mt.property_flags.contains(props) { return Ok(i as u32); }
    }
    Err("No memory type".to_string())
}

fn create_buffer_with_data<T: Copy>(
    device: &ash::Device, instance: &ash::Instance, pd: vk::PhysicalDevice,
    data: &[T], usage: vk::BufferUsageFlags,
) -> Result<(vk::Buffer, vk::DeviceMemory), String> {
    let size = (data.len() * std::mem::size_of::<T>()) as vk::DeviceSize;
    let bi = vk::BufferCreateInfo::default().size(size).usage(usage).sharing_mode(vk::SharingMode::EXCLUSIVE);
    let buf = unsafe { device.create_buffer(&bi, None) }.map_err(|e| format!("buf: {e:?}"))?;
    let req = unsafe { device.get_buffer_memory_requirements(buf) };
    let mt = find_mem_type(instance, pd, req.memory_type_bits, vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT)?;
    let ai = vk::MemoryAllocateInfo::default().allocation_size(req.size).memory_type_index(mt);
    let mem = unsafe { device.allocate_memory(&ai, None) }.map_err(|e| format!("mem: {e:?}"))?;
    unsafe {
        device.bind_buffer_memory(buf, mem, 0).expect("bind");
        let ptr = device.map_memory(mem, 0, size, vk::MemoryMapFlags::default()).expect("map");
        std::ptr::copy_nonoverlapping(data.as_ptr() as *const u8, ptr as *mut u8, size as usize);
        device.unmap_memory(mem);
    }
    Ok((buf, mem))
}

fn create_vertex_index_buffers(
    device: &ash::Device, instance: &ash::Instance, pd: vk::PhysicalDevice,
) -> Result<(vk::Buffer, vk::DeviceMemory, vk::Buffer, vk::DeviceMemory), String> {
    let verts: [f32; 8] = [0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, 1.0];
    let indices: [u16; 6] = [0, 1, 2, 1, 3, 2];
    let (vb, vm) = create_buffer_with_data(device, instance, pd, &verts, vk::BufferUsageFlags::VERTEX_BUFFER)?;
    let (ib, im) = create_buffer_with_data(device, instance, pd, &indices, vk::BufferUsageFlags::INDEX_BUFFER)?;
    Ok((vb, vm, ib, im))
}

fn create_atlas_texture(
    device: &ash::Device, instance: &ash::Instance, pd: vk::PhysicalDevice,
    queue: &vk::Queue, pool: vk::CommandPool,
) -> Result<(vk::Image, vk::DeviceMemory, vk::ImageView, vk::Sampler, std::collections::HashMap<char, (f32, f32, f32, f32)>), String> {
    let font_bytes: &[u8] = include_bytes!("../../assets/DejaVuSansMono.ttf");
    let font = Font::from_bytes(font_bytes, FontSettings { collection_index: 0, scale: CHAR_H as f32, load_substitutions: false }).expect("font");
    let mut atlas_data = vec![0u8; (ATLAS_W * ATLAS_H) as usize];
    let mut atlas_map = std::collections::HashMap::new();
    let chars: Vec<char> = " !\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~?".chars().collect();
    for (i, &ch) in chars.iter().enumerate() {
        let col = i % ATLAS_COLS; let row = i / ATLAS_COLS;
        atlas_map.insert(ch, (
            (col as f32 * CHAR_W as f32) / ATLAS_W as f32,
            (row as f32 * CHAR_H as f32) / ATLAS_H as f32,
            CHAR_W as f32 / ATLAS_W as f32,
            CHAR_H as f32 / ATLAS_H as f32,
        ));
        let (metrics, bitmap) = font.rasterize(ch, CHAR_H as f32);
        for y in 0..metrics.height.min(CHAR_H as usize) {
            for x in 0..metrics.width.min(CHAR_W as usize) {
                let a = bitmap[y * metrics.width + x];
                if a > 0 {
                    let px = (x as i32 + metrics.xmin).max(0) as usize;
                    let py = (y as i32 + CHAR_H as i32 - metrics.height as i32 - metrics.ymin).max(0) as usize;
                    if px < CHAR_W as usize && py < CHAR_H as usize {
                        atlas_data[(row * CHAR_H as usize + py) * ATLAS_W as usize + col * CHAR_W as usize + px] = a;
                    }
                }
            }
        }
    }

    let ii = vk::ImageCreateInfo::default()
        .image_type(vk::ImageType::TYPE_2D)
        .extent(vk::Extent3D { width: ATLAS_W, height: ATLAS_H, depth: 1 })
        .mip_levels(1).array_layers(1).format(vk::Format::R8_UNORM)
        .tiling(vk::ImageTiling::OPTIMAL).initial_layout(vk::ImageLayout::UNDEFINED)
        .usage(vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED)
        .samples(vk::SampleCountFlags::TYPE_1).sharing_mode(vk::SharingMode::EXCLUSIVE);
    let image = unsafe { device.create_image(&ii, None) }.map_err(|e| format!("img: {e:?}"))?;
    let req = unsafe { device.get_image_memory_requirements(image) };
    let mt = find_mem_type(instance, pd, req.memory_type_bits, vk::MemoryPropertyFlags::DEVICE_LOCAL)?;
    let mem = unsafe { device.allocate_memory(&vk::MemoryAllocateInfo::default().allocation_size(req.size).memory_type_index(mt), None) }.map_err(|e| format!("img mem: {e:?}"))?;
    unsafe { device.bind_image_memory(image, mem, 0).expect("bind img"); }

    // Staging
    let sz = atlas_data.len() as vk::DeviceSize;
    let sbi = vk::BufferCreateInfo::default().size(sz).usage(vk::BufferUsageFlags::TRANSFER_SRC).sharing_mode(vk::SharingMode::EXCLUSIVE);
    let sbuf = unsafe { device.create_buffer(&sbi, None) }.expect("staging buf");
    let sreq = unsafe { device.get_buffer_memory_requirements(sbuf) };
    let smt = find_mem_type(instance, pd, sreq.memory_type_bits, vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT)?;
    let smem = unsafe { device.allocate_memory(&vk::MemoryAllocateInfo::default().allocation_size(sreq.size).memory_type_index(smt), None) }.expect("staging mem");
    unsafe {
        device.bind_buffer_memory(sbuf, smem, 0).expect("bind staging");
        let ptr = device.map_memory(smem, 0, sz, vk::MemoryMapFlags::default()).expect("map staging");
        std::ptr::copy_nonoverlapping(atlas_data.as_ptr(), ptr as *mut u8, atlas_data.len());
        device.unmap_memory(smem);
    }

    // Copy
    let cmd = unsafe {
        let c = device.allocate_command_buffers(&vk::CommandBufferAllocateInfo::default()
            .command_pool(pool).level(vk::CommandBufferLevel::PRIMARY).command_buffer_count(1)).expect("alloc")[0];
        device.begin_command_buffer(c, &vk::CommandBufferBeginInfo::default().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT)).expect("begin");
        let b1 = vk::ImageMemoryBarrier::default()
            .old_layout(vk::ImageLayout::UNDEFINED).new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED).dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .image(image).subresource_range(vk::ImageSubresourceRange { aspect_mask: vk::ImageAspectFlags::COLOR, base_mip_level: 0, level_count: 1, base_array_layer: 0, layer_count: 1 })
            .src_access_mask(vk::AccessFlags::default()).dst_access_mask(vk::AccessFlags::TRANSFER_WRITE);
        device.cmd_pipeline_barrier(c, vk::PipelineStageFlags::TOP_OF_PIPE, vk::PipelineStageFlags::TRANSFER, vk::DependencyFlags::default(), &[], &[], std::slice::from_ref(&b1));
        device.cmd_copy_buffer_to_image(c, sbuf, image, vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            std::slice::from_ref(&vk::BufferImageCopy::default()
                .buffer_row_length(ATLAS_W).buffer_image_height(ATLAS_H)
                .image_subresource(vk::ImageSubresourceLayers { aspect_mask: vk::ImageAspectFlags::COLOR, mip_level: 0, base_array_layer: 0, layer_count: 1 })
                .image_extent(vk::Extent3D { width: ATLAS_W, height: ATLAS_H, depth: 1 })));
        let b2 = vk::ImageMemoryBarrier::default()
            .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL).new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED).dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .image(image).subresource_range(vk::ImageSubresourceRange { aspect_mask: vk::ImageAspectFlags::COLOR, base_mip_level: 0, level_count: 1, base_array_layer: 0, layer_count: 1 })
            .src_access_mask(vk::AccessFlags::TRANSFER_WRITE).dst_access_mask(vk::AccessFlags::SHADER_READ);
        device.cmd_pipeline_barrier(c, vk::PipelineStageFlags::TRANSFER, vk::PipelineStageFlags::FRAGMENT_SHADER, vk::DependencyFlags::default(), &[], &[], std::slice::from_ref(&b2));
        device.end_command_buffer(c).expect("end");
        let si = vk::SubmitInfo::default().command_buffers(std::slice::from_ref(&c));
        device.queue_submit(*queue, std::slice::from_ref(&si), vk::Fence::null()).expect("submit");
        device.queue_wait_idle(*queue).expect("wait");
        c
    };
    unsafe {
        device.free_command_buffers(pool, &[cmd]);
        device.destroy_buffer(sbuf, None);
        device.free_memory(smem, None);
    }

    let vi = vk::ImageViewCreateInfo::default()
        .image(image).view_type(vk::ImageViewType::TYPE_2D).format(vk::Format::R8_UNORM)
        .subresource_range(vk::ImageSubresourceRange { aspect_mask: vk::ImageAspectFlags::COLOR, base_mip_level: 0, level_count: 1, base_array_layer: 0, layer_count: 1 });
    let view = unsafe { device.create_image_view(&vi, None) }.expect("atlas view");
    let si = vk::SamplerCreateInfo::default()
        .mag_filter(vk::Filter::LINEAR).min_filter(vk::Filter::LINEAR)
        .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_EDGE).address_mode_v(vk::SamplerAddressMode::CLAMP_TO_EDGE).address_mode_w(vk::SamplerAddressMode::CLAMP_TO_EDGE)
        .border_color(vk::BorderColor::INT_OPAQUE_BLACK).unnormalized_coordinates(false);
    let sampler = unsafe { device.create_sampler(&si, None) }.expect("sampler");
    Ok((image, mem, view, sampler, atlas_map))
}

fn create_instance_buffer(
    device: &ash::Device, instance: &ash::Instance, pd: vk::PhysicalDevice, count: usize,
) -> Result<(vk::Buffer, vk::DeviceMemory, *mut CellInstance), String> {
    let sz = (count * std::mem::size_of::<CellInstance>()) as vk::DeviceSize;
    let bi = vk::BufferCreateInfo::default().size(sz).usage(vk::BufferUsageFlags::VERTEX_BUFFER).sharing_mode(vk::SharingMode::EXCLUSIVE);
    let buf = unsafe { device.create_buffer(&bi, None) }.map_err(|e| format!("inst buf: {e:?}"))?;
    let req = unsafe { device.get_buffer_memory_requirements(buf) };
    let mt = find_mem_type(instance, pd, req.memory_type_bits, vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT)?;
    let mem = unsafe { device.allocate_memory(&vk::MemoryAllocateInfo::default().allocation_size(req.size).memory_type_index(mt), None) }.map_err(|e| format!("inst mem: {e:?}"))?;
    unsafe {
        device.bind_buffer_memory(buf, mem, 0).expect("bind inst");
        let ptr = device.map_memory(mem, 0, sz, vk::MemoryMapFlags::default()).expect("map inst");
        Ok((buf, mem, ptr as *mut CellInstance))
    }
}

fn update_descriptor_set(device: &ash::Device, set: vk::DescriptorSet, view: vk::ImageView, sampler: vk::Sampler) {
    let ii = vk::DescriptorImageInfo::default().image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL).image_view(view).sampler(sampler);
    let w = vk::WriteDescriptorSet::default().dst_set(set).dst_binding(0)
        .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).image_info(std::slice::from_ref(&ii));
    unsafe { device.update_descriptor_sets(std::slice::from_ref(&w), &[]); }
}
