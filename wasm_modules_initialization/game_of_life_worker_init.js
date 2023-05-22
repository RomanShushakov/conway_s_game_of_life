importScripts("../wasm/game_of_life_worker.js");


const { GameOfLife } = wasm_bindgen;

let gameOfLife;

async function init_wasm_in_worker() {
    await wasm_bindgen("../wasm/game_of_life_worker_bg.wasm");

    this.addEventListener("message", async (event) => {
        const header = event.data.header;

        if (header === "initializeGameOfLife") {
            const gridSize = event.data.gridSize;
            const workgroupSize = event.data.workgroupSize;
            const canvasWorker = event.data.canvasWorker;

            // WebGPU device initialization
            if (!navigator.gpu) {
                throw new Error("WebGPU not supported on this browser.");
            }
            const adapter = await navigator.gpu.requestAdapter();
            if (!adapter) {
                throw new Error("No appropriate GPUAdapter found.");
            }
            const device = await adapter.requestDevice();
            const context = canvasWorker.getContext("webgpu");
            const canvasFormat = navigator.gpu.getPreferredCanvasFormat();
            context.configure({
                device: device,
                format: canvasFormat,
            });

            gameOfLife = GameOfLife.create(gridSize, workgroupSize, device, context, canvasFormat);

            this.postMessage({ 
                header: header, 
                status: "completed", 
                message: "Game of Life has been successfully created",
            });
        }

        if (header === "updateGrid") {

            const compState = event.data.compState;
            const rendState = event.data.rendState;

            gameOfLife.update_grid(compState, rendState);
        }
    });

}

init_wasm_in_worker();
