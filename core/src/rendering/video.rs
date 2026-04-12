use std::{
    collections::{HashMap, HashSet},
    mem,
    sync::Arc,
    thread::{self, JoinHandle},
};

use arc_swap::ArcSwap;
use crossbeam::channel::{Receiver, Sender, unbounded};

use crate::rendering::tile_cache::Tile;

#[derive(Debug)]
pub struct Frame {
    pub index: usize,
    pub rgba: Vec<u8>,
    pub width: usize,
    pub height: usize,
}

pub enum QuadFill {
    Color { color: [f32; 4] },
    Texture { tile_id: u64, uvs: [[f32; 2]; 4] },
}

pub enum TriangleFill {
    Color { colors: [[f32; 4]; 3] },
    Texture { tile_id: u64, uvs: [[f32; 2]; 3] },
}

pub enum Command {
    PushTile {
        tile_id: u64,
        tile: Tile,
    },
    PushTriangle {
        vertices: [[f32; 2]; 3],
        fill: TriangleFill,
    },
    PushQuad {
        vertices: [[f32; 2]; 4],
        fill: QuadFill,
    },
    Render,
}

struct Triangle {
    vertices: [[f32; 2]; 3],
    fill: TriangleFill,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Debug)]
struct GpuVertex {
    pos: [f32; 2],
    color: [f32; 4],
    uv: [f32; 2],
}

struct TileTexture {
    tile: Tile,
    texture: wgpu::Texture,
    texture_view: wgpu::TextureView,
}

pub struct VideoRenderer {
    /// Rendering thread handle
    _thread: JoinHandle<()>,

    /// Command sender for the rendering thread
    command_tx: Sender<Command>,

    /// Last frame received from the rendering thread
    last_frame: Arc<ArcSwap<Frame>>,

    /// TODO temp
    debug_tiles: Vec<(u64, Tile)>,
    debug_tile_ids: HashSet<u64>,
}

impl Default for VideoRenderer {
    fn default() -> Self {
        let (command_tx, command_rx) = unbounded::<Command>();

        let last_frame = Arc::new(ArcSwap::new(Arc::new(Frame {
            index: 0,
            rgba: vec![],
            width: 0,
            height: 0,
        })));

        let thread_last_frame = last_frame.clone();

        let thread = thread::Builder::new()
            .name("Render".to_string())
            .spawn(move || {
                render_thread(command_rx, thread_last_frame);
            })
            .expect("Failed to spawn render thread");

        Self {
            _thread: thread,
            command_tx,

            last_frame,

            debug_tiles: Vec::new(),
            debug_tile_ids: HashSet::new(),
        }
    }
}

impl VideoRenderer {
    pub fn push_command(&mut self, command: Command) {
        if let Command::PushTile { tile_id, tile } = &command
            && !self.debug_tile_ids.contains(tile_id)
        {
            self.debug_tiles.push((*tile_id, tile.clone()));
            self.debug_tile_ids.insert(*tile_id);
        }

        self.command_tx
            .send(command)
            .expect("Failed to send command to render thread");
    }

    pub fn get_frame(&self) -> Arc<Frame> {
        self.last_frame.load_full()
    }

    pub fn get_debug_tiles(&self) -> &Vec<(u64, Tile)> {
        &self.debug_tiles
    }
}

fn render_thread(command_rx: Receiver<Command>, last_frame: Arc<ArcSwap<Frame>>) {
    let mut renderer = WgpuRenderer::new();

    loop {
        match command_rx.recv() {
            Ok(command) => match command {
                Command::PushTile { tile_id, tile } => {
                    renderer.push_tile(tile_id, tile);
                }

                Command::PushTriangle { vertices, fill } => {
                    renderer.push_triangle(vertices, fill);
                }

                Command::PushQuad { vertices, fill } => {
                    renderer.push_quad(vertices, fill);
                }

                Command::Render => {
                    renderer.render(&last_frame);
                }
            },
            Err(e) => {
                log::error!("Render thread: failed to receive command, {}", e);

                return;
            }
        }
    }
}

struct WgpuRenderer {
    device: wgpu::Device,
    queue: wgpu::Queue,

    color_target: wgpu::Texture,
    color_view: wgpu::TextureView,
    // depth_target: wgpu::Texture,
    // depth_view: wgpu::TextureView,
    vertex_buffer: wgpu::Buffer,

    sampler: wgpu::Sampler, // TODO mult?

    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    readback_buffer: wgpu::Buffer,

    width: u32,
    height: u32,
    bytes_per_row: u32,

    triangle_queue: Vec<Triangle>,

    neutral_texture: wgpu::Texture,
    neutral_texture_view: wgpu::TextureView,

    tile_textures: HashMap<u64, TileTexture>,
}

const ERROR_COLOR: [f32; 4] = [1.0, 0.078, 0.576, 1.0]; // deeppink

