import { initializeGameOfLife } from "./wasm_modules_initialization/game_of_life_init.js";


const UPDATE_INTERVAL = 250;


const canvas = document.querySelector("canvas");
if (!canvas) {
    throw new Error("Canvas not found.");
}

// WebGPU device initialization
if (!navigator.gpu) {
    throw new Error("WebGPU not supported on this browser.");
}
const adapter = await navigator.gpu.requestAdapter();
if (!adapter) {
    throw new Error("No appropriate GPUAdapter found.");
}
const device = await adapter.requestDevice();
const context = canvas.getContext("webgpu");
const canvasFormat = navigator.gpu.getPreferredCanvasFormat();
context.configure({
    device: device,
    format: canvasFormat,
});

const gameOfLife = await initializeGameOfLife(device, context, canvasFormat);

let step = 0;
function updateGrid() {
    const compState = step % 2;
    step++;
    const rendState = step % 2;
    gameOfLife.update_grid(compState, rendState);
}
setInterval(updateGrid, UPDATE_INTERVAL);
