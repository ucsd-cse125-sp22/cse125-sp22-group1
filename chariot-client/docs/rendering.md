# Renderer and Scene Graph Architecture

I noticed there was a lot of confusion about the renderer and scene graph and I apologize for any unnecessary 
complexity as I was a little too excited at the begining of this project. Luckily, most people only need to 
interact with a high level interface.

## High Level Stuff
Basically the stuff most people will be using:

### Resources
Most things in the scene graph only have references to resources stored by the resource manager. This way, we can have multiple
instances of an object without requiring that each instance own its own copy of the data (AKA there would be multiple copies of the same data).
This is also slightly influenced by resource managers in traditional game engines that support lazy loading and unloading of resources. 
When games don't want to have all their assets (which may be several GB) on the GPU/CPU memory, the resource manager also takes on the job
of resolving all the resources for a scene/frame and unloading any resources that haven't been used in a while. 
Our resource manager isn't this fancy though. 

Right now the resource manager only supports importing gltf models, though in the future I might also add support for loading textures and other stuff.
Importing models is pretty simple:
```rust
let import_result = resources.import_gltf(&mut renderer, "models/DamagedHelmet.glb");
```

The import result struct contains the references to all the assets imported from the gltf file and some drawables ready to be inserted into the scene.
I'll get into what drawables are later but they are basically the things the stored by the scene for a single instance of a model. In the future they
can be cloned and inserted into the scene multiple times to get instances.

```rust
pub struct ImportData {
    pub tex_handles: Vec<TextureHandle>,
    pub material_handles: Vec<MaterialHandle>,
    pub mesh_handles: Vec<StaticMeshHandle>,
    pub drawables: Vec<StaticMeshDrawable>,
}
```
Ideally these would be maps from identifier strings to handles but the gltf format doesn't guarantee names for textures/materials/meshes 
so they're just stored in arrays. I would just recommend having one model per gltf file.

### Scene Graph
Those who have worked with game engines like Unreal or Unity might be familiar with this. 

