const worker = new Worker("../wasm_modules_initialization/game_of_life_worker_init.js");

export const initializeGameOfLife = (gridSize, workgroupSize, canvas) => {
    const handleInitializeGameOfLife = (message) => {
        document.dispatchEvent(new CustomEvent("workerMessage", {
            bubbles: true,
            composed: true,
            detail: { 
                header: message.data.header,
                status: message.data.status,
                message: message.data.message,
            },
        }));
        worker.removeEventListener("message", handleInitializeGameOfLife);
    }
    worker.addEventListener("message", handleInitializeGameOfLife, false);

    const canvasWorker = canvas.transferControlToOffscreen();

    worker.postMessage({
        header: "initializeGameOfLife",
        gridSize: gridSize,
        workgroupSize: workgroupSize,
        canvasWorker: canvasWorker,
    }, [canvasWorker]);
}


export const updateGrid = (compState, rendState) => {
    const handleUpdateGrid = (message) => {
        document.dispatchEvent(new CustomEvent("workerMessage", {
            bubbles: true,
            composed: true,
            detail: { 
                header: message.data.header,
                status: message.data.status,
                message: message.data.message,
            },
        }));
        worker.removeEventListener("message", handleUpdateGrid);
    }
    worker.addEventListener("message", handleUpdateGrid, false);

    worker.postMessage({ 
        header: "updateGrid",
        compState: compState,
        rendState: rendState,
    });
}
