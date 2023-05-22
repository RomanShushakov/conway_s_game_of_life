use wasm_bindgen::prelude::wasm_bindgen;

use web_sys::
{
    GpuDevice, GpuCanvasContext, GpuTextureFormat, GpuShaderModuleDescriptor, GpuShaderModule, GpuProgrammableStage,
    GpuBindGroupLayoutEntry, GpuBufferBindingLayout, GpuBufferBindingType, GpuBindGroupLayoutDescriptor, 
    GpuBindGroupLayout, GpuPipelineLayoutDescriptor, GpuPipelineLayout, GpuComputePipelineDescriptor, 
    GpuComputePipeline, GpuBindGroup, GpuBindGroupDescriptor, GpuBindGroupEntry, GpuBufferBinding, GpuBuffer,
    GpuBufferDescriptor, GpuQueue, GpuVertexState, GpuVertexBufferLayout, GpuVertexAttribute, GpuVertexFormat,
    GpuRenderPipelineDescriptor, GpuRenderPipeline,GpuFragmentState, GpuColorTargetState, GpuCommandEncoder,
    GpuComputePassEncoder, GpuRenderPassEncoder, GpuRenderPassDescriptor, GpuRenderPassColorAttachment, GpuLoadOp,
    GpuStoreOp, GpuTexture, GpuTextureView, GpuPrimitiveTopology, GpuPrimitiveState, GpuCommandBuffer, GpuColorDict,
};

use web_sys::gpu_shader_stage::{VERTEX, COMPUTE};

use web_sys::gpu_buffer_usage::{UNIFORM, COPY_DST, STORAGE};

use js_sys::{Float32Array, Uint32Array};

use rand::{thread_rng, Rng};


#[wasm_bindgen]
extern "C"
{
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(value: &str);
}


fn define_bind_group_layout(gpu_device: &GpuDevice) -> GpuBindGroupLayout
{
    let mut bind_group_layout_entry_0 = GpuBindGroupLayoutEntry::new(0, VERTEX | COMPUTE);
    let mut bind_group_layout_entry_0_buffer = GpuBufferBindingLayout::new();
    bind_group_layout_entry_0_buffer.type_(GpuBufferBindingType::Uniform);
    bind_group_layout_entry_0.buffer(&bind_group_layout_entry_0_buffer);

    let mut bind_group_layout_entry_1 = GpuBindGroupLayoutEntry::new(1, VERTEX | COMPUTE);
    let mut bind_group_layout_entry_1_buffer = GpuBufferBindingLayout::new();
    bind_group_layout_entry_1_buffer.type_(GpuBufferBindingType::ReadOnlyStorage);
    bind_group_layout_entry_1.buffer(&bind_group_layout_entry_1_buffer);

    let mut bind_group_layout_entry_2 = GpuBindGroupLayoutEntry::new(2, COMPUTE);
    let mut bind_group_layout_entry_2_buffer = GpuBufferBindingLayout::new();
    bind_group_layout_entry_2_buffer.type_(GpuBufferBindingType::Storage);
    bind_group_layout_entry_2.buffer(&bind_group_layout_entry_2_buffer);

    let bind_group_layout_entries = [
        bind_group_layout_entry_0, bind_group_layout_entry_1, bind_group_layout_entry_2,
    ].iter().collect::<js_sys::Array>();

    let mut bind_group_layout_descriptor = GpuBindGroupLayoutDescriptor::new(&bind_group_layout_entries);
    bind_group_layout_descriptor.label("Cell Bind Group Layout");
    let bind_group_layout = gpu_device.create_bind_group_layout(&bind_group_layout_descriptor);

    bind_group_layout
}


fn define_pipeline_layout(gpu_device: &GpuDevice, bind_group_layout: &GpuBindGroupLayout) -> GpuPipelineLayout
{
    let bind_group_layouts = [bind_group_layout].iter().collect::<js_sys::Array>();

    let mut pipeline_layout_descriptor = GpuPipelineLayoutDescriptor::new(&bind_group_layouts);
    pipeline_layout_descriptor.label("Cell Pipeline Layout");
    let pipeline_layout = gpu_device.create_pipeline_layout(&pipeline_layout_descriptor);

    pipeline_layout
}