![](https://teamwisp.github.io/images/sg/image_0.png)

Scenes usually can be represented as a hierarchy of objects (in our case we call them entities) which can be attached to other objects. 
When rendering, we traverse the graph and accumulate the parent transforms when calculating the location of each node.

Adding an entity to a scene graph is simple:
```rust
// First, get a mutable reference to the root entity, then attach the "helmet" entity to the root as a child
world.root_mut().add_child(helmet);
```

Entities usually store similar data types and it can be nice to have a uniform interface to access this data. For example, 
if we need to calculate the world-space transform of each entity, we need a way to access the transform data for each node.

This is where components come in. Components allow a way to both customize entities and access entity data in a uniform way.

```rust
let mut helmet = Entity::new();

// Add a component to this entity using set_component
helmet.set_component(Transform {
	translation: glam::Vec3::ZERO,
	rotation: glam::Quat::from_axis_angle(glam::Vec3::X, f32::to_radians(90.0)),
	scale: glam::vec3(0.3, 0.3, 0.3),
});

helmet.set_component(drawables);
```

Then, components can be read off entities in the application by traversing the graph like this:

```rust
// dfs() calls the given function argument for each entity in the graph in dfs order,
// starting from the given node (in this case the root node)
dfs(self.world.root(), &|e| {
	if let Some(transform) = e.get_component::<Transform>() {
		// do something with transform
	}
});
```

There are also dfs variants for traversals that modify entity data (`dfs_mut`) and 
accumulate data (`dfs_acc`, used when rendering to calculate global transforms).

Yes, this is different from how Unity and Unreal do things, but Rust makes that way difficult. 
I might try a simple home-rolled ECS later.

## Intermediate Level Stuff

Read this if you're working more closely with the renderer. 
Some of the stuff in this section is WIP and will be re-organized into different files since a lot of it is
just in drawable.rs for now.

### The story of a frame
Starting at `Application::render()`:
```rust
// A render job is created and render graphs are collected from the drawables:
let mut render_job = render_job::RenderJob::default();
dfs_acc(self.world.root_mut(), root_transform, |e, acc| {
	let model = // calculate world transform

	if let Some(drawables) = e.get_component::<Vec<StaticMeshDrawable>>() {
		for drawable in drawables.iter() {
			// upload the world transform and camera transform data to the GPU
			drawable.update_xforms(&self.renderer, &proj_view, &model);

			// Then get the render graph for this item and merge it into the job for this frame.
			// Render graphs are explained below in more detail but they basically contain a sequence 
			// of things that need to be executed for a single drawable.
			let render_graph = drawable.render_graph(&self.resources);
			render_job.merge_graph(render_graph);
		}
	}

	model
});

// Then send the render job to the renderer and let it send the commands to the GPU
self.renderer.render(&render_job);
```

Before we step into `Drawable::render_graph`, we first need to understand the structure of a drawable.

A drawable produces the render graph for an object on the screen. Right now there is just a StaticMeshDrawable, 
but in the future there could be drawables for particle effects and other stuff 
(games sometimes have a separate drawable type thing for skeletal meshes).

```rust
pub trait Drawable {
    fn render_graph<'a>(&'a self, resources: &'a ResourceManager) -> render_job::RenderGraph<'a>;
}
```

In the near future, drawables will be made up of techniques that store the data and produce individual draw items.

```rust
pub trait Technique {
    fn render_item<'a>(&'a self, resources: &'a ResourceManager) -> render_job::RenderItem<'a>;
}
```

So `render_graph()` for a StaticMeshDrawable might look like:
```rust
impl Drawable for StaticMeshDrawable {
    fn render_graph<'a>(&'a self, resources: &'a ResourceManager) -> render_job::RenderGraph<'a> {
        let mut graph_builder = RenderGraphBuilder::new();

		let shadow_deps = vec![];
		for shadow_draw in shadow_draws {
			let shadow_item = shadow_draws.render_item(resources);
        	let node_id = graph_builder.add_root(shadow_item);
			shadow_deps.push(node_id);
		}

		let forward_item = forward_draw.render_item(resources);
		graph_builder.add(&shadow_deps, forward_item);
        graph_builder.build()
    }
}
```

As an example of what `render_item()` might look like for a technique:
```rust
impl Technique for ForwardDrawTechnique {
    fn render_item<'a>(&'a self, resources: &'a ResourceManager) -> render_job::RenderItem<'a> {
		// First, resolve the static mesh from the resource registry
        let static_mesh = resources
            .meshes
            .get(&self.static_mesh)
            .expect("invalid static mesh handle");

		// Then resolve the material, which contains the specific parameters (what texture to bind, sampler, and other buffers) to draw this static mesh.
        let material = resources
            .materials
            .get(&self.material)
            .expect("invalid material handle");

		// Static meshes could contain multiple "submeshes", ex a car might have a body submesh with a paint material and a window submesh with another material.
		// They require different draw calls (and hence different draw items) but share the same vertex/index buffer.
		// They use different ranges of the buffer which we need to find.
        let vertex_buffers_with_ranges = static_mesh
            .vertex_buffers
            .iter()
            .zip(static_mesh.submeshes[self.submesh_idx].vertex_ranges.iter());

		// Combine the model and view uniforms with the material uniforms.
        let mut bind_group_refs = vec![&self.xform_bind_group];
        bind_group_refs.extend(material.bind_groups.values());

		// Then finally, create the RenderItem
        RenderItem::Graphics {
			// The pass name is used to find what shader (or more specifically, pipeline) to run. 
			// The pass should be registered with the renderer at the application start using renderer.register_pass()
            pass_name: material.pass_name.as_str(),
			// The framebuffer name is used to determine what texture(s) to draw on.
			// For example, should it draw directly on the screen ("surface", automatically registered) or on another previously registered framebuffer (registered with renderer.register_framebuffer())?
            framebuffer_name: "forward_out", 
            num_elements: static_mesh.submeshes[self.submesh_idx].num_elements, // The number of elements is usually just the number of triangles in this submesh
            vertex_buffers: vertex_buffers_with_ranges
                .map(|(buffer, range)| buffer.slice(*range))
                .collect::<Vec<wgpu::BufferSlice>>(),
            index_buffer: match &static_mesh.index_buffer {
                Some(buffer) => {
                    Some(buffer.slice(static_mesh.submeshes[self.submesh_idx].index_range.unwrap()))
                }
                None => None,
            },
            index_format: static_mesh.index_format,
            bind_group: bind_group_refs,
        }
    }
}
```

Post-processing and pre-processing passes can be added to the render job directly in `Application::render()` like so:
```rust
// ... merge the render graphs from the drawables...

// Assuming the toon shade technique was created when the application was initialized, create a render item from it and turn it into a graph
let toon_shade_graph = toon_shade_technique.render_item(&self.resources).to_graph();

// Then merge it into the render job after the forward pass
render_job.merge_graph_after("forward", toon_shade_graph);
```

### Registering passes and framebuffers
Render passes (kind of) contain an actual GPU program that is executed (WGPU calls them pipelines). These can be a vertex + fragment shader or a compute shader. 
WGPU also stores the "layout" of these programs which is just the format of the framebuffers/uniforms that should be bound so it can validate at run time that 
you're not doing anything incorrect.

They can be registered like so:
```rust
renderer.register_pass(
	"forward",
	&indirect_graphics_depth_pass!(
		include_str!("shader.wgsl"),
		wgpu::IndexFormat::Uint16,
		[wgpu::TextureFormat::Rgba16Float]
	),
);
```

There are a few macros for generating common pass descriptors. `indirect_graphics_depth_pass` generates a pass descriptor for 
a pass that draws to a custom framebuffer (not directly to the surface) and outputs depth.

A framebuffer is the set of textures a graphics pass draws to. In the simple case, a forward pass outputs just color and depth. 
However, some games also have gbuffer passes which output data (like normals, tangent, and base color) stored in several textures.
A framebuffer can be registered like this:
```rust
let (depth_tex, color_tex, fb_desc) =
	depth_color_framebuffer(&renderer, wgpu::TextureFormat::Rgba16Float);
renderer.register_framebuffer("forward_out", fb_desc, [depth_tex, color_tex]);
```
The renderer provides a framebuffer for drawing to the surface (`"surface"`) which also contains a depth buffer, so framebuffers only need to
be registered for drawing indirectly.

Some references on similar things:
 * [Unreal's thing](https://epicgames.ent.box.com/s/ul1h44ozs0t2850ug0hrohlzm53kxwrz)
 * [Unity's thing](https://enginearchitecture.realtimerendering.com/downloads/reac2021_unity_rendering_engine_architecture.pdf)
 * Unity's was presented as a part of a larger [architecture course](https://enginearchitecture.realtimerendering.com/2021_course/) at siggraph

## Low Level Stuff
todo
