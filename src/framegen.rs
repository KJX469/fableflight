//! amd fsr frame generation: a frame made between each two drawn ones by amd's runtime.
//!
//! amd ships it only as signed dlls in the fsr sdk (v2.4.0), so like fsr 4 it is
//! downloaded on request into its own folder, pinned by size and sha-256. it runs on
//! any vulkan gpu: amd's frame generation 4.0 on radeon rx 9000, its 3.1.7 elsewhere.
//!
//! amd's runtime owns the swapchain on vulkan. so while it runs, wgpu's surface is left
//! unconfigured and amd's swapchain is made on the same vulkan surface instead. each
//! frame the finished picture is copied into the image amd hands out, the depth and
//! motion vectors go to amd's prepare pass, and amd's present thread shows a generated
//! frame before each real one. the ui is drawn into its own texture, which amd lays
//! over both, so text never warps. amd needs three queues of its own besides wgpu's
//! (present, async compute, image acquire), asked for when the device is opened.

use crate::fsr4::File;
use parking_lot::Mutex;
use std::path::PathBuf;

/// the files, from the same amd sdk release fsr 4 comes from.
pub const FILES: [File; 3] = [
    File { path: "signedbin/amd_fidelityfx_loader_vk.dll", name: "amd_fidelityfx_loader_vk.dll", size: 39_184, sha256: "38a56a01fb3565c0757d9f5c1e4e69de4b65e5f2e5769b323b200dc1f142bab9" },
    File { path: "signedbin/amd_fidelityfx_framegeneration_vk.dll", name: "amd_fidelityfx_framegeneration_vk.dll", size: 36_806_928, sha256: "d002dabff8c5e71eaa715fc99438cd4dc81230262a7897b16bc402efec3498b4" },
    File { path: "docs/license.md", name: "license.md", size: 65_770, sha256: "32357259b965d6a8001e9e508ba60eda6034027c5923a7d226ffbaff813c2bfb" },
];

pub fn download_size() -> u64 {
    FILES.iter().map(|f| f.size).sum()
}

/// `<data>/fablegame/amd-fsr-fg-2.4.0/`, apart from fsr 4's so neither download clears the other.
pub fn dir() -> Option<PathBuf> {
    dirs::data_dir().map(|d| d.join("fablegame").join(format!("amd-fsr-fg-{}", crate::fsr4::SDK)))
}

static STATE: Mutex<Option<crate::fsr4::State>> = Mutex::new(None);

/// the runtime's state, checked on disk the first time.
pub fn state() -> crate::fsr4::State {
    let mut s = STATE.lock();
    if s.is_none() {
        let on_disk = dir().is_some_and(|d| FILES.iter().all(|f| std::fs::metadata(d.join(f.name)).is_ok_and(|m| m.len() == f.size)));
        *s = Some(if on_disk { crate::fsr4::State::Installed } else { crate::fsr4::State::NotInstalled });
    }
    s.clone().unwrap()
}

fn set(st: crate::fsr4::State) {
    *STATE.lock() = Some(st);
}

/// downloads the runtime on a thread (graphics settings).
pub fn install() {
    use crate::fsr4::State;
    if matches!(state(), State::Downloading(..)) {
        return;
    }
    set(State::Downloading(0, download_size()));
    std::thread::spawn(|| {
        let r = dir().ok_or_else(|| "no data folder".to_string()).and_then(|d| crate::fsr4::fetch(&FILES, d, &notice(), &|done, total| set(State::Downloading(done, total))));
        set(match r {
            Ok(()) => State::Installed,
            Err(e) => {
                log::warn!("frame generation download failed: {e}");
                State::Failed(e)
            }
        });
    });
}

/// checks the hashes before code is loaded from the folder.
pub fn verify() -> Result<PathBuf, String> {
    crate::fsr4::check(&FILES, dir().ok_or("no data folder")?)
}

fn notice() -> String {
    format!(
        "AMD FSR frame generation runtime, from AMD's FSR SDK v{0}:\n\
         https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/releases/tag/v{0}\n\n\
         Downloaded by FableFlight at the player's request; not part of FableFlight\n\
         and not covered by its licence. Copyright (C) Advanced Micro Devices, Inc.\n\
         The terms are in license.md, next to this file. Delete this folder to remove it.\n",
        crate::fsr4::SDK
    )
}

/// whether the device should be opened with amd's queues: windows, vulkan, runtime there.
pub fn wanted(adapter: &wgpu::AdapterInfo) -> bool {
    cfg!(windows) && adapter.backend == wgpu::Backend::Vulkan && state() == crate::fsr4::State::Installed && std::env::var_os("FABLE_NO_FRAMEGEN").is_none()
}

/// which queue family and index each of amd's queues uses.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Plan {
    pub present: (u32, u32),
    pub compute: (u32, u32),
    pub acquire: (u32, u32),
}

/// the gpu's queue families, and which can show on the window.
#[cfg(any(windows, target_os = "linux"))]
#[derive(Clone, Debug)]
pub struct Families {
    pub flags: Vec<ash::vk::QueueFlags>,
    pub counts: Vec<u32>,
    pub present: Vec<bool>,
}

/// no vulkan here (macos): no queue families to read.
#[cfg(not(any(windows, target_os = "linux")))]
#[derive(Clone, Debug)]
pub struct Families;

/// no vulkan here: never any families.
#[cfg(not(any(windows, target_os = "linux")))]
pub fn families(_: &wgpu::Adapter, _: &wgpu::Surface) -> Option<Families> {
    None
}

/// reads the queue families before the device is opened.
#[cfg(any(windows, target_os = "linux"))]
pub fn families(adapter: &wgpu::Adapter, surface: &wgpu::Surface) -> Option<Families> {
    use wgpu::hal::api::Vulkan;
    unsafe {
        let hal = adapter.as_hal::<Vulkan>()?;
        let shared = hal.shared_instance();
        let pd = hal.raw_physical_device();
        let props = shared.raw_instance().get_physical_device_queue_family_properties(pd);
        let raw_surface = surface.as_hal::<Vulkan>()?.raw_native_handle()?;
        let si = ash::khr::surface::Instance::new(shared.entry(), shared.raw_instance());
        let present = (0..props.len() as u32).map(|i| si.get_physical_device_surface_support(pd, i, raw_surface).unwrap_or(false)).collect();
        Some(Families { flags: props.iter().map(|p| p.queue_flags).collect(), counts: props.iter().map(|p| p.queue_count).collect(), present })
    }
}