fn define_simulation_pipeline(
    workgroup_size: usize, gpu_device: &GpuDevice, pipeline_layout: &GpuPipelineLayout,
) 
    -> GpuComputePipeline
{
    let mut simulation_shader_module_descriptor = GpuShaderModuleDescriptor::new(
        &include_str!("../shader/simulate.wgsl").replace("${WORKGROUP_SIZE}", &workgroup_size.to_string()),
    );
    simulation_shader_module_descriptor.label("Life simulation shader");
    let gpu_shader_module = gpu_device.create_shader_module(&simulation_shader_module_descriptor);
    let gpu_programmable_stage = GpuProgrammableStage::new("comp_main", &gpu_shader_module);

    let mut gpu_compute_pipeline_descriptor = GpuComputePipelineDescriptor::new(
        pipeline_layout, &gpu_programmable_stage,
    );
    gpu_compute_pipeline_descriptor.label("Simulation pipeline");
    let gpu_compute_pipeline = gpu_device.create_compute_pipeline(&gpu_compute_pipeline_descriptor);

    gpu_compute_pipeline
}


fn define_bind_groups(
    grid_size: usize, gpu_device: &GpuDevice, bind_group_layout: &GpuBindGroupLayout,
) 
    -> [GpuBindGroup; 2]
{
    let mut uniform_array = Float32Array::new_with_length([grid_size, grid_size].len() as u32);
    uniform_array.copy_from(&[grid_size as f32, grid_size as f32]);
    let mut uniform_buffer_descriptor = GpuBufferDescriptor::new(
        uniform_array.byte_length().into(),
        UNIFORM | COPY_DST,
    );
    uniform_buffer_descriptor.label("Grid Uniforms");
    let uniform_buffer = gpu_device.create_buffer(&uniform_buffer_descriptor);
    gpu_device.queue().write_buffer_with_u32_and_buffer_source(&uniform_buffer, 0, &uniform_array);

    let mut cell_state = Vec::new();
    for i in 0..grid_size * grid_size
    {
        let rnd = thread_rng().gen_range(0u32..2);
        cell_state.push(rnd);
    }
    let cell_state_array = Uint32Array::new_with_length((grid_size * grid_size) as u32);
    cell_state_array.copy_from(&cell_state);
    let mut cell_state_a_storage_descriptor = GpuBufferDescriptor::new(
        cell_state_array.byte_length().into(),
        STORAGE | COPY_DST,
    );
    cell_state_a_storage_descriptor.label("Cell State A");
    let cell_state_a_storage = gpu_device.create_buffer(&cell_state_a_storage_descriptor);
    let mut cell_state_b_storage_descriptor = GpuBufferDescriptor::new(
        cell_state_array.byte_length().into(),
        STORAGE | COPY_DST,
    );
    cell_state_b_storage_descriptor.label("Cell State B");
    let cell_state_b_storage = gpu_device.create_buffer(&cell_state_b_storage_descriptor);

    let cell_state_storage = [cell_state_a_storage, cell_state_b_storage];
    gpu_device.queue().write_buffer_with_u32_and_buffer_source(&cell_state_storage[0], 0, &cell_state_array);

    let bind_group_a_entry_0_resource = GpuBufferBinding::new(&uniform_buffer);
    let bind_group_a_entry_0 = GpuBindGroupEntry::new(0, &bind_group_a_entry_0_resource);
    let bind_group_a_entry_1_resource = GpuBufferBinding::new(&cell_state_storage[0]);
    let bind_group_a_entry_1 = GpuBindGroupEntry::new(1, &bind_group_a_entry_1_resource);
    let bind_group_a_entry_2_resource = GpuBufferBinding::new(&cell_state_storage[1]);
    let bind_group_a_entry_2 = GpuBindGroupEntry::new(2, &bind_group_a_entry_2_resource);

    let bind_group_a_entries = [
        bind_group_a_entry_0, bind_group_a_entry_1, bind_group_a_entry_2,
    ].iter().collect::<js_sys::Array>();
    let mut bind_group_a_descriptor = GpuBindGroupDescriptor::new(&bind_group_a_entries, &bind_group_layout);
    bind_group_a_descriptor.label("Cell renderer bind group A");
    let bind_group_a = gpu_device.create_bind_group(&bind_group_a_descriptor);

    let bind_group_b_entry_0_resource = GpuBufferBinding::new(&uniform_buffer);
    let bind_group_b_entry_0 = GpuBindGroupEntry::new(0, &bind_group_b_entry_0_resource);
    let bind_group_b_entry_1_resource = GpuBufferBinding::new(&cell_state_storage[1]);
    let bind_group_b_entry_1 = GpuBindGroupEntry::new(1, &bind_group_b_entry_1_resource);
    let bind_group_b_entry_2_resource = GpuBufferBinding::new(&cell_state_storage[0]);
    let bind_group_b_entry_2 = GpuBindGroupEntry::new(2, &bind_group_b_entry_2_resource);

    let bind_group_b_entries = [
        bind_group_b_entry_0, bind_group_b_entry_1, bind_group_b_entry_2,
    ].iter().collect::<js_sys::Array>();
    let mut bind_group_b_descriptor = GpuBindGroupDescriptor::new(&bind_group_b_entries, &bind_group_layout);
    bind_group_b_descriptor.label("Cell renderer bind group B");
    let bind_group_b = gpu_device.create_bind_group(&bind_group_b_descriptor);

    [bind_group_a, bind_group_b]
}


