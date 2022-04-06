use std::mem;
use wgpu::util::DeviceExt;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow}
};
use chariot_core::GLOBAL_CONFIG;

mod game;
mod renderer;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 2]
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Local {
    scale : f32
}

fn vec2(x : f32, y : f32) -> Vertex {
    Vertex{position : [x, y]}
}

// for anyone skimming this: vertex buffers are created & set up in code but not actually used by the shader (yet),
// index buffers are also created but binding is borked for now
// also, don't try building for wasm because I don't think that works yet either
fn main() {
    // at some point, networking PoC:
    // let ip_addr = format!("{}:{}", GLOBAL_CONFIG.server_address, GLOBAL_CONFIG.port);
    // let game_client = game::GameClient::new(ip_addr);
    // game_client.ping();

    let event_loop = winit::event_loop::EventLoop::new();
    let context = renderer::Context::new(&event_loop);
    let mut renderer = renderer::Renderer::new(context);

    let vertex_size = mem::size_of::<Vertex>();
    renderer.register_pass("boring", &renderer::RenderPassDescriptor::Graphics { 
        source: include_str!("shader.wgsl"), 
        push_constant_ranges: &[], 
        targets: None, 
        primitive_state: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleStrip,
            strip_index_format: Some(wgpu::IndexFormat::Uint16),
            ..wgpu::PrimitiveState::default()
        },
        depth_stencil_state: None, 
        multisample_state: wgpu::MultisampleState::default(), 
        multiview: None
    });

    let tri_verts : &[Vertex; 3] = &[
        vec2(-1.0, -1.0),
        vec2(0.0, 1.0),
        vec2(1.0, -1.0)
    ];

    let tri_inds : &[u16] = &[
        0, 1, 2
    ];

    let vertex_buffer = renderer.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("tri_verts"),
        contents: bytemuck::cast_slice(tri_verts),
        usage: wgpu::BufferUsages::VERTEX,
    });

    let index_buffer = renderer.device.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
            label: Some("tri_inds"),
            contents: bytemuck::cast_slice(tri_inds),
            usage: wgpu::BufferUsages::INDEX,
        }
    );

    let uniform_data = &[Local {
        scale: 0.5
    }];

    let uniform_buffer = renderer.device.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
            label: Some("uniform_buf"),
            contents: bytemuck::cast_slice(uniform_data),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST
        }
    );

    let bind_group = renderer.create_bind_group("boring", 0, 
        &[uniform_buffer.as_entire_binding()]
    );

    event_loop.run(move |event, _, control_flow| {
        // Have the closure take ownership of the resources.
        // `event_loop.run` never returns, therefore we must do this to ensure
        // the resources are properly cleaned up.
        
        let tri_render_item = renderer::RenderItem::Graphics { 
            pass_name: "boring", 
            framebuffer_name: "surface", 
            num_vertices: 3, 
            vertex_buffers: &[vertex_buffer.slice(..)], 
            index_buffer: Some(index_buffer.slice(..)), 
            index_format: wgpu::IndexFormat::Uint16, 
            bind_group: &[&bind_group], 
            push_constants: &[]
        };

        let mut render_job = renderer::RenderJob::new();
        render_job.add_item(tri_render_item);

        *control_flow = ControlFlow::Wait;
        match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                renderer.handle_surface_resize(size);
            }
            Event::RedrawRequested(_) => {
                renderer.render(&render_job)
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            _ => {}
        }
    });
}
