use std::{
    collections::{HashMap, VecDeque},
    num::NonZeroU32,
};

#[allow(dead_code)] // Compute stuff is unused for now
pub enum RenderPassDescriptor<'a> {
    Graphics {
        source: &'a str,
        push_constant_ranges: &'a [wgpu::PushConstantRange],
        targets: Option<&'a [wgpu::ColorTargetState]>,
        primitive_state: wgpu::PrimitiveState,
        outputs_depth: bool,
        multisample_state: wgpu::MultisampleState,
        multiview: Option<NonZeroU32>,
    },
    Compute {
        source: &'a str,
        bind_group_layouts: &'a [&'a wgpu::BindGroupLayout],
        push_constant_ranges: &'a [wgpu::PushConstantRange],
    },
}

/*
 * A render pass encapsulates everything needed for pipeline setup.
 * For now, ignore all the compute pass/render item stuff in this file since that will mostly
 * come in handy later if I want to do some fancy stuff in the end of this class. Most of it will
 * have to wait until the render graph is implemented.
 */
pub enum RenderPass {
    Graphics {
        shader: wgpu::ShaderModule,
        pipeline_layout: wgpu::PipelineLayout,
        render_pipeline: wgpu::RenderPipeline,
    },
    Compute {
        shader: wgpu::ShaderModule,
        pipeline_layout: wgpu::PipelineLayout,
        compute_pipeline: wgpu::ComputePipeline,
    },
}

pub fn pass_render_pipeline(pass: &RenderPass) -> Option<&wgpu::RenderPipeline> {
    match pass {
        RenderPass::Graphics {
            render_pipeline, ..
        } => Some(render_pipeline),
        _ => None,
    }
}

#[allow(dead_code)] // Compute stuff is unused for now
pub fn pass_compute_pipeline(pass: &RenderPass) -> Option<&wgpu::ComputePipeline> {
    match pass {
        RenderPass::Compute {
            compute_pipeline, ..
        } => Some(compute_pipeline),
        _ => None,
    }
}

/*
 * A RenderItem stores all state for a single draw call (or in the future, a compute dispatch call)
 */

#[allow(dead_code)] // Compute and Custom stuff is unused for now
#[derive(Clone)]
pub enum RenderItem<'a> {
    Graphics {
        pass_name: &'a str,
        framebuffer_name: &'a str,
        num_elements: u32,
        vertex_buffers: Vec<wgpu::BufferSlice<'a>>,
        index_buffer: Option<wgpu::BufferSlice<'a>>,
        index_format: wgpu::IndexFormat,
        bind_group: Vec<&'a wgpu::BindGroup>,
    },
    Compute {
        pass_name: &'a str,
        bind_group: Vec<&'a wgpu::BindGroup>,
    },
    Custom {
        pass_name: &'a str,
        cb: fn(&mut wgpu::CommandEncoder),
    },
}

impl<'a> RenderItem<'a> {
    pub fn to_graph(self) -> RenderGraph<'a> {
        let mut builder = RenderGraphBuilder::new();
        builder.add_root(self);
        builder.build()
    }
}

pub fn render_item_pass_name<'a>(pass: &'a RenderItem) -> &'a str {
    match pass {
        RenderItem::Graphics { pass_name, .. } => pass_name,
        RenderItem::Compute { pass_name, .. } => pass_name,
        RenderItem::Custom { pass_name, .. } => pass_name,
    }
}

pub fn render_item_framebuffer_name<'a>(item: &'a RenderItem) -> Option<&'a str> {
    match item {
        RenderItem::Graphics {
            framebuffer_name, ..
        } => Some(framebuffer_name),
        _ => None,
    }
}

pub fn try_unpack_graphics_item<'a, 'b>(
    item: &'b RenderItem<'a>,
) -> Option<(
    &'a str,
    &'a str,
    u32,
    &'b [wgpu::BufferSlice<'a>],
    Option<&'b wgpu::BufferSlice<'a>>,
    wgpu::IndexFormat,
    &'b [&'a wgpu::BindGroup],
)> {
    if let RenderItem::Graphics {
        pass_name,
        framebuffer_name,
        num_elements,
        vertex_buffers,
        index_buffer,
        index_format,
        bind_group,
    } = item
    {
        Some((
            pass_name,
            framebuffer_name,
            *num_elements,
            vertex_buffers,
            index_buffer.as_ref(),
            *index_format,
            bind_group,
        ))
    } else {
        None
    }
}

type RenderNodeId = usize;

/*
 * A render graph stores the graph of items that need to be executed to draw a single drawable.
 * For example, first render the mesh to the shadow map, then forward pass, etc
 */
#[derive(Default)]
pub struct RenderGraph<'a> {
    items: Vec<RenderItem<'a>>,
    nodes: HashMap<RenderNodeId, Vec<RenderNodeId>>,
    roots: Vec<RenderNodeId>,
}

impl<'a> RenderGraph<'a> {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            nodes: HashMap::new(),
            roots: Vec::new(),
        }
    }
}

pub struct RenderGraphBuilder<'a> {
    render_graph: RenderGraph<'a>,
}

impl<'a> RenderGraphBuilder<'a> {
    pub fn new() -> Self {
        Self {
            render_graph: RenderGraph::new(),
        }
    }

