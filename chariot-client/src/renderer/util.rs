// For directly drawing to the surface
#[macro_export]
macro_rules! direct_graphics_depth_pass {
    ( $filename: expr ) => {
        crate::renderer::render_job::RenderPassDescriptor::Graphics {
            source: std::fs::read_to_string($filename)
                .expect("Unable to open shader file")
                .as_str(),
            push_constant_ranges: &[],
            targets: None,
            primitive_state: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                ..wgpu::PrimitiveState::default()
            },
            outputs_depth: true,
            multisample_state: wgpu::MultisampleState::default(),
            multiview: None,
        }
    };
}

pub(crate) use direct_graphics_depth_pass;

// For drawing to an arbitary framebuffer
#[macro_export]
macro_rules! indirect_graphics_depth_pass {
    ( $filename: expr, $formats: expr ) => {
        crate::renderer::render_job::RenderPassDescriptor::Graphics {
            source: std::fs::read_to_string($filename)
                .expect("Unable to open shader file")
                .as_str(),
            push_constant_ranges: &[],
            targets: Some(
                &$formats
                    .iter()
                    .map(|f| wgpu::ColorTargetState {
                        format: *f,
                        blend: Some(wgpu::BlendState {
                            alpha: wgpu::BlendComponent::REPLACE,
                            color: wgpu::BlendComponent::REPLACE,
                        }),
                        write_mask: wgpu::ColorWrites::ALL,
                    })
                    .collect::<Vec<wgpu::ColorTargetState>>(),
            ),
            primitive_state: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                ..wgpu::PrimitiveState::default()
            },
            outputs_depth: true,
            multisample_state: wgpu::MultisampleState::default(),
            multiview: None,
        }
    };
}

pub(crate) use indirect_graphics_depth_pass;

#[macro_export]
macro_rules! direct_graphics_nodepth_pass {
    ( $filename: expr ) => {
        crate::renderer::render_job::RenderPassDescriptor::Graphics {
            source: std::fs::read_to_string($filename)
                .expect("Unable to open shader file")
                .as_str(),
            push_constant_ranges: &[],
            targets: None,
            primitive_state: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                ..wgpu::PrimitiveState::default()
            },
            outputs_depth: false,
            multisample_state: wgpu::MultisampleState::default(),
            multiview: None,
        }
    };
}

pub(crate) use direct_graphics_nodepth_pass;

#[macro_export]
macro_rules! shadow_pass {
    ( $filename: expr ) => {
        crate::renderer::render_job::RenderPassDescriptor::Graphics {
            source: std::fs::read_to_string($filename)
                .expect("Unable to open shader file")
                .as_str(),
            push_constant_ranges: &[],
            targets: Some(&[]),
            primitive_state: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                cull_mode: Some(wgpu::Face::Front),
                ..wgpu::PrimitiveState::default()
            },
            outputs_depth: true,
            multisample_state: wgpu::MultisampleState::default(),
            multiview: None,
        }
    };
}

pub(crate) use shadow_pass;

#[macro_export]
macro_rules! indirect_surfel_pass {
    ( $filename: expr, $formats: expr ) => {
        crate::renderer::render_job::RenderPassDescriptor::Graphics {
            source: std::fs::read_to_string($filename)
                .expect("Unable to open shader file")
                .as_str(),
            push_constant_ranges: &[],
            targets: Some(
                &$formats
                    .iter()
                    .map(|f| wgpu::ColorTargetState {
                        format: *f,
                        blend: Some(wgpu::BlendState {
                            alpha: wgpu::BlendComponent::REPLACE,
                            color: wgpu::BlendComponent::REPLACE,
                        }),
                        write_mask: wgpu::ColorWrites::ALL,
                    })
                    .collect::<Vec<wgpu::ColorTargetState>>(),
            ),
            primitive_state: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::PointList,
                strip_index_format: None,
                ..wgpu::PrimitiveState::default()
            },
            outputs_depth: true,
            multisample_state: wgpu::MultisampleState::default(),
            multiview: None,
        }
    };
}

pub(crate) use indirect_surfel_pass;
