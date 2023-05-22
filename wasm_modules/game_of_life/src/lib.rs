use wasm_bindgen::prelude::wasm_bindgen;

use web_sys::
{
    GpuDevice, GpuCanvasContext, GpuTextureFormat, GpuShaderModuleDescriptor, GpuShaderModule, GpuProgrammableStage,
    GpuBindGroupLayoutEntry, GpuBufferBindingLayout, GpuBufferBindingType, GpuBindGroupLayoutDescriptor, 
    GpuBindGroupLayout, GpuPipelineLayoutDescriptor, GpuPipelineLayout, GpuComputePipelineDescriptor, 
    GpuComputePipeline, GpuBindGroup, GpuBindGroupDescriptor, GpuBindGroupEntry, GpuBufferBinding, GpuBuffer,
    GpuBufferDescriptor, GpuQueue,
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


const GRID_SIZE: usize = 8;


#[wasm_bindgen]
pub struct GameOfLife 
{
    gpu_device: GpuDevice,
    context: GpuCanvasContext,
    canvas_format: GpuTextureFormat,
}


#[wasm_bindgen]
impl GameOfLife
{
    pub fn create(
        gpu_device: GpuDevice, context: GpuCanvasContext, canvas_format: GpuTextureFormat,
    ) 
        -> GameOfLife
    {
        GameOfLife { gpu_device, context, canvas_format }
    }


    fn define_bind_group_layout(&self) -> GpuBindGroupLayout
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
        let bind_group_layout = self.gpu_device.create_bind_group_layout(&bind_group_layout_descriptor);

        bind_group_layout
    }


    fn define_simulation_pipeline(&self, bind_group_layout: &GpuBindGroupLayout) -> GpuComputePipeline
    {
        let bind_group_layouts = [bind_group_layout].iter().collect::<js_sys::Array>();

        let mut pipeline_layout_descriptor = GpuPipelineLayoutDescriptor::new(&bind_group_layouts);
        pipeline_layout_descriptor.label("Cell Pipeline Layout");
        let pipeline_layout = self.gpu_device.create_pipeline_layout(&pipeline_layout_descriptor);

        let mut gpu_shader_module_descriptor = GpuShaderModuleDescriptor::new(include_str!("../shader/simulate.wgsl"));
        gpu_shader_module_descriptor.label("Life simulation shader");
        let gpu_shader_module = self.gpu_device.create_shader_module(&gpu_shader_module_descriptor);
        let gpu_programmable_stage = GpuProgrammableStage::new("comp_main", &gpu_shader_module);

        let mut gpu_compute_pipeline_descriptor = GpuComputePipelineDescriptor::new(&pipeline_layout, &gpu_programmable_stage);
        gpu_compute_pipeline_descriptor.label("Simulation pipeline");
        let gpu_compute_pipeline = self.gpu_device.create_compute_pipeline(&gpu_compute_pipeline_descriptor);

        gpu_compute_pipeline
    }


    fn define_bind_groups(&self, bind_group_layout: &GpuBindGroupLayout) -> [GpuBindGroup; 2]
    {
        let mut uniform_array = Float32Array::new_with_length(
            ([GRID_SIZE, GRID_SIZE].len() * std::mem::size_of::<f32>()) as u32,
        );
        uniform_array.copy_from(&[GRID_SIZE as f32, GRID_SIZE as f32]);
        let mut uniform_buffer_descriptor = GpuBufferDescriptor::new(
            uniform_array.byte_length().into(),
            UNIFORM | COPY_DST,
        );
        uniform_buffer_descriptor.label("Grid Uniforms");
        let uniform_buffer = self.gpu_device.create_buffer(&uniform_buffer_descriptor);
        self.gpu_device.queue().write_buffer_with_u32_and_buffer_source(&uniform_buffer, 0, &uniform_array);

        let mut cell_state = [0u32; GRID_SIZE * GRID_SIZE];
        for i in 0..cell_state.len()
        {
            let rnd = thread_rng().gen_range(0u32..2);
            cell_state[i] = rnd;
        }
        let cell_state_array = Uint32Array::new_with_length(
            (GRID_SIZE * GRID_SIZE * std::mem::size_of::<u32>()) as u32,
        );
        cell_state_array.copy_from(&cell_state);
        let mut cell_state_a_storage_descriptor = GpuBufferDescriptor::new(
            cell_state_array.byte_length().into(),
            STORAGE | COPY_DST,
        );
        cell_state_a_storage_descriptor.label("Cell State A");
        let cell_state_a_storage = self.gpu_device.create_buffer(&cell_state_a_storage_descriptor);
        let mut cell_state_b_storage_descriptor = GpuBufferDescriptor::new(
            cell_state_array.byte_length().into(),
            STORAGE | COPY_DST,
        );
        cell_state_b_storage_descriptor.label("Cell State B");
        let cell_state_b_storage = self.gpu_device.create_buffer(&cell_state_b_storage_descriptor);
        let cell_state_storage = [cell_state_a_storage, cell_state_b_storage];
        self.gpu_device.queue().write_buffer_with_u32_and_buffer_source(&cell_state_storage[0], 0, &cell_state_array);

        let bind_group_a_entry_0_resource = GpuBufferBinding::new(&uniform_buffer);
        let bind_group_a_entry_0 = GpuBindGroupEntry::new(0, &bind_group_a_entry_0_resource);
        let bind_group_a_entry_1_resource = GpuBufferBinding::new(&cell_state_storage[0]);
        let bind_group_a_entry_1 = GpuBindGroupEntry::new(1, &bind_group_a_entry_1_resource);
        let bind_group_a_entry_2_resource = GpuBufferBinding::new(&cell_state_storage[1]);
        let bind_group_a_entry_2 = GpuBindGroupEntry::new(1, &bind_group_a_entry_2_resource);

        let bind_group_a_entries = [
            bind_group_a_entry_0, bind_group_a_entry_1, bind_group_a_entry_2,
        ].iter().collect::<js_sys::Array>();
        let mut bind_group_a_descriptor = GpuBindGroupDescriptor::new(&bind_group_a_entries, &bind_group_layout);
        bind_group_a_descriptor.label("Cell renderer bind group A");
        let bind_group_a = self.gpu_device.create_bind_group(&bind_group_a_descriptor);

        let bind_group_b_entry_0_resource = GpuBufferBinding::new(&uniform_buffer);
        let bind_group_b_entry_0 = GpuBindGroupEntry::new(0, &bind_group_b_entry_0_resource);
        let bind_group_b_entry_1_resource = GpuBufferBinding::new(&cell_state_storage[1]);
        let bind_group_b_entry_1 = GpuBindGroupEntry::new(1, &bind_group_b_entry_1_resource);
        let bind_group_b_entry_2_resource = GpuBufferBinding::new(&cell_state_storage[0]);
        let bind_group_b_entry_2 = GpuBindGroupEntry::new(1, &bind_group_b_entry_2_resource);

        let bind_group_b_entries = [
            bind_group_b_entry_0, bind_group_b_entry_1, bind_group_b_entry_2,
        ].iter().collect::<js_sys::Array>();
        let mut bind_group_b_descriptor = GpuBindGroupDescriptor::new(&bind_group_b_entries, &bind_group_layout);
        bind_group_b_descriptor.label("Cell renderer bind group B");
        let bind_group_b = self.gpu_device.create_bind_group(&bind_group_b_descriptor);

        [bind_group_a, bind_group_b]
    }
}
