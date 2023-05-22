import { initializeGameOfLife, updateGrid } from "./workers/game_of_life_worker.js";

const GRID_SIZE = 64;
const UPDATE_INTERVAL = 100;
const WORKGROUP_SIZE = 8;

const canvas = document.getElementById("wasm-js-worker");
if (!canvas) {
    throw new Error("Canvas not found.");
}

document.addEventListener("workerMessage", (event) => console.log(event.detail.message));

setTimeout ( () => initializeGameOfLife(GRID_SIZE, WORKGROUP_SIZE, canvas), 500);

setTimeout ( () => {
    let step = 0;
    function updGrd() {
        const compState = step % 2;
        step++;
        const rendState = step % 2;
        updateGrid(compState, rendState);
    }
    setInterval(updGrd, UPDATE_INTERVAL);
}, 1000);