fn define_cell_pipeline(
    gpu_device: &GpuDevice, canvas_format: GpuTextureFormat, pipeline_layout: &GpuPipelineLayout,
) 
    -> GpuRenderPipeline
{
    let mut cell_shader_module_descriptor = GpuShaderModuleDescriptor::new(include_str!("../shader/cell.wgsl"));
    cell_shader_module_descriptor.label("Cell shader");
    let cell_shader_module = gpu_device.create_shader_module(&cell_shader_module_descriptor);

    let mut gpu_vertex_state = GpuVertexState::new("vert_main", &cell_shader_module);
    let vertex_attribute = GpuVertexAttribute::new(GpuVertexFormat::Float32x2, 0f64, 0u32);
    let vertex_buffer_layout_attributes = [vertex_attribute].iter().collect::<js_sys::Array>();
    let vertex_buffer_layout = GpuVertexBufferLayout::new(8f64, &vertex_buffer_layout_attributes);
    let vertex_state_buffers = [vertex_buffer_layout].iter().collect::<js_sys::Array>();
    gpu_vertex_state.buffers(&vertex_state_buffers);

    let color_target_state = GpuColorTargetState::new(canvas_format);
    let fragment_state_targets = [color_target_state].iter().collect::<js_sys::Array>();
    let gpu_fragment_state = GpuFragmentState::new("frag_main", &cell_shader_module, &fragment_state_targets);

    let mut gpu_render_pipeline_descriptor = GpuRenderPipelineDescriptor::new(
        pipeline_layout, &gpu_vertex_state,
    );
    gpu_render_pipeline_descriptor.label("Cell pipeline");
    gpu_render_pipeline_descriptor.fragment(&gpu_fragment_state);
    let mut gpu_primitive_state = GpuPrimitiveState::new();
    gpu_primitive_state.topology(GpuPrimitiveTopology::TriangleList);
    gpu_render_pipeline_descriptor.primitive(&gpu_primitive_state);
    let gpu_render_pipeline = gpu_device.create_render_pipeline(&gpu_render_pipeline_descriptor);

    gpu_render_pipeline
}