/// priorities for every queue asked for (vulkan reads them while the device is made).
#[cfg(any(windows, target_os = "linux"))]
static PRIO: [f32; 32] = [1.0; 32];

/// picks amd's queues the way amd's own samples do and asks for them alongside wgpu's.
/// all four must be different queues; none if the gpu cannot give that.
#[cfg(any(windows, target_os = "linux"))]
pub fn add_queues(f: &Families, infos: &mut Vec<ash::vk::DeviceQueueCreateInfo<'_>>) -> Option<Plan> {
    let plan = plan(f, infos.first()?.queue_family_index)?;
    let mut extra = vec![0u32; f.counts.len()];
    for (fam, _) in [plan.present, plan.compute, plan.acquire] {
        extra[fam as usize] += 1;
    }
    for (fam, n) in extra.iter().enumerate().filter(|(_, n)| **n > 0) {
        let fam = fam as u32;
        if let Some(i) = infos.iter().position(|q| q.queue_family_index == fam) {
            let have = infos[i].queue_count;
            infos[i] = infos[i].queue_priorities(&PRIO[..(have + n) as usize]);
        } else {
            infos.push(ash::vk::DeviceQueueCreateInfo::default().queue_family_index(fam).queue_priorities(&PRIO[..*n as usize]));
        }
    }
    log::info!("frame generation queues: {plan:?}");
    Some(plan)
}

/// the plan for wgpu's queue (family `game`, index 0).
#[cfg(any(windows, target_os = "linux"))]
pub fn plan(f: &Families, game: u32) -> Option<Plan> {
    use ash::vk::QueueFlags as Q;
    let mut used = vec![0u32; f.counts.len()];
    *used.get_mut(game as usize)? = 1;
    let mut take = |ok: &dyn Fn(usize) -> bool| -> Option<(u32, u32)> {
        let i = (0..f.counts.len()).find(|&i| ok(i) && used[i] < f.counts[i])?;
        used[i] += 1;
        Some((i as u32, used[i] - 1))
    };
    let gc = Q::GRAPHICS | Q::COMPUTE;
    let present = take(&|i| f.flags[i].contains(gc) && f.present[i]).or_else(|| take(&|i| f.present[i] && f.flags[i].intersects(gc | Q::TRANSFER)))?;
    let acquire = take(&|i| !f.flags[i].intersects(gc) && f.flags[i].contains(Q::TRANSFER)).or_else(|| take(&|i| !f.flags[i].contains(Q::GRAPHICS) && f.flags[i].intersects(Q::COMPUTE | Q::TRANSFER))).or_else(|| take(&|i| f.flags[i].intersects(gc | Q::TRANSFER)))?;
    let compute = take(&|i| f.flags[i].contains(Q::COMPUTE) && !f.flags[i].contains(Q::GRAPHICS)).or_else(|| take(&|i| f.flags[i].contains(Q::COMPUTE)))?;
    Some(Plan { present, compute, acquire })
}

#[cfg(windows)]
pub use rt::{FrameGen, Prep};

#[cfg(windows)]
mod rt {
    use super::Plan;
    use ash::vk;
    use std::ffi::c_void;
    use wgpu::hal::api::Vulkan;

