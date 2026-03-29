use std::{
    mem,
    sync::Arc,
    thread::{self, JoinHandle},
};

use arc_swap::ArcSwap;
use crossbeam::channel::{Receiver, RecvError, Sender, unbounded};

use crate::rendering::atlas::Atlas;

#[derive(Debug)]
pub struct Frame {
    pub index: usize,
    pub rgba: Vec<u8>,
    pub width: usize,
    pub height: usize,
}

pub struct Texture {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

struct Triangle {
    vertices: [[f32; 2]; 3],
    fill: TriangleFillIndexed,
}

pub enum QuadFill {
    Color { color: [f32; 4] },
    Texture { tile_slot: u8, uvs: [[f32; 2]; 4] },
}

pub enum TriangleFill {
    Color { colors: [[f32; 4]; 3] },
    Texture { tile_slot: u8, uvs: [[f32; 2]; 3] },
}

enum TriangleFillIndexed {
    Color {
        colors: [[f32; 4]; 3],
    },
    Texture {
        tile_index: usize,
        uvs: [[f32; 2]; 3],
    },
}

pub enum Command {
    PushTile {
        slot: u8,
        rgba: Vec<u8>,
        width: u32,
        height: u32,
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

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Debug)]
struct GpuVertex {
    pos: [f32; 2],
    color: [f32; 4],
    uv: [f32; 2],
}

pub struct VideoRenderer {
    /// Rendering thread handle
    _thread: JoinHandle<()>,

    /// Command sender for the rendering thread
    command_tx: Sender<Command>,

    /// Last frame received from the rendering thread
    last_frame: Arc<ArcSwap<Frame>>,

    // TODO temp
    atlas_texture_debug: Arc<ArcSwap<Frame>>,
}

impl VideoRenderer {
    pub fn new() -> Self {
        let (command_tx, command_rx) = unbounded::<Command>();

        let last_frame = Arc::new(ArcSwap::new(Arc::new(Frame {
            index: 0,
            rgba: vec![],
            width: 0,
            height: 0,
        })));

        let atlas_texture_debug = Arc::new(ArcSwap::new(Arc::new(Frame {
            index: 0,
            rgba: vec![0; 1024 * 1024 * 4],
            width: 1024,
            height: 1024,
        })));

        let thread_last_frame = last_frame.clone();
        let thread_atlas_texture_debug = atlas_texture_debug.clone();

        let thread = thread::Builder::new()
            .name("Render".to_string())
            .spawn(move || {
                render_thread(command_rx, thread_last_frame, thread_atlas_texture_debug);
            })
            .expect("Failed to spawn render thread");

        Self {
            _thread: thread,
            command_tx,
            last_frame,
            atlas_texture_debug,
        }
    }

    pub fn push_command(&mut self, command: Command) {
        self.command_tx
            .send(command)
            .expect("Failed to send command to render thread");
    }

    pub fn get_frame(&self) -> Arc<Frame> {
        self.last_frame.load_full()
    }

