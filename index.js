const GRID_SIZE = 32;
const UPDATE_INTERVAL = 250;
const WORKGROUP_SIZE = 8;

const canvas = document.querySelector("canvas");

// WebGPU device initialization
if (!navigator.gpu) {
    throw new Error("WebGPU not supported on this browser.");
}

const adapter = await navigator.gpu.requestAdapter();
if (!adapter) {
    throw new Error("No appropriate GPUAdapter found.");
}

const device = await adapter.requestDevice();

// Canvas configuration
const context = canvas.getContext("webgpu");
const canvasFormat = navigator.gpu.getPreferredCanvasFormat();
context.configure({
    device: device,
    format: canvasFormat,
});

// Create a buffer with the vertices for a single cell.
const vertices = new Float32Array([
    -0.8, -0.8,
    0.8, -0.8,
    0.8, 0.8,

    -0.8, -0.8,
    0.8, 0.8,
    -0.8, 0.8,
]);
const vertexBuffer = device.createBuffer({
    label: "Cell vertices",
    size: vertices.byteLength,
    usage: GPUBufferUsage.VERTEX | GPUBufferUsage.COPY_DST,
});
device.queue.writeBuffer(vertexBuffer, 0, vertices);

const vertexBufferLayout = {
    arrayStride: 8,
    attributes: [{
        format: "float32x2",
        offset: 0,
        shaderLocation: 0, // Position. Matches @location(0) in the @vertex shader.
    }],
};

// Create the bind group layout and pipeline layout.
const bindGroupLayout = device.createBindGroupLayout({
    label: "Cell Bind Group Layout",
    entries: [
        {
            binding: 0,
            visibility: GPUShaderStage.VERTEX | GPUShaderStage.COMPUTE,
            buffer: {}, // Grid uniform buffer
        }, 
        {
            binding: 1,
            visibility: GPUShaderStage.VERTEX | GPUShaderStage.COMPUTE,
            buffer: { type: "read-only-storage" }, // Cell state input buffer
        }, 
        {
            binding: 2,
            visibility: GPUShaderStage.COMPUTE,
            buffer: { type: "storage" }, // Cell state output buffer
        },
    ],
});

const pipelineLayout = device.createPipelineLayout({
    label: "Cell Pipeline Layout",
    bindGroupLayouts: [bindGroupLayout],
});

// Create the shader that will render the cells.
const cellShaderModule = device.createShaderModule({
    label: "Cell shader",
    code: `
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
    `,
});

// Create a pipeline that renders the cell.
const cellPipeline = device.createRenderPipeline({
    label: "Cell pipeline",
    layout: pipelineLayout,
    vertex: {
        module: cellShaderModule,
        entryPoint: "vert_main",
        buffers: [vertexBufferLayout],
    },
    fragment: {
        module: cellShaderModule,
        entryPoint: "frag_main",
        targets: [{
            format: canvasFormat,
        }],
    },
    primitive: {
        topology: "triangle-list",
    },
});

// Create the compute shader that will process the game of life simulation.
const simulationShaderModule = device.createShaderModule({
    label: "Life simulation shader",
    code: `
        @group(0) @binding(0) var<uniform> grid: vec2<f32>;

        @group(0) @binding(1) var<storage> cell_state_in: array<u32>;
        @group(0) @binding(2) var<storage, read_write> cell_state_out: array<u32>;

        fn cell_index(cell: vec2<u32>) -> u32 
        {
            return (cell.y % u32(grid.y)) * u32(grid.x) + (cell.x % u32(grid.x));
        }

        fn cell_active(x: u32, y: u32) -> u32 
        {
            return cell_state_in[cell_index(vec2<u32>(x, y))];
        }

        @compute @workgroup_size(${WORKGROUP_SIZE}, ${WORKGROUP_SIZE})
        fn comp_main(@builtin(global_invocation_id) cell: vec3<u32>)
        {
            // Determine how many active neighbors this cell has.
            let activeNeighbors = 
                cell_active(cell.x + 1u, cell.y + 1u) +
                cell_active(cell.x + 1u, cell.y) +
                cell_active(cell.x + 1u, cell.y - 1u) +
                cell_active(cell.x, cell.y - 1u) +
                cell_active(cell.x - 1u, cell.y - 1u) +
                cell_active(cell.x - 1u, cell.y) +
                cell_active(cell.x - 1u, cell.y + 1u) +
                cell_active(cell.x, cell.y + 1u);

            let i = cell_index(cell.xy);

            // Conway's game of life rules:
            switch activeNeighbors 
            {
                case 2u: // Active cells with 2 neighbors stay active.
                { 
                    cell_state_out[i] = cell_state_in[i];
                }
                case 3u: // Cells with 3 neighbors become or stay active.
                { 
                    cell_state_out[i] = 1u;
                }
                default: // Cells with < 2 or > 3 neighbors become inactive.
                {
                    cell_state_out[i] = 0u;
                }
            }
        }
    `,
});

