// DOM Elements
const imageUpload = document.getElementById('image-upload');
const fileName = document.getElementById('file-name');
const startBtn = document.getElementById('start-btn');
const stopBtn = document.getElementById('stop-btn');
const algorithmSelect = document.getElementById('algorithm');
const maxIterationsInput = document.getElementById('max-iterations');
const targetAccuracyInput = document.getElementById('target-accuracy');
const batchSizeInput = document.getElementById('batch-size');
const progressFill = document.getElementById('progress-fill');
const statusEl = document.getElementById('status');
const iterationEl = document.getElementById('iteration');
const accuracyEl = document.getElementById('accuracy');
const originalCanvas = document.getElementById('original-canvas');
const approxCanvas = document.getElementById('approx-canvas');
const originalCtx = originalCanvas.getContext('2d');
const approxCtx = approxCanvas.getContext('2d');

// State
let worker = null;
let imageData = null;
let isRunning = false;
let animationFrameId = null;
let pendingBitmap = null;

// Handle image upload
imageUpload.addEventListener('change', (e) => {
    const file = e.target.files[0];
    if (!file) return;

    fileName.textContent = file.name;

    const img = new Image();
    img.onload = () => {
        // Set canvas sizes
        originalCanvas.width = img.width;
        originalCanvas.height = img.height;
        approxCanvas.width = img.width;
        approxCanvas.height = img.height;

        // Draw original image
        originalCtx.drawImage(img, 0, 0);

        // Extract pixel data
        imageData = originalCtx.getImageData(0, 0, img.width, img.height);

        // Clear approximation canvas
        approxCtx.fillStyle = '#000';
        approxCtx.fillRect(0, 0, img.width, img.height);

        // Enable start button
        startBtn.disabled = false;
        statusEl.textContent = 'Ready - Click Start';

        // Reset stats
        iterationEl.textContent = 'Iteration: 0';
        accuracyEl.textContent = 'Accuracy: 0.00%';
        progressFill.style.width = '0%';
    };

    img.src = URL.createObjectURL(file);
});

// Render pending bitmap on animation frame
function renderLoop() {
    if (pendingBitmap) {
        approxCtx.drawImage(pendingBitmap, 0, 0);
        pendingBitmap.close(); // Free the bitmap
        pendingBitmap = null;
    }
    if (isRunning) {
        animationFrameId = requestAnimationFrame(renderLoop);
    }
}

// Start algorithm
startBtn.addEventListener('click', () => {
    if (!imageData) return;

    isRunning = true;
    startBtn.disabled = true;
    stopBtn.disabled = false;
    statusEl.textContent = 'Running...';

    // Start render loop
    animationFrameId = requestAnimationFrame(renderLoop);

    // Create worker
    worker = new Worker('worker.js', { type: 'module' });

    worker.onmessage = (e) => {
        const { type, data } = e.data;

        switch (type) {
            case 'ready':
                // Worker is ready, send image data
                worker.postMessage({
                    type: 'start',
                    data: {
                        pixels: Array.from(imageData.data),
                        width: imageData.width,
                        height: imageData.height,
                        maxIterations: parseInt(maxIterationsInput.value),
                        targetAccuracy: parseFloat(targetAccuracyInput.value),
                        batchSize: parseInt(batchSizeInput.value),
                        algorithm: parseInt(algorithmSelect.value)
                    }
                });
                break;

            case 'progress':
                updateProgress(data);
                break;

            case 'finished':
                updateProgress(data);
                statusEl.textContent = 'Finished!';
                stopAlgorithm();
                break;

            case 'error':
                console.error('Worker error:', data);
                statusEl.textContent = 'Error: ' + data;
                stopAlgorithm();
                break;
        }
    };

    worker.onerror = (e) => {
        console.error('Worker error:', e);
        statusEl.textContent = 'Worker error';
        stopAlgorithm();
    };
});

// Stop algorithm
stopBtn.addEventListener('click', () => {
    if (worker) {
        worker.postMessage({ type: 'stop' });
    }
    stopAlgorithm();
});

function stopAlgorithm() {
    isRunning = false;
    startBtn.disabled = !imageData;
    stopBtn.disabled = true;

    if (animationFrameId) {
        cancelAnimationFrame(animationFrameId);
        animationFrameId = null;
    }

    // Render any final pending bitmap
    if (pendingBitmap) {
        approxCtx.drawImage(pendingBitmap, 0, 0);
        pendingBitmap.close();
        pendingBitmap = null;
    }

    if (worker) {
        worker.terminate();
        worker = null;
    }
}

function updateProgress(data) {
    const { iteration, accuracy, bitmap, maxIterations, targetAccuracy } = data;

    // Update stats
    iterationEl.textContent = `Iteration: ${iteration}`;
    accuracyEl.textContent = `Accuracy: ${(accuracy * 100).toFixed(2)}%`;

    // Update progress bar
    const iterProgress = iteration / maxIterations;
    const accProgress = accuracy / targetAccuracy;
    const progress = Math.min(Math.max(iterProgress, accProgress) * 100, 100);
    progressFill.style.width = `${progress}%`;

    // Store bitmap for rendering on next animation frame
    if (bitmap) {
        if (pendingBitmap) {
            pendingBitmap.close(); // Free old bitmap if not yet rendered
        }
        pendingBitmap = bitmap;
    }
}
