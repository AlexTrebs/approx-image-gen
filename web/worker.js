// Web Worker for running the genetic algorithm

import init, { WasmAlgorithm } from '../pkg/approx_image_gen.js';

let algorithm = null;
let running = false;
let batchSize = 10;
let maxIterations = 50000;
let targetAccuracy = 0.9;

// OffscreenCanvas for zero-copy rendering
let offscreen = null;
let offscreenCtx = null;

// Throttle updates (ms)
const UPDATE_INTERVAL = 50;
let lastUpdateTime = 0;

// Initialize WASM module
async function initWasm() {
    try {
        await init();
        self.postMessage({ type: 'ready' });
    } catch (e) {
        self.postMessage({ type: 'error', data: e.message });
    }
}

// Run algorithm loop
async function runLoop() {
    while (running && algorithm && !algorithm.is_finished()) {
        // Run a batch of iterations
        const pixels = algorithm.step(batchSize);
        const iteration = algorithm.get_iteration();
        const accuracy = algorithm.get_accuracy();
        const width = algorithm.get_width();
        const height = algorithm.get_height();

        const now = performance.now();
        const shouldUpdate = now - lastUpdateTime >= UPDATE_INTERVAL;

        if (shouldUpdate || algorithm.is_finished()) {
            lastUpdateTime = now;

            // Render to OffscreenCanvas
            const imageData = new ImageData(new Uint8ClampedArray(pixels), width, height);
            offscreenCtx.putImageData(imageData, 0, 0);

            // Create transferable bitmap
            const bitmap = offscreen.transferToImageBitmap();

            // Send progress update with bitmap (zero-copy transfer)
            self.postMessage({
                type: algorithm.is_finished() ? 'finished' : 'progress',
                data: {
                    iteration,
                    accuracy,
                    bitmap,
                    maxIterations,
                    targetAccuracy
                }
            }, [bitmap]);
        }

        // Yield to allow message processing
        await new Promise(resolve => setTimeout(resolve, 0));
    }

    if (algorithm && algorithm.is_finished() && running) {
        const pixels = algorithm.step(0);
        const width = algorithm.get_width();
        const height = algorithm.get_height();

        const imageData = new ImageData(new Uint8ClampedArray(pixels), width, height);
        offscreenCtx.putImageData(imageData, 0, 0);
        const bitmap = offscreen.transferToImageBitmap();

        self.postMessage({
            type: 'finished',
            data: {
                iteration: algorithm.get_iteration(),
                accuracy: algorithm.get_accuracy(),
                bitmap,
                maxIterations,
                targetAccuracy
            }
        }, [bitmap]);
    }
}

// Handle messages from main thread
self.onmessage = async (e) => {
    const { type, data } = e.data;

    switch (type) {
        case 'start':
            const { pixels, width, height } = data;
            maxIterations = data.maxIterations;
            targetAccuracy = data.targetAccuracy;
            batchSize = data.batchSize;
            const algorithmType = data.algorithm || 0;

            // Create OffscreenCanvas
            offscreen = new OffscreenCanvas(width, height);
            offscreenCtx = offscreen.getContext('2d');

            // Create algorithm instance
            algorithm = new WasmAlgorithm(
                new Uint8Array(pixels),
                width,
                height,
                maxIterations,
                targetAccuracy,
                algorithmType
            );

            running = true;
            lastUpdateTime = 0;
            runLoop();
            break;

        case 'stop':
            running = false;
            break;
    }
};

// Initialize on load
initWasm();
