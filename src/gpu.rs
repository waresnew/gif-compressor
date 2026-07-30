use std::sync::mpsc::channel;

use bytemuck::{Pod, Zeroable};
use wgpu::{
    BindGroupDescriptor, BindGroupEntry, BufferDescriptor, BufferUsages, ComputePipelineDescriptor,
    DeviceDescriptor, Limits, PollType, RequestAdapterOptions,
    util::{BufferInitDescriptor, DeviceExt, DownloadBuffer},
};

use crate::image::{Image, Rgb};

pub fn run_shader_with_frames(
    entry_point: &str,
    frames: Vec<&Image>,
    palettes: Vec<&Vec<Rgb>>,
) -> Vec<Image> {
    pollster::block_on(run_shader_with_frames_async(entry_point, frames, palettes))
}
#[derive(Pod, Zeroable, Clone, Copy)]
#[repr(C)]
struct GlobalInfo {
    num_frames: u32,
    height: u32,
    width: u32,
    _padding: u32, //TODO: needed?
}

#[derive(Pod, Zeroable, Clone, Copy)]
#[repr(C)]
struct RgbGpu {
    r: u32,
    g: u32,
    b: u32,
    _padding: u32, //need to align vec3 to 16 bytes
}
impl RgbGpu {
    pub fn from_rgb(rgb: Rgb) -> Self {
        Self {
            r: rgb.r as u32,
            g: rgb.g as u32,
            b: rgb.b as u32,
            _padding: 0,
        }
    }
}

async fn run_shader_with_frames_async(
    entry_point: &str,
    frames: Vec<&Image>,
    palettes: Vec<&Vec<Rgb>>,
) -> Vec<Image> {
    if frames.is_empty() {
        return frames.into_iter().cloned().collect();
    }
    let num_frames = frames.len();
    if palettes.len() != num_frames {
        panic!(
            "palettes.len() was {} but num_frames was {}",
            palettes.len(),
            num_frames
        );
    }
    let height = frames.first().unwrap().height;
    let width = frames.first().unwrap().width;
    let instance = wgpu::Instance::default();
    let adapter = instance
        .request_adapter(&RequestAdapterOptions::default())
        .await
        .unwrap();
    let adapter_limits = adapter.limits();
    let (device, queue) = adapter
        .request_device(&DeviceDescriptor {
            required_limits: Limits {
                max_storage_buffer_binding_size: adapter_limits.max_storage_buffer_binding_size,
                max_buffer_size: adapter_limits.max_buffer_size,
                ..Default::default()
            },
            ..Default::default()
        })
        .await
        .unwrap();

    let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

    let global_info = GlobalInfo {
        num_frames: num_frames as u32,
        height: height as u32,
        width: width as u32,
        _padding: 0,
    };
    let global_info_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: Some("global_info"),
        contents: bytemuck::cast_slice(&[global_info]),
        usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
    });

    let mut palette_offsets = Vec::new();
    {
        let mut acc = 0_u32;
        for palette in &palettes {
            palette_offsets.push(acc);
            acc += palette.len() as u32;
        }
        palette_offsets.push(acc);
    }
    let palettes_input: Vec<RgbGpu> = palettes
        .into_iter()
        .flatten()
        .copied()
        .map(RgbGpu::from_rgb)
        .collect();
    let palette_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: Some("palettes"),
        contents: bytemuck::cast_slice(&palettes_input),
        usage: BufferUsages::COPY_DST | BufferUsages::STORAGE,
    });
    let palette_offset_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: Some("palette_offsets"),
        contents: bytemuck::cast_slice(&palette_offsets),
        usage: BufferUsages::COPY_DST | BufferUsages::STORAGE,
    });

    let frames_input: Vec<RgbGpu> = frames
        .iter()
        .flat_map(|img| img.buffer.clone())
        .map(RgbGpu::from_rgb)
        .collect();

    let input_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: Some("input"),
        contents: bytemuck::cast_slice(&frames_input),
        usage: BufferUsages::COPY_DST | BufferUsages::STORAGE,
    });
    let output_buffer = device.create_buffer(&BufferDescriptor {
        label: Some("output"),
        size: input_buffer.size(),
        usage: BufferUsages::COPY_SRC | BufferUsages::STORAGE,
        mapped_at_creation: false,
    });

    let pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
        label: Some("GIF frame pipeline"),
        layout: None,
        module: &shader,
        entry_point: Some(entry_point),
        compilation_options: Default::default(),
        cache: Default::default(),
    });
    let bind_group = device.create_bind_group(&BindGroupDescriptor {
        label: None,
        layout: &pipeline.get_bind_group_layout(0),
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: global_info_buffer.as_entire_binding(),
            },
            BindGroupEntry {
                binding: 1,
                resource: palette_buffer.as_entire_binding(),
            },
            BindGroupEntry {
                binding: 2,
                resource: palette_offset_buffer.as_entire_binding(),
            },
            BindGroupEntry {
                binding: 3,
                resource: input_buffer.as_entire_binding(),
            },
            BindGroupEntry {
                binding: 4,
                resource: output_buffer.as_entire_binding(),
            },
        ],
    });
    let mut encoder = device.create_command_encoder(&Default::default());
    {
        let mut cpass = encoder.begin_compute_pass(&Default::default());
        cpass.set_pipeline(&pipeline);
        cpass.set_bind_group(0, &bind_group, &[]);
        cpass.dispatch_workgroups(
            num_frames.div_ceil(4) as u32,
            height.div_ceil(8) as u32,
            width.div_ceil(8) as u32,
        );
    }
    queue.submit(Some(encoder.finish()));
    let (tx, rx) = channel();
    DownloadBuffer::read_buffer(&device, &queue, &output_buffer.slice(..), move |result| {
        tx.send(result.unwrap().to_vec()).unwrap()
    });
    device.poll(PollType::wait_indefinitely()).unwrap();
    let bytes = rx.recv().unwrap();
    bytes
        .chunks_exact(bytes.len() / num_frames)
        .map(|frame_bytes| {
            let rgbs = frame_bytes.chunks_exact(4 * 4).map(|rgb_bytes| {
                Rgb::new(rgb_bytes[0], rgb_bytes[4], rgb_bytes[8]) //little endian
                //remember 12..16 is _padding
            });
            assert_eq!(rgbs.len(), height * width);

            Image {
                buffer: rgbs.collect(),
                height,
                width,
            }
        })
        .collect()
}
pub fn get_highest_chunk_size() -> usize {
    pollster::block_on(get_highest_chunk_size_async())
}
async fn get_highest_chunk_size_async() -> usize {
    let instance = wgpu::Instance::default();
    let adapter = instance
        .request_adapter(&RequestAdapterOptions::default())
        .await
        .unwrap();
    let adapter_limits = adapter.limits();
    adapter_limits
        .max_storage_buffer_binding_size
        .min(adapter_limits.max_buffer_size)
        .min(usize::MAX as u64) as usize
}
