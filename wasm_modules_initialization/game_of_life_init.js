import init, { GameOfLife } from "../wasm/game_of_life.js";


export async function initializeGameOfLife(gridSize, workgroupSize, device, context, canvasFormat) {
    await init();
    const game_of_life = GameOfLife.create(gridSize, workgroupSize, device, context, canvasFormat);
    return game_of_life;    
}
