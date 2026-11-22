//! Frame profiling, on with FABLE_GPU_TIMES=1: GPU time per render stage
//! (timestamps between passes and, where the adapter allows, inside them)
//! and CPU time per game subsystem (`cpu`), logged every 60 frames as
//! `gpu ms: label 1.2, ...` and `cpu ms: label 0.4, ...` (averages per frame).
//! Reading the stamps waits for the GPU each frame: for profiling only.

use std::cell::{Cell, RefCell};
use std::time::Instant;

const SLOTS: u32 = 160;

fn on() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var("FABLE_GPU_TIMES").is_ok_and(|v| v == "1"))
}

pub struct GpuTimer {
    set: wgpu::QuerySet,
    resolve: wgpu::Buffer,
    read: wgpu::Buffer,
    period: f64,
    in_pass: bool,
    labels: RefCell<Vec<&'static str>>,
    acc: Vec<(&'static str, f64)>,
    frames: u32,
}

impl GpuTimer {
    /// the device features needed, if the env var asks for timing and the adapter has them.
    pub fn features(adapter: &wgpu::Adapter) -> wgpu::Features {
        let f = wgpu::Features::TIMESTAMP_QUERY | wgpu::Features::TIMESTAMP_QUERY_INSIDE_ENCODERS;
        if on() && adapter.features().contains(f) {
            f | (adapter.features() & wgpu::Features::TIMESTAMP_QUERY_INSIDE_PASSES)
        } else {
            wgpu::Features::empty()
        }
    }

    pub fn new(d: &wgpu::Device, q: &wgpu::Queue) -> Option<Self> {
        if !d.features().contains(wgpu::Features::TIMESTAMP_QUERY_INSIDE_ENCODERS) {
            return None;
        }
        let size = SLOTS as u64 * 8;
        let set = d.create_query_set(&wgpu::QuerySetDescriptor { label: Some("gpu timer"), ty: wgpu::QueryType::Timestamp, count: SLOTS });
        let resolve = d.create_buffer(&wgpu::BufferDescriptor { label: None, size, usage: wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC, mapped_at_creation: false });
        let read = d.create_buffer(&wgpu::BufferDescriptor { label: None, size, usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ, mapped_at_creation: false });
        let in_pass = d.features().contains(wgpu::Features::TIMESTAMP_QUERY_INSIDE_PASSES);
        log::info!("gpu stage timing on (inside passes: {in_pass})");
        Some(Self { set, resolve, read, period: q.get_timestamp_period() as f64, in_pass, labels: RefCell::new(Vec::new()), acc: Vec::new(), frames: 0 })
    }

    fn slot(&self, label: &'static str) -> Option<u32> {
        let mut l = self.labels.borrow_mut();
        ((l.len() as u32) < SLOTS).then(|| {
            l.push(label);
            l.len() as u32 - 1
        })
    }

    /// stamp the end of a stage.
    pub fn mark(&self, enc: &mut wgpu::CommandEncoder, label: &'static str) {
        if let Some(i) = self.slot(label) {
            enc.write_timestamp(&self.set, i);
        }
    }

    /// stamp the end of a stage inside a render pass (nothing where the adapter cannot).
    pub fn mark_rp(&self, rp: &mut wgpu::RenderPass, label: &'static str) {
        if self.in_pass {
            if let Some(i) = self.slot(label) {
                rp.write_timestamp(&self.set, i);
            }
        }
    }

    /// copy the stamps out before submit.
    pub fn resolve(&mut self, enc: &mut wgpu::CommandEncoder) {
        let n = self.labels.borrow().len() as u32;
        if n > 1 {
            enc.resolve_query_set(&self.set, 0..n, &self.resolve, 0);
            enc.copy_buffer_to_buffer(&self.resolve, 0, &self.read, 0, n as u64 * 8);
        }
    }

    /// read the stamps after submit, this waits for the gpu so it is for profiling only.
    pub fn collect(&mut self, d: &wgpu::Device) {
        let labels = std::mem::take(&mut *self.labels.borrow_mut());
        let n = labels.len();
        if n > 1 {
            self.read.slice(..).map_async(wgpu::MapMode::Read, |_| {});
            let _ = d.poll(wgpu::PollType::Wait { submission_index: None, timeout: None });
            {
                let data = self.read.slice(..).get_mapped_range().expect("mapped");
                let ts: &[u64] = bytemuck::cast_slice(&data[..n * 8]);
                for i in 1..n {
                    let ms = ts[i].saturating_sub(ts[i - 1]) as f64 * self.period / 1e6;
                    match self.acc.iter_mut().find(|a| a.0 == labels[i]) {
                        Some(a) => a.1 += ms,
                        None => self.acc.push((labels[i], ms)),
                    }
                }
            }
            self.read.unmap();
            self.frames += 1;
        }
        if self.frames >= 60 {
            let k = 1.0 / self.frames as f64;
            let s: Vec<String> = self.acc.iter().map(|(l, ms)| format!("{l} {:.3}", ms * k)).collect();
            log::info!("gpu ms: {}", s.join(", "));
            CPU.with(|c| {
                let mut acc = c.acc.borrow_mut();
                let s: Vec<String> = acc.iter().map(|(l, ms)| format!("{l} {:.3}", ms * k)).collect();
                log::info!("cpu ms: {}", s.join(", "));
                acc.clear();
            });
            self.acc.clear();
            self.frames = 0;
        }
    }
}

struct Cpu {
    last: Cell<Option<Instant>>,
    acc: RefCell<Vec<(&'static str, f64)>>,
}

thread_local! {
    static CPU: Cpu = const { Cpu { last: Cell::new(None), acc: RefCell::new(Vec::new()) } };
}

/// start of a frame's CPU work.
pub fn cpu_begin() {
    if on() {
        CPU.with(|c| c.last.set(Some(Instant::now())));
    }
}

/// the CPU time since the last mark goes to `label`.
pub fn cpu(label: &'static str) {
    if !on() {
        return;
    }
    CPU.with(|c| {
        let now = Instant::now();
        if let Some(t) = c.last.replace(Some(now)) {
            let ms = (now - t).as_secs_f64() * 1e3;
            let mut acc = c.acc.borrow_mut();
            match acc.iter_mut().find(|a| a.0 == label) {
                Some(a) => a.1 += ms,
                None => acc.push((label, ms)),
            }
        }
    });
}
