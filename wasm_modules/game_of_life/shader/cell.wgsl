struct VertexOutput 
{
    @builtin(position) position: vec4<f32>,
    @location(0) cell: vec2<f32>,
};

@group(0) @binding(0) var<uniform> grid: vec2<f32>;
@group(0) @binding(1) var<storage> cell_state: array<u32>;

@vertex
fn vert_main(
    @location(0) position: vec2<f32>, @builtin(instance_index) instance: u32,
)
    -> VertexOutput
{
    var output: VertexOutput;

    let i = f32(instance);
    let cell = vec2<f32>(i % grid.x, floor(i / grid.x));

    let scale = f32(cell_state[instance]);
    let cell_offset = cell / grid * 2.0;
    let grid_pos = (position * scale + 1.0) / grid - 1.0 + cell_offset;

    output.position = vec4<f32>(grid_pos, 0.0, 1.0);
    output.cell = cell / grid;
    return output;
}

@fragment
fn frag_main(input: VertexOutput) -> @location(0) vec4<f32>
{
    return vec4<f32>(input.cell, 1.0 - input.cell.x, 1.0);
}