    #[repr(C)]
    struct Header {
        ty: u64,
        next: *mut Header,
    }
    impl Header {
        fn new(ty: u64) -> Self {
            Header { ty, next: std::ptr::null_mut() }
        }
    }
    #[repr(C)]
    #[derive(Clone, Copy, Default)]
    struct Dim {
        width: u32,
        height: u32,
    }
    #[repr(C)]
    #[derive(Clone, Copy, Default)]
    struct Float2 {
        x: f32,
        y: f32,
    }
    #[repr(C)]
    #[derive(Clone, Copy, Default)]
    struct Rect {
        left: i32,
        top: i32,
        width: i32,
        height: i32,
    }
    #[repr(C)]
    #[derive(Clone, Copy)]
    struct ResourceDesc {
        ty: u32,
        format: u32,
        width: u32,
        height: u32,
        depth: u32,
        mip_count: u32,
        flags: u32,
        usage: u32,
    }
    #[repr(C)]
    #[derive(Clone, Copy)]
    struct Resource {
        resource: *mut c_void,
        desc: ResourceDesc,
        state: u32,
    }
    impl Resource {
        fn null() -> Self {
            Resource { resource: std::ptr::null_mut(), desc: ResourceDesc { ty: 0, format: 0, width: 0, height: 0, depth: 0, mip_count: 0, flags: 0, usage: 0 }, state: 0 }
        }
        fn image(image: vk::Image, format: u32, (w, h): (u32, u32), usage: u32, state: u32) -> Self {
            Resource { resource: vk::Handle::as_raw(image) as *mut c_void, desc: ResourceDesc { ty: TEX2D, format, width: w, height: h, depth: 1, mip_count: 1, flags: 0, usage }, state }
        }
    }
    #[repr(C)]
    struct QueueInfo {
        queue: vk::Queue,
        family: u32,
        submit: Option<unsafe extern "C" fn(u32, *const vk::SubmitInfo, vk::Fence) -> vk::Result>,
    }
    #[repr(C)]
    struct CreateSwapchain<'a> {
        header: Header,
        physical: vk::PhysicalDevice,
        device: vk::Device,
        swapchain: *mut vk::SwapchainKHR,
        allocator: *const c_void,
        info: vk::SwapchainCreateInfoKHR<'a>,
        game: QueueInfo,
        compute: QueueInfo,
        present: QueueInfo,
        acquire: QueueInfo,
    }
    #[repr(C)]
    struct Version {
        header: Header,
        version: u32,
    }
    type CreateSwapchainFn = unsafe extern "C" fn(vk::Device, *const vk::SwapchainCreateInfoKHR, *const c_void, *mut vk::SwapchainKHR, *mut c_void) -> vk::Result;
    type DestroySwapchainFn = unsafe extern "C" fn(vk::Device, vk::SwapchainKHR, *const c_void, *mut c_void);
    #[repr(C)]
    struct QueryFns {
        header: Header,
        create: Option<CreateSwapchainFn>,
        destroy: Option<DestroySwapchainFn>,
        images: Option<vk::PFN_vkGetSwapchainImagesKHR>,
        acquire: Option<vk::PFN_vkAcquireNextImageKHR>,
        present: Option<vk::PFN_vkQueuePresentKHR>,
        hdr: Option<vk::PFN_vkSetHdrMetadataEXT>,
        count: Option<unsafe extern "C" fn(vk::SwapchainKHR) -> u64>,
    }
    #[repr(C)]
    struct DeviceCtx {
        device: vk::Device,
        physical: vk::PhysicalDevice,
        gdpa: vk::PFN_vkGetDeviceProcAddr,
    }
    #[repr(C)]
    struct Backend {
        header: Header,
        ctx: DeviceCtx,
    }
    #[repr(C)]
    struct CreateFg {
        header: Header,
        flags: u32,
        display: Dim,
        max_render: Dim,
        format: u32,
    }
    #[repr(C)]
    struct ConfigureFg {
        header: Header,
        swapchain: *mut c_void,
        present_cb: *const c_void,
        present_ctx: *mut c_void,
        fg_cb: Option<unsafe extern "C" fn(*mut Header, *mut c_void) -> u32>,
        fg_ctx: *mut c_void,
        enabled: bool,
        allow_async: bool,
        hudless: Resource,
        flags: u32,
        only_generated: bool,
        rect: Rect,
        frame_id: u64,
    }
    #[repr(C)]
    struct PrepareV2 {
        header: Header,
        frame_id: u64,
        flags: u32,
        cmd: *mut c_void,
        render: Dim,
        jitter: Float2,
        mv_scale: Float2,
        dt_ms: f32,
        reset: bool,
        near: f32,
        far: f32,
        fov_y: f32,
        to_meters: f32,
        depth: Resource,
        motion: Resource,
        cam_pos: [f32; 3],
        cam_up: [f32; 3],
        cam_right: [f32; 3],
        cam_fwd: [f32; 3],
    }
    #[repr(C)]
    struct KeyValue {
        header: Header,
        key: u64,
        u64: u64,
        ptr: *mut c_void,
    }
    #[repr(C)]
    struct PacingTuning {
        safety_ms: f32,
        variance: f32,
        hybrid_spin: bool,
        spin_time: u32,
        wait_on_fence: bool,
    }
    #[repr(C)]
    struct RegisterUi {
        header: Header,
        ui: Resource,
        flags: u32,
    }

    const SC_CREATE: u64 = 0x0004_0001;
    const SC_REGISTER_UI: u64 = 0x0004_0002;
    const SC_FUNCTIONS: u64 = 0x0004_0005;
    const SC_WAIT_PRESENTS: u64 = 0x0004_0007;
    const SC_KEY_VALUE: u64 = 0x0004_0008;
    const KEY_PACING: u64 = 2;
    const SC_VERSION: u64 = 0x0004_000a;
    const SC_API_VERSION: u32 = 4 << 22;
    const FG_CREATE: u64 = 0x0002_0001;
    const FG_CONFIGURE: u64 = 0x0002_0002;
    const FG_PREPARE_V2: u64 = 0x0002_000c;
    const FG_VERSION: u64 = 0x0002_000e;
    const FG_API_VERSION: u32 = (4 << 22) | 2;
    const BACKEND_VK: u64 = 0x0100_0003;
    const DEPTH_INVERTED: u32 = 1 << 3;
    const DEPTH_INFINITE: u32 = 1 << 4;
    const UI_PREMUL: u32 = 1 << 0;
    const UI_DOUBLE_BUFFER: u32 = 1 << 1;
    const TEX2D: u32 = 2;
    const FMT_RGBA8: u32 = 10;
    const FMT_BGRA8: u32 = 14;
    const FMT_RG16F: u32 = 18;
    const FMT_R32F: u32 = 28;
    const USAGE_READ: u32 = 0;
    const USAGE_DEPTH: u32 = 1 << 2;
    const STATE_PIXEL_COMPUTE_READ: u32 = (1 << 3) | (1 << 2);
    /// frames in flight for the final copy.
    const SLOTS: usize = 3;

    type Ctx = *mut c_void;
    type FnCreate = unsafe extern "C" fn(*mut Ctx, *mut Header, *const c_void) -> u32;
    type FnDestroy = unsafe extern "C" fn(*mut Ctx, *const c_void) -> u32;
    type FnConfigure = unsafe extern "C" fn(*mut Ctx, *const Header) -> u32;
    type FnQuery = unsafe extern "C" fn(*mut Ctx, *mut Header) -> u32;
    type FnDispatch = unsafe extern "C" fn(*mut Ctx, *const Header) -> u32;

    /// what amd's swapchain calls back into to make the generated frame.
    struct Callback {
        dispatch: FnDispatch,
        ctx: Ctx,
    }

    unsafe extern "C" fn fg_dispatch(params: *mut Header, user: *mut c_void) -> u32 {
        let c = unsafe { &mut *(user as *mut Callback) };
        unsafe { (c.dispatch)(&mut c.ctx, params) }
    }

    struct Slot {
        cmd: vk::CommandBuffer,
        fence: vk::Fence,
        acquired: vk::Semaphore,
        done: vk::Semaphore,
    }

    /// what the prepare pass is told about this frame.
    pub struct Prep {
        pub render: (u32, u32),
        pub jitter: [f32; 2],
        pub dt: f32,
        pub reset: bool,
        pub near: f32,
        pub fov_y: f32,
        /// camera right, up and forward in the world.
        pub basis: [[f32; 3]; 3],
    }

    /// amd's runtime, its swapchain and frame generation contexts, and the final copy.
    pub struct FrameGen {
        _lib: libloading::Library,
        destroy: FnDestroy,
        configure: FnConfigure,
        dispatch: FnDispatch,
        sc_ctx: Ctx,
        cb: Box<Callback>,
        fns: QueryFns,
        device: ash::Device,
        physical: vk::PhysicalDevice,
        gdpa: vk::PFN_vkGetDeviceProcAddr,
        game: vk::Queue,
        surface: vk::SurfaceKHR,
        info: vk::SwapchainCreateInfoKHR<'static>,
        swapchain: vk::SwapchainKHR,
        images: Vec<vk::Image>,
        pool: vk::CommandPool,
        slots: Vec<Slot>,
        slot: usize,
        image: Option<u32>,
        frame_id: u64,
        /// the swapchain went out of date: remake it before the next frame.
        pub stale: bool,
        /// the ui is drawn here and laid over every shown frame by amd's swapchain.
        pub ui: wgpu::Texture,
        pub ui_view: wgpu::TextureView,
        pub provider: String,
    }

    // only touched from the render thread.
    unsafe impl Send for FrameGen {}

    fn ffx_format(f: wgpu::TextureFormat) -> Option<(vk::Format, u32)> {
        match f {
            wgpu::TextureFormat::Bgra8Unorm => Some((vk::Format::B8G8R8A8_UNORM, FMT_BGRA8)),
            wgpu::TextureFormat::Rgba8Unorm => Some((vk::Format::R8G8B8A8_UNORM, FMT_RGBA8)),
            _ => None,
        }
    }

    impl FrameGen {
        /// loads the runtime and puts amd's swapchain on the window's surface. wgpu's own
        /// surface must hold no swapchain. `format` is what the finished frame is drawn in.
        pub fn new(device: &wgpu::Device, surface: &wgpu::Surface, plan: &Plan, size: (u32, u32), format: wgpu::TextureFormat, vsync: bool) -> Result<Self, String> {
            let dir = super::verify()?;
            let (vk_format, ffx_fmt) = ffx_format(format).ok_or_else(|| format!("{format:?} frames"))?;
            unsafe {
                let lib = libloading::Library::new(dir.join("amd_fidelityfx_loader_vk.dll")).map_err(|e| e.to_string())?;
                let create: FnCreate = *lib.get::<FnCreate>(b"ffxCreateContext\0").map_err(|e| e.to_string())?;
                let destroy: FnDestroy = *lib.get::<FnDestroy>(b"ffxDestroyContext\0").map_err(|e| e.to_string())?;
                let configure: FnConfigure = *lib.get::<FnConfigure>(b"ffxConfigure\0").map_err(|e| e.to_string())?;
                let query: FnQuery = *lib.get::<FnQuery>(b"ffxQuery\0").map_err(|e| e.to_string())?;
                let dispatch: FnDispatch = *lib.get::<FnDispatch>(b"ffxDispatch\0").map_err(|e| e.to_string())?;
                let hal = device.as_hal::<Vulkan>().ok_or("not a Vulkan device")?;
                let shared = hal.shared_instance();
                let raw = hal.raw_device().clone();
                let physical = hal.raw_physical_device();
                let gdpa = shared.raw_instance().fp_v1_0().get_device_proc_addr;
                let game = hal.raw_queue();
                let family = hal.queue_family_index();
                let q = |(f, i): (u32, u32)| QueueInfo { queue: raw.get_device_queue(f, i), family: f, submit: None };
                let raw_surface = surface.as_hal::<Vulkan>().and_then(|s| s.raw_native_handle()).ok_or("no Vulkan surface")?;
                let si = ash::khr::surface::Instance::new(shared.entry(), shared.raw_instance());
                if !si.get_physical_device_surface_formats(physical, raw_surface).map_err(|e| e.to_string())?.iter().any(|f| f.format == vk_format) {
                    return Err(format!("the window cannot show {vk_format:?}"));
                }
                let info = swapchain_info(&si, physical, raw_surface, size, vk_format, vsync)?;
                let mut swapchain = vk::SwapchainKHR::null();
                let mut version = Version { header: Header::new(SC_VERSION), version: SC_API_VERSION };
                let mut desc = CreateSwapchain {
                    header: Header { ty: SC_CREATE, next: &mut version.header },
                    physical,
                    device: raw.handle(),
                    swapchain: &mut swapchain,
                    allocator: std::ptr::null(),
                    info,
                    game: QueueInfo { queue: game, family, submit: None },
                    compute: q(plan.compute),
                    present: q(plan.present),
                    acquire: q(plan.acquire),
                };
                let mut sc_ctx: Ctx = std::ptr::null_mut();
                let r = create(&mut sc_ctx, &mut desc.header, std::ptr::null());
                if r != 0 || sc_ctx.is_null() || swapchain == vk::SwapchainKHR::null() {
                    return Err(format!("amd's swapchain would not start ({r})"));
                }
                let mut fns = QueryFns { header: Header::new(SC_FUNCTIONS), create: None, destroy: None, images: None, acquire: None, present: None, hdr: None, count: None };
                let r = query(&mut sc_ctx, &mut fns.header);
                if r != 0 || fns.images.is_none() || fns.acquire.is_none() || fns.present.is_none() || fns.destroy.is_none() || fns.create.is_none() {
                    destroy(&mut sc_ctx, std::ptr::null());
                    return Err(format!("amd's swapchain functions ({r})"));
                }
                let pool = raw.create_command_pool(&vk::CommandPoolCreateInfo::default().queue_family_index(family).flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER), None).map_err(|e| e.to_string())?;
                let cmds = raw.allocate_command_buffers(&vk::CommandBufferAllocateInfo::default().command_pool(pool).level(vk::CommandBufferLevel::PRIMARY).command_buffer_count(SLOTS as u32)).map_err(|e| e.to_string())?;
                let slots = cmds
                    .into_iter()
                    .map(|cmd| Slot {
                        cmd,
                        fence: raw.create_fence(&vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED), None).unwrap_or_default(),
                        acquired: raw.create_semaphore(&vk::SemaphoreCreateInfo::default(), None).unwrap_or_default(),
                        done: raw.create_semaphore(&vk::SemaphoreCreateInfo::default(), None).unwrap_or_default(),
                    })
                    .collect();
                let (ui, ui_view) = ui_target(device, size, format);
                drop(hal);
                let mut fg = FrameGen {
                    _lib: lib,
                    destroy,
                    configure,
                    dispatch,
                    sc_ctx,
                    cb: Box::new(Callback { dispatch, ctx: std::ptr::null_mut() }),
                    fns,
                    device: raw,
                    physical,
                    gdpa,
                    game,
                    surface: raw_surface,
                    info: vk::SwapchainCreateInfoKHR { old_swapchain: vk::SwapchainKHR::null(), ..info },
                    swapchain,
                    images: Vec::new(),
                    pool,
                    slots,
                    slot: 0,
                    image: None,
                    frame_id: 0,
                    stale: false,
                    ui,
                    ui_view,
                    provider: String::new(),
                };
                fg.images = fg.swapchain_images()?;
                fg.tune_pacing();
                fg.make_fg(create, query, size, ffx_fmt)?;
                log::info!("AMD frame generation on: {}x{}, {} images, {:?}, provider {}", size.0, size.1, fg.images.len(), fg.info.present_mode, fg.provider);
                Ok(fg)
            }
        }

        unsafe fn swapchain_images(&self) -> Result<Vec<vk::Image>, String> {
            let f = self.fns.images.ok_or("no images function")?;
            let mut n = 0u32;
            unsafe {
                f(self.device.handle(), self.swapchain, &mut n, std::ptr::null_mut()).result().map_err(|e| e.to_string())?;
                let mut v = vec![vk::Image::null(); n as usize];
                f(self.device.handle(), self.swapchain, &mut n, v.as_mut_ptr()).result().map_err(|e| e.to_string())?;
                v.truncate(n as usize);
                Ok(v)
            }
        }

        /// the frame generation context for the display size.
        unsafe fn make_fg(&mut self, create: FnCreate, query: FnQuery, size: (u32, u32), fmt: u32) -> Result<(), String> {
            unsafe {
                if !self.cb.ctx.is_null() {
                    (self.destroy)(&mut self.cb.ctx, std::ptr::null());
                    self.cb.ctx = std::ptr::null_mut();
                }
                let mut backend = Backend { header: Header::new(BACKEND_VK), ctx: DeviceCtx { device: self.device.handle(), physical: self.physical, gdpa: self.gdpa } };
                let mut version = Version { header: Header { ty: FG_VERSION, next: &mut backend.header }, version: FG_API_VERSION };
                let mut desc = CreateFg {
                    header: Header { ty: FG_CREATE, next: &mut version.header },
                    flags: DEPTH_INVERTED | DEPTH_INFINITE,
                    display: Dim { width: size.0, height: size.1 },
                    // the render resolution changes with the settings and dynamic resolution.
                    max_render: Dim { width: size.0, height: size.1 },
                    format: fmt,
                };
                let r = create(&mut self.cb.ctx, &mut desc.header, std::ptr::null());
                if r != 0 || self.cb.ctx.is_null() {
                    self.cb.ctx = std::ptr::null_mut();
                    return Err(format!("amd's frame generation would not start ({r})"));
                }
                #[repr(C)]
                struct QueryProvider {
                    header: Header,
                    id: u64,
                    name: *const std::ffi::c_char,
                }
                let mut v = QueryProvider { header: Header::new(6), id: 0, name: std::ptr::null() };
                self.provider = if query(&mut self.cb.ctx, &mut v.header) == 0 && !v.name.is_null() { std::ffi::CStr::from_ptr(v.name).to_string_lossy().into_owned() } else { "unknown".into() };
                Ok(())
            }
        }

        /// frames amd's swapchain has shown, generated ones too.
        pub fn presents(&self) -> u64 {
            self.fns.count.map_or(0, |f| unsafe { f(self.swapchain) })
        }

        /// the display size the swapchain was made for.
        pub fn size(&self) -> (u32, u32) {
            (self.info.image_extent.width, self.info.image_extent.height)
        }

        /// a new window size or vsync: the swapchain is remade in place and the
        /// frame generation context for the new size.
        pub fn resize(&mut self, device: &wgpu::Device, size: (u32, u32), format: wgpu::TextureFormat, vsync: bool) -> Result<(), String> {
            let (vk_format, ffx_fmt) = ffx_format(format).ok_or("format")?;
            unsafe {
                self.wait_presents();
                let _ = self.device.device_wait_idle();
                let hal = device.as_hal::<Vulkan>().ok_or("not a Vulkan device")?;
                let shared = hal.shared_instance();
                let si = ash::khr::surface::Instance::new(shared.entry(), shared.raw_instance());
                let info = swapchain_info(&si, self.physical, self.surface, size, vk_format, vsync)?;
                drop(hal);
                // remade in place: the context argument is amd's swapchain itself (its handle
                // is the proxy object), and there is no old swapchain to hand over.
                let create = self.fns.create.ok_or("no create function")?;
                let mut sc = self.swapchain;
                let proxy = vk::Handle::as_raw(self.swapchain) as *mut c_void;
                let r = create(self.device.handle(), &info, std::ptr::null(), &mut sc, proxy);
                if r != vk::Result::SUCCESS {
                    return Err(format!("amd's swapchain would not resize ({r:?})"));
                }
                self.swapchain = sc;
                self.info = info;
                self.images = self.swapchain_images()?;
                self.tune_pacing();
                let lib = &self._lib;
                let create_ctx: FnCreate = *lib.get::<FnCreate>(b"ffxCreateContext\0").map_err(|e| e.to_string())?;
                let query: FnQuery = *lib.get::<FnQuery>(b"ffxQuery\0").map_err(|e| e.to_string())?;
                self.make_fg(create_ctx, query, size, ffx_fmt)?;
                (self.ui, self.ui_view) = ui_target(device, size, format);
                // a new size starts the frame count over.
                self.frame_id = 0;
                self.stale = false;
                Ok(())
            }
        }

        /// amd's tuning set b: the default pacer gets stuck at a slow frame rate after a
        /// heavy stretch, as it paces to an average its own pacing holds up (ffx_api_types.h).
        fn tune_pacing(&mut self) {
            let mut tune = PacingTuning { safety_ms: 0.01, variance: 0.3, hybrid_spin: false, spin_time: 2, wait_on_fence: false };
            let kv = KeyValue { header: Header::new(SC_KEY_VALUE), key: KEY_PACING, u64: 0, ptr: &mut tune as *mut PacingTuning as *mut c_void };
            let r = unsafe { (self.configure)(&mut self.sc_ctx, &kv.header) };
            if r != 0 {
                log::warn!("frame generation pacing tuning not taken ({r})");
            }
        }

        fn wait_presents(&mut self) {
            let h = Header::new(SC_WAIT_PRESENTS);
            unsafe {
                (self.dispatch)(&mut self.sc_ctx, &h);
            }
        }

        /// this frame's inputs: amd's settings, then its prepare pass on the depth and
        /// motion vectors, recorded into a command buffer of its own and submitted. depth
        /// and motion must be where wgpu left them: depth read only, motion sampled.
        pub fn prepare(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, depth: &wgpu::Texture, motion: &wgpu::Texture, p: &Prep) -> Result<(), String> {
            let img = |t: &wgpu::Texture| unsafe { t.as_hal::<Vulkan>().map(|h| h.raw_handle()) };
            let (depth_img, motion_img) = (img(depth).ok_or("depth")?, img(motion).ok_or("motion")?);
            let (w, h) = self.size();
            let cfg = ConfigureFg {
                header: Header::new(FG_CONFIGURE),
                swapchain: vk::Handle::as_raw(self.swapchain) as *mut c_void,
                present_cb: std::ptr::null(),
                present_ctx: std::ptr::null_mut(),
                fg_cb: Some(fg_dispatch),
                fg_ctx: &mut *self.cb as *mut Callback as *mut c_void,
                enabled: true,
                allow_async: false,
                hudless: Resource::null(),
                flags: 0,
                only_generated: false,
                rect: Rect { left: 0, top: 0, width: w as i32, height: h as i32 },
                frame_id: self.frame_id,
            };
            let r = unsafe { (self.configure)(&mut self.cb.ctx, &cfg.header) };
            if r != 0 {
                return Err(format!("frame generation settings ({r})"));
            }
            let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("AMD frame generation prepare") });
            let mut err = None;
            unsafe {
                let raw_dev = self.device.clone();
                let ctx = &mut self.cb.ctx;
                let dispatch = self.dispatch;
                let frame_id = self.frame_id;
                enc.as_hal_mut::<Vulkan, _, _>(|e| {
                    let Some(e) = e else {
                        err = Some("no command buffer".to_string());
                        return;
                    };
                    let cmd = e.raw_handle();
                    let depth_barrier = |old: vk::ImageLayout, new: vk::ImageLayout| {
                        let b = vk::ImageMemoryBarrier::default()
                            .src_access_mask(vk::AccessFlags::SHADER_READ)
                            .dst_access_mask(vk::AccessFlags::SHADER_READ)
                            .old_layout(old)
                            .new_layout(new)
                            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                            .image(depth_img)
                            .subresource_range(vk::ImageSubresourceRange { aspect_mask: vk::ImageAspectFlags::DEPTH, base_mip_level: 0, level_count: 1, base_array_layer: 0, layer_count: 1 });
                        raw_dev.cmd_pipeline_barrier(cmd, vk::PipelineStageFlags::ALL_COMMANDS, vk::PipelineStageFlags::ALL_COMMANDS, vk::DependencyFlags::empty(), &[], &[], &[b]);
                    };
                    depth_barrier(vk::ImageLayout::DEPTH_STENCIL_READ_ONLY_OPTIMAL, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL);
                    let [right, up, fwd] = p.basis;
                    let d = PrepareV2 {
                        header: Header::new(FG_PREPARE_V2),
                        frame_id,
                        flags: 0,
                        cmd: vk::Handle::as_raw(cmd) as *mut c_void,
                        render: Dim { width: p.render.0, height: p.render.1 },
                        jitter: Float2 { x: p.jitter[0], y: p.jitter[1] },
                        // our motion vectors are uv offsets; amd's are scaled to pixels by this.
                        mv_scale: Float2 { x: p.render.0 as f32, y: p.render.1 as f32 },
                        dt_ms: p.dt * 1000.0,
                        reset: p.reset,
                        near: p.near,
                        far: 1.0e7,
                        fov_y: p.fov_y,
                        to_meters: 1.0,
                        depth: Resource::image(depth_img, FMT_R32F, p.render, USAGE_DEPTH, STATE_PIXEL_COMPUTE_READ),
                        motion: Resource::image(motion_img, FMT_RG16F, p.render, USAGE_READ, STATE_PIXEL_COMPUTE_READ),
                        // drawn around the camera, so it sits at the origin.
                        cam_pos: [0.0; 3],
                        cam_up: up,
                        cam_right: right,
                        cam_fwd: fwd,
                    };
                    let r = dispatch(ctx, &d.header);
                    depth_barrier(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL, vk::ImageLayout::DEPTH_STENCIL_READ_ONLY_OPTIMAL);
                    if r != 0 {
                        err = Some(format!("frame generation prepare ({r})"));
                    }
                });
            }
            queue.submit([enc.finish()]);
            err.map_or(Ok(()), Err)
        }

        /// hands amd's swapchain the ui for this frame (sampled, in `ui`).
        pub fn register_ui(&mut self, format: wgpu::TextureFormat) {
            let Some(img) = (unsafe { self.ui.as_hal::<Vulkan>().map(|h| h.raw_handle()) }) else { return };
            let fmt = ffx_format(format).map_or(FMT_BGRA8, |f| f.1);
            let d = RegisterUi { header: Header::new(SC_REGISTER_UI), ui: Resource::image(img, fmt, self.size(), USAGE_READ, STATE_PIXEL_COMPUTE_READ), flags: UI_PREMUL | UI_DOUBLE_BUFFER };
            unsafe {
                (self.configure)(&mut self.sc_ctx, &d.header);
            }
        }

        /// copies the finished frame (`src`, left by wgpu as a copy source and the
        /// window's size) into the next image and presents it through amd's swapchain.
        /// runs after wgpu's last submit of the frame, on wgpu's queue, on this thread.
        pub fn present(&mut self, src: &wgpu::Texture) -> Result<(), String> {
            let src_img = unsafe { src.as_hal::<Vulkan>().map(|h| h.raw_handle()) }.ok_or("frame")?;
            let s = &self.slots[self.slot];
            let d = &self.device;
            unsafe {
                d.wait_for_fences(&[s.fence], true, 2_000_000_000).map_err(|e| format!("copy fence: {e}"))?;
                let acquire = self.fns.acquire.ok_or("no acquire function")?;
                let mut idx = 0u32;
                let r = acquire(d.handle(), self.swapchain, 2_000_000_000, s.acquired, vk::Fence::null(), &mut idx);
                match r {
                    vk::Result::SUCCESS => {}
                    vk::Result::SUBOPTIMAL_KHR => self.stale = true,
                    vk::Result::ERROR_OUT_OF_DATE_KHR => {
                        self.stale = true;
                        return Ok(());
                    }
                    e => return Err(format!("acquire: {e:?}")),
                }
                self.image = Some(idx);
                let dst = *self.images.get(idx as usize).ok_or("image index")?;
                d.reset_fences(&[s.fence]).map_err(|e| e.to_string())?;
                d.reset_command_buffer(s.cmd, vk::CommandBufferResetFlags::empty()).map_err(|e| e.to_string())?;
                d.begin_command_buffer(s.cmd, &vk::CommandBufferBeginInfo::default().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT)).map_err(|e| e.to_string())?;
                let range = vk::ImageSubresourceRange { aspect_mask: vk::ImageAspectFlags::COLOR, base_mip_level: 0, level_count: 1, base_array_layer: 0, layer_count: 1 };
                let barrier = |img, old, new, src_access, dst_access| vk::ImageMemoryBarrier::default().src_access_mask(src_access).dst_access_mask(dst_access).old_layout(old).new_layout(new).src_queue_family_index(vk::QUEUE_FAMILY_IGNORED).dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED).image(img).subresource_range(range);
                // (wgpu's earlier work on this queue finishes writing the frame first)
                d.cmd_pipeline_barrier(
                    s.cmd,
                    vk::PipelineStageFlags::ALL_COMMANDS,
                    vk::PipelineStageFlags::TRANSFER,
                    vk::DependencyFlags::empty(),
                    &[],
                    &[],
                    &[
                        barrier(src_img, vk::ImageLayout::TRANSFER_SRC_OPTIMAL, vk::ImageLayout::TRANSFER_SRC_OPTIMAL, vk::AccessFlags::MEMORY_WRITE, vk::AccessFlags::TRANSFER_READ),
                        barrier(dst, vk::ImageLayout::UNDEFINED, vk::ImageLayout::TRANSFER_DST_OPTIMAL, vk::AccessFlags::empty(), vk::AccessFlags::TRANSFER_WRITE),
                    ],
                );
                let (w, h) = self.size();
                let layers = vk::ImageSubresourceLayers { aspect_mask: vk::ImageAspectFlags::COLOR, mip_level: 0, base_array_layer: 0, layer_count: 1 };
                let ext = vk::Extent3D { width: w.min(src.width()), height: h.min(src.height()), depth: 1 };
                d.cmd_copy_image(s.cmd, src_img, vk::ImageLayout::TRANSFER_SRC_OPTIMAL, dst, vk::ImageLayout::TRANSFER_DST_OPTIMAL, &[vk::ImageCopy { src_subresource: layers, src_offset: vk::Offset3D::default(), dst_subresource: layers, dst_offset: vk::Offset3D::default(), extent: ext }]);
                d.cmd_pipeline_barrier(s.cmd, vk::PipelineStageFlags::TRANSFER, vk::PipelineStageFlags::BOTTOM_OF_PIPE, vk::DependencyFlags::empty(), &[], &[], &[barrier(dst, vk::ImageLayout::TRANSFER_DST_OPTIMAL, vk::ImageLayout::PRESENT_SRC_KHR, vk::AccessFlags::TRANSFER_WRITE, vk::AccessFlags::empty())]);
                d.end_command_buffer(s.cmd).map_err(|e| e.to_string())?;
                let waits = [s.acquired];
                let stages = [vk::PipelineStageFlags::TRANSFER];
                let cmds = [s.cmd];
                let signals = [s.done];
                let submit = vk::SubmitInfo::default().wait_semaphores(&waits).wait_dst_stage_mask(&stages).command_buffers(&cmds).signal_semaphores(&signals);
                d.queue_submit(self.game, &[submit], s.fence).map_err(|e| format!("copy submit: {e}"))?;
                let present = self.fns.present.ok_or("no present function")?;
                let swapchains = [self.swapchain];
                let indices = [idx];
                let info = vk::PresentInfoKHR::default().wait_semaphores(&signals).swapchains(&swapchains).image_indices(&indices);
                let r = present(self.game, &info);
                self.slot = (self.slot + 1) % SLOTS;
                self.frame_id += 1;
                self.image = None;
                match r {
                    vk::Result::SUCCESS => Ok(()),
                    vk::Result::SUBOPTIMAL_KHR | vk::Result::ERROR_OUT_OF_DATE_KHR => {
                        self.stale = true;
                        Ok(())
                    }
                    e => Err(format!("present: {e:?}")),
                }
            }
        }
    }

    impl Drop for FrameGen {
        fn drop(&mut self) {
            unsafe {
                self.wait_presents();
                let _ = self.device.device_wait_idle();
                if !self.cb.ctx.is_null() {
                    (self.destroy)(&mut self.cb.ctx, std::ptr::null());
                }
                (self.destroy)(&mut self.sc_ctx, std::ptr::null());
                if let Some(f) = self.fns.destroy {
                    f(self.device.handle(), self.swapchain, std::ptr::null(), std::ptr::null_mut());
                }
                for s in &self.slots {
                    self.device.destroy_fence(s.fence, None);
                    self.device.destroy_semaphore(s.acquired, None);
                    self.device.destroy_semaphore(s.done, None);
                }
                self.device.destroy_command_pool(self.pool, None);
            }
            log::info!("AMD frame generation off");
        }
    }

    fn ui_target(device: &wgpu::Device, (w, h): (u32, u32), format: wgpu::TextureFormat) -> (wgpu::Texture, wgpu::TextureView) {
        let t = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("frame generation ui"),
            size: wgpu::Extent3d { width: w.max(1), height: h.max(1), depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let v = t.create_view(&Default::default());
        (t, v)
    }

    /// the swapchain description: three images or more, fifo with vsync, the fastest
    /// mode the window has without.
    fn swapchain_info(si: &ash::khr::surface::Instance, pd: vk::PhysicalDevice, surface: vk::SurfaceKHR, (w, h): (u32, u32), format: vk::Format, vsync: bool) -> Result<vk::SwapchainCreateInfoKHR<'static>, String> {
        unsafe {
            let caps = si.get_physical_device_surface_capabilities(pd, surface).map_err(|e| e.to_string())?;
            let modes = si.get_physical_device_surface_present_modes(pd, surface).map_err(|e| e.to_string())?;
            let mode = if vsync { vk::PresentModeKHR::FIFO } else { [vk::PresentModeKHR::IMMEDIATE, vk::PresentModeKHR::MAILBOX].into_iter().find(|m| modes.contains(m)).unwrap_or(vk::PresentModeKHR::FIFO) };
            let mut n = caps.min_image_count.max(3);
            if caps.max_image_count > 0 {
                n = n.min(caps.max_image_count);
            }
            let extent = if caps.current_extent.width != u32::MAX { caps.current_extent } else { vk::Extent2D { width: w, height: h } };
            let alpha = [vk::CompositeAlphaFlagsKHR::OPAQUE, vk::CompositeAlphaFlagsKHR::INHERIT, vk::CompositeAlphaFlagsKHR::PRE_MULTIPLIED, vk::CompositeAlphaFlagsKHR::POST_MULTIPLIED].into_iter().find(|a| caps.supported_composite_alpha.contains(*a)).unwrap_or(vk::CompositeAlphaFlagsKHR::OPAQUE);
            Ok(vk::SwapchainCreateInfoKHR::default()
                .surface(surface)
                .min_image_count(n)
                .image_format(format)
                .image_color_space(vk::ColorSpaceKHR::SRGB_NONLINEAR)
                .image_extent(extent)
                .image_array_layers(1)
                .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::TRANSFER_SRC)
                .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
                .pre_transform(caps.current_transform)
                .composite_alpha(alpha)
                .present_mode(mode)
                .clipped(true))
        }
    }
}