    pub fn add(&mut self, item: RenderItem<'a>, deps: &[RenderNodeId]) -> RenderNodeId {
        let res_id = self.render_graph.items.len();
        self.render_graph.items.push(item);
        for dep in deps {
            self.render_graph
                .nodes
                .get_mut(dep)
                .expect("Invalid dependency id")
                .push(res_id);
        }

        if deps.is_empty() {
            self.render_graph.roots.push(res_id);
        }

        self.render_graph.nodes.insert(res_id, vec![]);

        res_id
    }

    pub fn add_root(&mut self, item: RenderItem<'a>) -> RenderNodeId {
        self.add(item, &[])
    }

    pub fn build(&mut self) -> RenderGraph<'a> {
        std::mem::take(&mut self.render_graph)
    }
}

/*
 * kinda ugly but whatevs
 * Encapsulates a graph of RenderItem lists organized by render pass.
 * It just does a bit of work when merging render graphs to organize everything properly.
 * Otherwise it doesn't care to organize further into vertex buffer or bind group bindings.
 * It also has a bfs iterator to help with iteration. The implementation is a little lazy since it
 * just pretraverses the graph and caches the indexing order.
 */

#[derive(Default)]
pub struct RenderJob<'a> {
    pass_to_id: HashMap<String, RenderNodeId>,
    pass_items: Vec<Vec<RenderItem<'a>>>,
    graph: HashMap<RenderNodeId, HashMap<String, RenderNodeId>>,
    roots: HashMap<String, RenderNodeId>,
}

impl<'a> RenderJob<'a> {
    pub fn merge_graph(&mut self, graph: RenderGraph<'a>) {
        self.merge_graph_internal(None, graph);
    }
    pub fn merge_graph_after(&mut self, after: &str, graph: RenderGraph<'a>) {
        let root_id = self.pass_to_id.get(&after.to_string()).map(|r| *r);
        self.merge_graph_internal(root_id, graph);
    }

    fn merge_graph_internal(&mut self, root_id: Option<RenderNodeId>, graph: RenderGraph<'a>) {
        let root = root_id.map_or(&mut self.roots, |id| self.graph.get_mut(&id).unwrap());
        let job_root_ids: Vec<usize> = graph
            .roots
            .iter()
            .map(|rid| {
                let pass_name = render_item_pass_name(&graph.items[*rid]);
                let job_id = root.entry(String::from(pass_name)).or_insert_with(|| {
                    let new_id = self.pass_items.len();
                    self.pass_items.push(vec![]);
                    new_id
                });

                *job_id
            })
            .collect();

        let mut stack: Vec<(RenderNodeId, RenderNodeId)> = graph
            .roots
            .into_iter()
            .zip(job_root_ids.into_iter())
            .collect();
        let max_job_id = stack.last().unwrap().1;
        (self.pass_items.len()..max_job_id).for_each(|_| self.pass_items.push(vec![]));
        while !stack.is_empty() {
            let (graph_id, job_id) = stack.pop().unwrap();
            self.pass_items[job_id].push(graph.items[graph_id].clone());
            let pass_name = render_item_pass_name(&graph.items[graph_id]);
            self.pass_to_id.insert(pass_name.to_string(), job_id);

            let cur_node = self.graph.entry(job_id).or_default();
            for child_graph_id in graph.nodes.get(&graph_id).unwrap_or(&vec![]).iter() {
                let child_pass_name = render_item_pass_name(&graph.items[*child_graph_id]);
                let child_job_id = cur_node
                    .entry(String::from(child_pass_name))
                    .or_insert_with(|| {
                        let new_id = self.pass_items.len();
                        self.pass_items.push(vec![]);
                        new_id
                    });

                stack.push((*child_graph_id, *child_job_id));
            }
        }
    }

    pub fn iter_bfs(&'a self) -> Iter {
        let mut res = vec![];
        let mut processed: Vec<bool> = vec![false; self.pass_items.len()];
        let mut queue: VecDeque<RenderNodeId> = self.roots.clone().into_values().collect();
        while !queue.is_empty() {
            let cur_id = queue.pop_front().unwrap();
            if processed[cur_id] {
                continue;
            }

            let cur_items = &self.pass_items[cur_id];
            assert!(cur_items.len() > 0);
            let pass_name = render_item_pass_name(&cur_items[0]);
            res.push((pass_name, cur_id));

            processed[cur_id] = true;
            queue.extend(
                self.graph
                    .get(&cur_id)
                    .unwrap_or(&HashMap::default())
                    .values(),
            );
        }

        Iter::new(res, self)
    }
}

pub struct Iter<'a, 'b> {
    node_ids: Vec<(&'a str, RenderNodeId)>,
    job: &'a RenderJob<'b>,
    cur_idx: usize,
}

impl<'a, 'b> Iter<'a, 'b> {
    fn new(node_ids: Vec<(&'a str, RenderNodeId)>, job: &'a RenderJob<'b>) -> Self {
        Self {
            node_ids,
            job,
            cur_idx: 0,
        }
    }
}

impl<'a, 'b> Iterator for Iter<'a, 'b> {
    type Item = (&'a str, &'a [RenderItem<'b>]);

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.node_ids.get(self.cur_idx);
        self.cur_idx += 1;
        next.map(|(s, i)| (*s, self.job.pass_items[*i].as_slice()))
    }
}
