import init, { GameOfLife } from "../wasm/game_of_life.js";


export async function initializeGameOfLife(device, context, canvasFormat) {
    await init();
    const game_of_life = GameOfLife.create(device, context, canvasFormat);
    return game_of_life;    
}