    pub fn get_atlas_texture(&self) -> Arc<Frame> {
        self.atlas_texture_debug.load_full()
    }
}

fn render_thread(
    command_rx: Receiver<Command>,
    last_frame: Arc<ArcSwap<Frame>>,
    atlas_texture_debug: Arc<ArcSwap<Frame>>,
) {
    let mut renderer = WgpuRenderer::new();

    loop {
        match command_rx.recv() {
            Ok(command) => match command {
                Command::PushTile {
                    slot,
                    rgba,
                    width,
                    height,
                } => {
                    renderer.push_tile(slot, rgba, width, height);
                }

                Command::PushTriangle { vertices, fill } => {
                    renderer.push_triangle(vertices, fill);
                }

                Command::PushQuad { vertices, fill } => {
                    renderer.push_quad(vertices, fill);
                }

                Command::Render => {
                    renderer.render(&last_frame, &atlas_texture_debug);
                }
            },
            Err(RecvError) => {
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

    atlas_texture: wgpu::Texture,
    atlas_texture_view: wgpu::TextureView,
    sampler: wgpu::Sampler,
    bind_group_layout: wgpu::BindGroupLayout,

    atlas_texture_debug: Vec<u8>,

    pipeline: wgpu::RenderPipeline,
    readback_buffer: wgpu::Buffer,

    width: u32,
    height: u32,
    bytes_per_row: u32,

    triangle_queue: Vec<Triangle>,

    tile_textures: Vec<Texture>,
    last_tile_texture_texture_per_slot: [Option<usize>; 8],
}

const ERROR_COLOR: [f32; 4] = [1.0, 0.078, 0.576, 1.0]; // deeppink

impl WgpuRenderer {
    pub fn new() -> Self {
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

        const MAX_VERTICES: u64 = 10_000;

        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("vertex buffer"),
            size: mem::size_of::<GpuVertex>() as u64 * MAX_VERTICES,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        const ATLAS_SIZE: u32 = 1024;

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("texture"),
            size: wgpu::Extent3d {
                width: ATLAS_SIZE,
                height: ATLAS_SIZE,
                depth_or_array_layers: 1,
            },
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("texture_view"),
            format: Some(wgpu::TextureFormat::Rgba8UnormSrgb),
            dimension: Some(wgpu::TextureViewDimension::D2),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
            ..Default::default()
        });

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
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
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
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                //strip_index_format: None,
                //front_face: wgpu::FrontFace::Ccw,
                //cull_mode: None,
                //unclipped_depth: false,
                //polygon_mode: wgpu::PolygonMode::Fill,
                //conservative: false,
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

        // Receive commands

        Self {
            device,
            queue,

            color_target,
            color_view,
            // depth_target,
            // depth_view,
            vertex_buffer,

            atlas_texture: texture,
            atlas_texture_view: texture_view,
            atlas_texture_debug: vec![0; (ATLAS_SIZE * ATLAS_SIZE * 4) as usize],
            sampler,
            bind_group_layout,

            pipeline,
            readback_buffer: readback,

            width,
            height,
            bytes_per_row,

            triangle_queue: Vec::new(),

            tile_textures: Vec::new(),
            last_tile_texture_texture_per_slot: [None; 8],
        }
    }

    pub fn push_tile(&mut self, tile_slot: u8, rgba: Vec<u8>, width: u32, height: u32) {
        self.tile_textures.push(Texture {
            data: rgba,
            width,
            height,
        });

        self.last_tile_texture_texture_per_slot[tile_slot as usize] =
            Some(self.tile_textures.len() - 1);
    }

    pub fn push_triangle(&mut self, vertices: [[f32; 2]; 3], fill: TriangleFill) {
        let fill_indexed = match fill {
            TriangleFill::Color { colors } => TriangleFillIndexed::Color { colors },

            // Textured: associate the triangle with the last loaded texture corresponding to that slot
            TriangleFill::Texture { tile_slot, uvs } => {
                if let Some(tile_index) =
                    self.last_tile_texture_texture_per_slot[tile_slot as usize]
                {
                    TriangleFillIndexed::Texture { tile_index, uvs }
                } else {
                    log::error!("Video renderer: texture slot {} not loaded", tile_slot);

                    TriangleFillIndexed::Color {
                        colors: [ERROR_COLOR; 3],
                    }
                }
            }
        };

        self.triangle_queue.push(Triangle {
            vertices,
            fill: fill_indexed,
        });
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

            QuadFill::Texture {
                tile_slot: texture_slot,
                uvs,
            } => (
                TriangleFill::Texture {
                    tile_slot: texture_slot,
                    uvs: [uvs[0], uvs[1], uvs[3]],
                },
                TriangleFill::Texture {
                    tile_slot: texture_slot,
                    uvs: [uvs[2], uvs[3], uvs[1]],
                },
            ),
        };

        self.push_triangle(triangle1_vertices, triangle1_fill);
        self.push_triangle(triangle2_vertices, triangle2_fill);
    }

    pub fn render(
        &mut self,
        last_frame: &Arc<ArcSwap<Frame>>,
        atlas_texture_debug: &Arc<ArcSwap<Frame>>,
    ) {
        // let mut fake_data: Vec<u8> = vec![0; 1024 * 1024 * 4];
        // for y in 0..1024 {
        //     for x in 0..1024 {
        //         let offset = y * 1024 * 4 + x * 4;
        //         fake_data[offset + 0] = ((y as f32) / 1024.0 * 255.0) as u8;
        //         fake_data[offset + 1] = ((x as f32) / 1024.0 * 255.0) as u8;
        //         fake_data[offset + 3] = 0xFF;
        //     }
        // }

        // self.queue.write_texture(
        //     wgpu::TexelCopyTextureInfo {
        //         texture: &self.atlas_texture,
        //         mip_level: 0,
        //         origin: wgpu::Origin3d { x: 0, y: 0, z: 0 },
        //         aspect: wgpu::TextureAspect::All,
        //     },
        //     &fake_data,
        //     wgpu::TexelCopyBufferLayout {
        //         offset: 0,
        //         bytes_per_row: Some(4 * 1024),
        //         rows_per_image: Some(1024),
        //     },
        //     wgpu::Extent3d {
        //         width: 1024,
        //         height: 1024,
        //         depth_or_array_layers: 1,
        //     },
        // );

        // Generate and upload the atlas texture

        let atlas = Atlas::build(&self.tile_textures, 1024, 1024);

        //log::warn!("atlas cells COUNT {:?}", atlas.cells().len());

        for cell in atlas.cells() {
            // log::warn!(
            //     "cell {:?}, {:?}, {:?}, {:?}",
            //     cell.x,
            //     cell.y,
            //     cell.width,
            //     cell.height
            // );

            let tile_texture = &self.tile_textures[cell.tile_index];

            self.queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &self.atlas_texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: cell.x,
                        y: cell.y,
                        z: 0,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                &tile_texture.data,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * cell.width),
                    rows_per_image: Some(cell.height),
                },
                wgpu::Extent3d {
                    width: cell.width,
                    height: cell.height,
                    depth_or_array_layers: 1,
                },
            );
        }

        // TODO temp

        self.atlas_texture_debug.fill(0);

        for cell in atlas.cells() {
            let tile_texture = &self.tile_textures[cell.tile_index];

            let atlas_stride = 1024 as usize * 4;
            let tile_stride = cell.width as usize * 4;

            for y in 0..cell.height {
                let atlas_offset = (cell.y + y) as usize * atlas_stride + (cell.x as usize * 4);

                let tile_offset = y as usize * tile_stride;

                if y == 0 {
                    self.atlas_texture_debug[atlas_offset] = 0xFF;
                    self.atlas_texture_debug[atlas_offset + 1] = 0;
                    self.atlas_texture_debug[atlas_offset + 2] = 0;
                    self.atlas_texture_debug[atlas_offset + 3] = 0xFF;
                } else {
                    self.atlas_texture_debug[atlas_offset..atlas_offset + tile_stride]
                        .copy_from_slice(
                            &tile_texture.data[tile_offset..tile_offset + tile_stride],
                        );
                }
            }
        }

        atlas_texture_debug.store(Arc::new(Frame {
            index: atlas_texture_debug.load().index + 1,
            rgba: self.atlas_texture_debug.clone(),
            width: 1024,
            height: 1024,
        }));

        // TODO once???
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("bind group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&self.atlas_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
        });

        // Upload the geometry

        let triangle_count = self.triangle_queue.len();
        let vertex_count = triangle_count * 3;

        // TODO alloc once
        let mut gpu_vertices = Vec::with_capacity(vertex_count);

        for triangle in self.triangle_queue.iter() {
            match triangle.fill {
                TriangleFillIndexed::Color { colors } => {
                    for (vertex_pos, vertex_color) in triangle.vertices.iter().zip(colors) {
                        gpu_vertices.push(GpuVertex {
                            pos: *vertex_pos,
                            color: vertex_color,
                            uv: [0.0, 0.0],
                        });
                    }
                }

                TriangleFillIndexed::Texture { tile_index, uvs } => {
                    for (vertex_pos, vertex_uv) in triangle.vertices.iter().zip(uvs) {
                        gpu_vertices.push(GpuVertex {
                            pos: *vertex_pos,
                            color: [1.0, 1.0, 1.0, 1.0],
                            uv: atlas.remap_uv(vertex_uv, tile_index),
                        });
                    }
                }
            };
        }

        //log::warn!("quad count {:?}", triangle_count / 2);
        // log::warn!(
        //     "gpu_vertices  {:?}, vertex_count {:?}, triangle_count {:?}",
        //     gpu_vertices,
        //     vertex_count,
        //     triangle_count
        // );

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
            });

            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            pass.draw(0..vertex_count as u32, 0..triangle_count as u32);
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

        // Clear the rendered data

        self.triangle_queue.clear();

        self.tile_textures.clear();
        self.last_tile_texture_texture_per_slot = [None; 8];
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
