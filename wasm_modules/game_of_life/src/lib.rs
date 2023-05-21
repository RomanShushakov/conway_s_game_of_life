use wasm_bindgen::prelude::wasm_bindgen;

use web_sys::
{
    GpuDevice, GpuCanvasContext, GpuTextureFormat, GpuShaderModuleDescriptor, GpuShaderModule,
    GpuProgrammableStage, GpuBindGroupLayoutEntry, GpuBufferBindingLayout, GpuBufferBindingType,
    GpuBindGroupLayoutDescriptor, GpuBindGroupLayout, GpuPipelineLayoutDescriptor, 
    GpuPipelineLayout, GpuComputePipelineDescriptor, GpuComputePipeline,
};

use web_sys::gpu_shader_stage::{VERTEX, COMPUTE};


#[wasm_bindgen]
extern "C"
{
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(value: &str);
}


const GRID_SIZE: u8 = 8;


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


    pub fn define_simulation_pipeline(&self)
    // fn define_simulation_pipeline(&self) -> GpuComputePipeline
    {
        let mut gpu_shader_module_descriptor = GpuShaderModuleDescriptor::new(include_str!("../shader/simulate.wgsl"));
        gpu_shader_module_descriptor.label("Life simulation shader");
        let gpu_shader_module = self.gpu_device.create_shader_module(&gpu_shader_module_descriptor);
    
        let gpu_programmable_stage = GpuProgrammableStage::new("comp_main", &gpu_shader_module);

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
        let bind_group_layouts = [bind_group_layout].iter().collect::<js_sys::Array>();

        let mut pipeline_layout_descriptor = GpuPipelineLayoutDescriptor::new(&bind_group_layouts);
        pipeline_layout_descriptor.label("Cell Pipeline Layout");
        let pipeline_layout = self.gpu_device.create_pipeline_layout(&pipeline_layout_descriptor);

        let mut gpu_compute_pipeline_descriptor = GpuComputePipelineDescriptor::new(&pipeline_layout, &gpu_programmable_stage);
        gpu_compute_pipeline_descriptor.label("Simulation pipeline");
        let gpu_compute_pipeline = self.gpu_device.create_compute_pipeline(&gpu_compute_pipeline_descriptor);

        // gpu_compute_pipeline
    }
}