// Create a compute pipeline that updates the game state.
const simulationPipeline = device.createComputePipeline({
    label: "Simulation pipeline",
    layout: pipelineLayout,
    compute: {
        module: simulationShaderModule,
        entryPoint: "comp_main",
    }
});

// Create a uniform buffer that describes the grid.
const uniformArray = new Float32Array([GRID_SIZE, GRID_SIZE]);
const uniformBuffer = device.createBuffer({
    label: "Grid Uniforms",
    size: uniformArray.byteLength,
    usage: GPUBufferUsage.UNIFORM | GPUBufferUsage.COPY_DST,
});
device.queue.writeBuffer(uniformBuffer, 0, uniformArray);

// Create an array representing the active state of each cell.
const cellStateArray = new Uint32Array(GRID_SIZE * GRID_SIZE);

// Create two storage buffers to hold the cell state.
const cellStateStorage = [
    device.createBuffer({
        label: "Cell State A",
        size: cellStateArray.byteLength,
        usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_DST,
    }),
    device.createBuffer({
        label: "Cell State B",
        size: cellStateArray.byteLength,
        usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_DST,
    }),
];

// Set each cell to a random state, then copy the JavaScript array into
// the storage buffer.
for (let i = 0; i < cellStateArray.length; ++i) {
    cellStateArray[i] = Math.random() > 0.6 ? 1 : 0;
}
device.queue.writeBuffer(cellStateStorage[0], 0, cellStateArray);

// Create a bind group to pass the grid uniforms into the pipeline
const bindGroups = [
    device.createBindGroup({
        label: "Cell renderer bind group A",
        layout: bindGroupLayout,
        entries: [
            {
                binding: 0,
                resource: { buffer: uniformBuffer },
            }, 
            {
                binding: 1,
                resource: { buffer: cellStateStorage[0] },
            }, 
            {
                binding: 2,
                resource: { buffer: cellStateStorage[1] },
            },
        ],
    }),
    device.createBindGroup({
        label: "Cell renderer bind group B",
        layout: bindGroupLayout,
        entries: [
            {
                binding: 0,
                resource: { buffer: uniformBuffer },
            }, 
            {
                binding: 1,
                resource: { buffer: cellStateStorage[1] },
            }, 
            {
                binding: 2,
                resource: { buffer: cellStateStorage[0] },
            },
        ],
    }),
];

let step = 0;
function updateGrid() {
    const encoder = device.createCommandEncoder();

    // Start a compute pass
    const computePass = encoder.beginComputePass();

    computePass.setPipeline(simulationPipeline);
    computePass.setBindGroup(0, bindGroups[step % 2]);
    const workgroupCount = Math.ceil(GRID_SIZE / WORKGROUP_SIZE);
    computePass.dispatchWorkgroups(workgroupCount, workgroupCount);
    computePass.end();

    step++; // Increment the step count

    // Start a render pass
    const renderPass = encoder.beginRenderPass({
        colorAttachments: [{
            view: context.getCurrentTexture().createView(),
            loadOp: "clear",
            clearValue: { r: 0, g: 0, b: 0.4, a: 1.0 },
            storeOp: "store",
        }],
    });

    // Draw the grid.
    renderPass.setPipeline(cellPipeline);
    renderPass.setBindGroup(0, bindGroups[step % 2]);
    renderPass.setVertexBuffer(0, vertexBuffer);
    renderPass.draw(vertices.length / 2, GRID_SIZE * GRID_SIZE);

    // End the render pass and submit the command buffer
    renderPass.end();
    device.queue.submit([encoder.finish()]);
}
setInterval(updateGrid, UPDATE_INTERVAL);