fn define_vertex_buffer(gpu_device: &GpuDevice) -> GpuBuffer
{
    let vertices = [
        -0.8f32, -0.8, 0.8, -0.8, 0.8, 0.8,
        -0.8, -0.8, 0.8, 0.8, -0.8, 0.8,
    ];

    let mut vertices_array = Float32Array::new_with_length(vertices.len() as u32);
    vertices_array.copy_from(&vertices);

    let mut vertex_buffer_descriptor = GpuBufferDescriptor::new(
        vertices_array.byte_length().into(),
        web_sys::gpu_buffer_usage::VERTEX | COPY_DST,
    );
    vertex_buffer_descriptor.label("Cell vertices");
    let vertex_buffer = gpu_device.create_buffer(&vertex_buffer_descriptor);

    gpu_device.queue().write_buffer_with_u32_and_buffer_source(&vertex_buffer, 0, &vertices_array);

    vertex_buffer
}


struct Props
{
    grid_size: usize,
    workgroup_size: usize,
}


#[wasm_bindgen]
pub struct GameOfLife 
{
    props: Props,
    gpu_device: GpuDevice,
    context: GpuCanvasContext,
    canvas_format: GpuTextureFormat,
    simulation_pipeline: GpuComputePipeline,
    cell_pipeline: GpuRenderPipeline,
    vertex_buffer: GpuBuffer,
    bind_groups: [GpuBindGroup; 2],
}


#[wasm_bindgen]
impl GameOfLife
{
    pub fn create(
        grid_size: usize,
        workgroup_size: usize,
        gpu_device: GpuDevice, 
        context: GpuCanvasContext, 
        canvas_format: GpuTextureFormat,
    ) 
        -> GameOfLife
    {
        let props = Props { grid_size, workgroup_size };

        let bind_group_layout = define_bind_group_layout(&gpu_device);
        let pipeline_layout = define_pipeline_layout(&gpu_device, &bind_group_layout);
        let simulation_pipeline = define_simulation_pipeline(workgroup_size, &gpu_device, &pipeline_layout);
        let cell_pipeline = define_cell_pipeline(&gpu_device, canvas_format, &pipeline_layout);
        let vertex_buffer = define_vertex_buffer(&gpu_device);
        let bind_groups = define_bind_groups(grid_size, &gpu_device, &bind_group_layout);

        GameOfLife 
        { 
            props, gpu_device, context, canvas_format, simulation_pipeline, cell_pipeline, vertex_buffer, bind_groups,
        }
    }


    pub fn update_grid(&self, comp_state: usize, rend_state: usize)
    {
        let encoder = self.gpu_device.create_command_encoder();

        let compute_pass = encoder.begin_compute_pass();

        compute_pass.set_pipeline(&self.simulation_pipeline);
        compute_pass.set_bind_group(0, &self.bind_groups[comp_state]);
        let workgroup_count = (self.props.grid_size / self.props.workgroup_size) as u32;
        compute_pass.dispatch_workgroups_with_workgroup_count_y(workgroup_count, workgroup_count);
        compute_pass.end();

        let mut color_attachment = GpuRenderPassColorAttachment::new(
            GpuLoadOp::Clear, GpuStoreOp::Store, &self.context.get_current_texture().create_view(),
        );
        color_attachment.clear_value(&GpuColorDict::new(1.0, 0.4, 0.0, 0.0));
        let color_attachments = [color_attachment].iter().collect::<js_sys::Array>();
        let render_pass_descriptor = GpuRenderPassDescriptor::new(&color_attachments);
        let render_pass = encoder.begin_render_pass(&render_pass_descriptor);
        render_pass.set_pipeline(&self.cell_pipeline);
        render_pass.set_bind_group(0, &self.bind_groups[rend_state]);
        render_pass.set_vertex_buffer(0u32, &self.vertex_buffer);
        render_pass.draw_with_instance_count(6, (self.props.grid_size * self.props.grid_size) as u32);
        render_pass.end();

        self.gpu_device.queue().submit(&[encoder.finish()].iter().collect::<js_sys::Array>());
    }
}