impl WgpuRenderer {
    pub fn new() -> Self {
        // Device

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: None,
            ..Default::default()
        }))
        .expect("Failed to request adapter");

        let (device, queue) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
                .expect("Failed to request device");

        // Color target

        let width = 512;
        let height = 512;

        let color_target = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("color target"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });

        let color_view = color_target.create_view(&wgpu::TextureViewDescriptor::default());

        // let depth_target = device.create_texture(&wgpu::TextureDescriptor {
        //     label: Some("depth target"),
        //     size: wgpu::Extent3d {
        //         width,
        //         height,
        //         depth_or_array_layers: 1,
        //     },
        //     mip_level_count: 1,
        //     sample_count: 1,
        //     dimension: wgpu::TextureDimension::D2,
        //     format: wgpu::TextureFormat::Depth32Float,
        //     usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        //     view_formats: &[],
        // });

        // let depth_view = depth_target.create_view(&wgpu::TextureViewDescriptor::default());

        // Vertex buffer

        const MAX_VERTICES: u64 = 10_000;

        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("vertex buffer"),
            size: mem::size_of::<GpuVertex>() as u64 * MAX_VERTICES,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let (neutral_texture, neutral_texture_view) = create_texture(
            &Tile {
                width: 1,
                height: 1,
                rgba: Arc::new(vec![0xFF; 4]),
            },
            &device,
            &queue,
        );

        // Pipeline

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("texture_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("sampler"),
            address_mode_u: wgpu::AddressMode::Repeat, // TODO?
            address_mode_v: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("video.shader").into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("pipeline layout"),
            bind_group_layouts: &[&bind_group_layout],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: mem::size_of::<GpuVertex>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x4, 2 => Float32x2],
                }],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        let unpadded_bytes_per_row = width * 4;
        let bytes_per_row =
            unpadded_bytes_per_row.next_multiple_of(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT);

        let readback = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("readback"),
            size: u64::from(bytes_per_row * height),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        Self {
            device,
            queue,

            color_target,
            color_view,
            // depth_target,
            // depth_view,
            vertex_buffer,

            sampler,

            pipeline,
            bind_group_layout,
            readback_buffer: readback,

            width,
            height,
            bytes_per_row,

            triangle_queue: Vec::new(),

            tile_textures: HashMap::new(),

            neutral_texture,
            neutral_texture_view,
        }
    }

    pub fn push_tile(&mut self, tile_id: u64, tile: Tile) {
        if !self.tile_textures.contains_key(&tile_id) {
            let (texture, texture_view) = create_texture(&tile, &self.device, &self.queue);

            self.tile_textures.insert(
                tile_id,
                TileTexture {
                    tile,
                    texture,
                    texture_view,
                },
            );
        }
    }

    pub fn push_triangle(&mut self, vertices: [[f32; 2]; 3], fill: TriangleFill) {
        self.triangle_queue.push(Triangle { vertices, fill });
    }

    pub fn push_quad(&mut self, vertices: [[f32; 2]; 4], fill: QuadFill) {
        // Split the quad into two triangles

        let triangle1_vertices = [vertices[0], vertices[1], vertices[3]];
        let triangle2_vertices = [vertices[2], vertices[3], vertices[1]];

        let (triangle1_fill, triangle2_fill) = match fill {
            QuadFill::Color { color } => (
                TriangleFill::Color {
                    colors: [color, color, color],
                },
                TriangleFill::Color {
                    colors: [color, color, color],
                },
            ),

            QuadFill::Texture { tile_id, uvs } => (
                TriangleFill::Texture {
                    tile_id,
                    uvs: [uvs[0], uvs[1], uvs[3]],
                },
                TriangleFill::Texture {
                    tile_id,
                    uvs: [uvs[2], uvs[3], uvs[1]],
                },
            ),
        };

        self.push_triangle(triangle1_vertices, triangle1_fill);
        self.push_triangle(triangle2_vertices, triangle2_fill);
    }

    pub fn render(&mut self, last_frame: &Arc<ArcSwap<Frame>>) {
        // Upload the geometry

        let triangle_count = self.triangle_queue.len();
        let vertex_count = triangle_count * 3;

        // TODO alloc once OR do it on push?
        let mut gpu_vertices = Vec::with_capacity(vertex_count);

        for triangle in self.triangle_queue.iter() {
            match triangle.fill {
                TriangleFill::Color { colors } => {
                    for (vertex_pos, vertex_color) in triangle.vertices.iter().zip(colors) {
                        gpu_vertices.push(GpuVertex {
                            pos: *vertex_pos,
                            color: vertex_color,
                            uv: [0.0, 0.0],
                        });
                    }
                }

                TriangleFill::Texture { uvs, .. } => {
                    for (vertex_pos, vertex_uv) in triangle.vertices.iter().zip(uvs) {
                        gpu_vertices.push(GpuVertex {
                            pos: *vertex_pos,
                            color: [1.0, 1.0, 1.0, 1.0],
                            uv: vertex_uv,
                        });
                    }
                }
            };
        }

        self.queue.write_buffer(
            &self.vertex_buffer,
            0,
            bytemuck::cast_slice(gpu_vertices.as_slice()),
        );

        // Render

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("encoder"),
            });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.color_view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
                // TODO simplify
            });

            pass.set_pipeline(&self.pipeline);
            pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));

            // One draw call per ???? TODO doc

            let mut rendered_vertices = 0;

            self.triangle_queue
                .chunk_by(|a, b| same_render_group(&a.fill, &b.fill))
                .for_each(|triangles| {
                    // Get the texture used by this chunk, if any

                    let texture_view = triangles
                        .iter()
                        .find_map(|triangle| {
                            if let TriangleFill::Texture { tile_id, .. } = &triangle.fill
                                && let Some(tile) = self.tile_textures.get(tile_id)
                            {
                                Some(&tile.texture_view)
                            } else {
                                None
                            }
                        })
                        .unwrap_or(&self.neutral_texture_view);

                    // Render

                    let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("bind group"),
                        layout: &self.bind_group_layout,
                        entries: &[
                            wgpu::BindGroupEntry {
                                binding: 0,
                                resource: wgpu::BindingResource::TextureView(texture_view),
                            },
                            wgpu::BindGroupEntry {
                                binding: 1,
                                resource: wgpu::BindingResource::Sampler(&self.sampler),
                            },
                        ],
                    });

                    pass.set_bind_group(0, &bind_group, &[]);

                    pass.draw(
                        rendered_vertices..rendered_vertices + (triangles.len() * 3) as u32,
                        0..1,
                    );

                    rendered_vertices += (triangles.len() * 3) as u32;
                });
        }

        encoder.copy_texture_to_buffer(
            self.color_target.as_image_copy(),
            wgpu::TexelCopyBufferInfo {
                buffer: &self.readback_buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(self.bytes_per_row),
                    rows_per_image: Some(self.height),
                },
            },
            wgpu::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
        );

        let submission = self.queue.submit(std::iter::once(encoder.finish()));

        // Read the output buffer

        let last_frame = last_frame.clone();
        let readback_buffer = self.readback_buffer.clone();
        let width = self.width as usize;
        let height = self.height as usize;
        let bytes_per_row = self.bytes_per_row as usize;

        self.readback_buffer
            .slice(..)
            .map_async(wgpu::MapMode::Read, move |_| {
                let data = readback_buffer.slice(..).get_mapped_range();

                let rgba = unpack_rgba_rows(&data, width, height, bytes_per_row);

                drop(data);

                readback_buffer.unmap();

                last_frame.store(Arc::new(Frame {
                    index: last_frame.load().index + 1,
                    rgba,
                    width,
                    height,
                }));
            });

        self.device
            .poll(wgpu::PollType::Wait {
                submission_index: Some(submission),
                timeout: None,
            })
            .expect("Failed to wait for submission");

        // Clear the rendered triangles

        self.triangle_queue.clear();
    }
}