/// where amd's runtime does not exist: never ready.
#[cfg(not(windows))]
pub use stub::{FrameGen, Prep};

#[cfg(not(windows))]
mod stub {
    pub struct Prep {
        pub render: (u32, u32),
        pub jitter: [f32; 2],
        pub dt: f32,
        pub reset: bool,
        pub near: f32,
        pub fov_y: f32,
        pub basis: [[f32; 3]; 3],
    }
    pub struct FrameGen {
        pub stale: bool,
        pub ui: wgpu::Texture,
        pub ui_view: wgpu::TextureView,
        pub provider: String,
    }
    impl FrameGen {
        pub fn new(_: &wgpu::Device, _: &wgpu::Surface, _: &super::Plan, _: (u32, u32), _: wgpu::TextureFormat, _: bool) -> Result<Self, String> {
            Err("AMD frame generation needs Windows".into())
        }
        pub fn size(&self) -> (u32, u32) {
            (0, 0)
        }
        pub fn presents(&self) -> u64 {
            0
        }
        pub fn resize(&mut self, _: &wgpu::Device, _: (u32, u32), _: wgpu::TextureFormat, _: bool) -> Result<(), String> {
            Err("AMD frame generation needs Windows".into())
        }
        pub fn prepare(&mut self, _: &wgpu::Device, _: &wgpu::Queue, _: &wgpu::Texture, _: &wgpu::Texture, _: &Prep) -> Result<(), String> {
            Err("AMD frame generation needs Windows".into())
        }
        pub fn register_ui(&mut self, _: wgpu::TextureFormat) {}
        pub fn present(&mut self, _: &wgpu::Texture) -> Result<(), String> {
            Err("AMD frame generation needs Windows".into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn files_are_pinned() {
        assert_eq!(download_size(), 39_184 + 36_806_928 + 65_770);
        for f in &FILES {
            assert_eq!(f.sha256.len(), 64);
            assert!(f.path.ends_with(f.name));
        }
        // the loader is the same file fsr 4 uses.
        assert_eq!(FILES[0].sha256, crate::fsr4::FILES[0].sha256);
    }
}

/// the queue picking, where there is vulkan.
#[cfg(all(test, any(windows, target_os = "linux")))]
mod queue_tests {
    use super::*;
    use ash::vk::QueueFlags as Q;

    #[test]
    fn nvidia_style_queues() {
        // graphics 16, transfer 2, compute 8, video decode 1.
        let f = Families { flags: vec![Q::GRAPHICS | Q::COMPUTE | Q::TRANSFER, Q::TRANSFER, Q::COMPUTE | Q::TRANSFER, Q::VIDEO_DECODE_KHR], counts: vec![16, 2, 8, 1], present: vec![true, false, true, false] };
        let p = plan(&f, 0).expect("plan");
        assert_eq!(p.present, (0, 1), "present shares the graphics family, its next queue");
        assert_eq!(p.acquire, (1, 0), "acquire on the transfer family");
        assert_eq!(p.compute, (2, 0), "async compute on the compute family");
    }

    #[test]
    fn amd_style_queues() {
        // one graphics queue only: present goes to a compute queue that can show.
        let f = Families { flags: vec![Q::GRAPHICS | Q::COMPUTE | Q::TRANSFER, Q::COMPUTE | Q::TRANSFER, Q::TRANSFER], counts: vec![1, 4, 2], present: vec![true, true, false] };
        let p = plan(&f, 0).expect("plan");
        let all = [(0, 0), p.present, p.compute, p.acquire];
        for i in 0..4 {
            for j in i + 1..4 {
                assert_ne!(all[i], all[j], "every queue different");
            }
        }
        assert!(f.present[p.present.0 as usize]);
    }

    #[test]
    fn too_few_queues() {
        let f = Families { flags: vec![Q::GRAPHICS | Q::COMPUTE | Q::TRANSFER], counts: vec![2], present: vec![true] };
        assert!(plan(&f, 0).is_none(), "four distinct queues are needed");
    }

    #[test]
    fn queue_infos_grow_wgpus_family() {
        let f = Families { flags: vec![Q::GRAPHICS | Q::COMPUTE | Q::TRANSFER, Q::TRANSFER, Q::COMPUTE | Q::TRANSFER], counts: vec![16, 2, 8], present: vec![true, false, true] };
        let mut infos = vec![ash::vk::DeviceQueueCreateInfo::default().queue_family_index(0).queue_priorities(&[1.0])];
        add_queues(&f, &mut infos).expect("plan");
        assert_eq!(infos[0].queue_count, 2, "wgpu's family gets the present queue too");
        assert_eq!(infos.len(), 3);
        assert!(infos.iter().all(|i| i.queue_count >= 1));
    }
}