fn unpack_rgba_rows(padded: &[u8], width: usize, height: usize, bytes_per_row: usize) -> Vec<u8> {
    let row_pixels = width * 4;
    let mut out = Vec::with_capacity(row_pixels * height);
    for row in 0..height {
        let start = row * bytes_per_row;
        out.extend_from_slice(&padded[start..start + row_pixels]);
    }
    out
}

fn create_texture(
    tile: &Tile,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
) -> (wgpu::Texture, wgpu::TextureView) {
    // TODO default params?

    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: None,
        size: wgpu::Extent3d {
            width: tile.width,
            height: tile.height,
            depth_or_array_layers: 1,
        },
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        mip_level_count: 1,
        sample_count: 1,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });

    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d { x: 0, y: 0, z: 0 },
            aspect: wgpu::TextureAspect::All,
        },
        &tile.rgba,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(4 * tile.width),
            rows_per_image: Some(tile.height),
        },
        wgpu::Extent3d {
            width: tile.width,
            height: tile.height,
            depth_or_array_layers: 1,
        },
    );

    let view = texture.create_view(&wgpu::TextureViewDescriptor {
        label: None,
        format: Some(wgpu::TextureFormat::Rgba8UnormSrgb),
        dimension: Some(wgpu::TextureViewDimension::D2),
        aspect: wgpu::TextureAspect::All,
        base_mip_level: 0,
        mip_level_count: None,
        base_array_layer: 0,
        array_layer_count: None,
        ..Default::default()
    });

    (texture, view)
}

fn same_render_group(a: &TriangleFill, b: &TriangleFill) -> bool {
    // TODO split color/tex?
    match (a, b) {
        (TriangleFill::Color { .. }, TriangleFill::Color { .. }) => true,
        (TriangleFill::Color { .. }, TriangleFill::Texture { .. }) => true,
        (TriangleFill::Texture { .. }, TriangleFill::Color { .. }) => true,
        (
            TriangleFill::Texture {
                tile_id: a_tile_id, ..
            },
            TriangleFill::Texture {
                tile_id: b_tile_id, ..
            },
        ) => a_tile_id == b_tile_id,
    }
}
