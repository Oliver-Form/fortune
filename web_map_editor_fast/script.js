// High-performance map editor with multiple optimization strategies
class FastMapEditor {
    constructor() {
        this.canvas = document.getElementById('mapCanvas');
        this.ctx = this.canvas.getContext('2d');
        this.minimap = document.getElementById('minimapCanvas');
        this.minimapCtx = this.minimap.getContext('2d');
        
        // Disable image smoothing for crisp pixels
        this.ctx.imageSmoothingEnabled = false;
        this.minimapCtx.imageSmoothingEnabled = false;
        
        // Map data
        this.mapWidth = 4096;
        this.mapHeight = 4096;
        this.tiles = new Uint16Array(this.mapWidth * this.mapHeight);
        
        // Performance settings
        this.performanceMode = 'balanced';
        this.tileSize = 2; // Smaller tiles for better performance
        this.chunkSize = 256; // Render in chunks
        this.maxVisibleChunks = 64; // Limit visible chunks
        
        // Rendering optimization
        this.imageDataCache = new Map();
        this.dirtyChunks = new Set();
        this.visibleChunks = new Set();
        this.needsRedraw = true;
        this.isRendering = false;
        
        // FPS tracking
        this.lastFrameTime = 0;
        this.frameCount = 0;
        this.fps = 0;
        
        // View state
        this.zoom = 0.25; // Start more zoomed out
        this.offsetX = -this.mapWidth * this.tileSize * this.zoom / 4; // Center view
        this.offsetY = -this.mapHeight * this.tileSize * this.zoom / 4;
        this.isDragging = false;
        this.lastMouseX = 0;
        this.lastMouseY = 0;
        
        // Tool state
        this.selectedTile = 0;
        this.selectedTool = 'paint';
        this.brushSize = 1;
        this.showChunkGrid = false;
        
        // Color palette for tiles
        this.tileColors = [
            '#4a7c3a', // Grass - green
            '#d4a853', // Desert - sandy yellow
            '#4a90d4', // Water - blue
            '#6b5b5b', // Rock - gray
            '#f4e4a6'  // Sand - light yellow
        ];
        
        this.tileNames = ['Grass', 'Desert', 'Water', 'Rock', 'Sand'];
        
        this.initializeCanvas();
        this.setupEventListeners();
        this.startRenderLoop();
        this.fillMapWithDefault();
    }
    
    initializeCanvas() {
        this.resizeCanvas();
        window.addEventListener('resize', () => this.resizeCanvas());
    }
    
    resizeCanvas() {
        const container = this.canvas.parentElement;
        this.canvas.width = container.clientWidth;
        this.canvas.height = container.clientHeight;
        this.ctx.imageSmoothingEnabled = false;
        this.needsRedraw = true;
    }
    
    setupEventListeners() {
        // File operations
        document.getElementById('loadBtn').addEventListener('click', () => {
            document.getElementById('fileInput').click();
        });
        
        document.getElementById('fileInput').addEventListener('change', (e) => {
            if (e.target.files.length > 0) {
                this.loadMapFile(e.target.files[0]);
            }
        });
        
        document.getElementById('saveBtn').addEventListener('click', () => this.saveMap());
        document.getElementById('exportBtn').addEventListener('click', () => this.exportPNG());
        document.getElementById('newBtn').addEventListener('click', () => this.newMap());
        
        // Tools
        document.querySelectorAll('.tool-btn').forEach(btn => {
            btn.addEventListener('click', () => {
                document.querySelectorAll('.tool-btn').forEach(b => b.classList.remove('active'));
                btn.classList.add('active');
                this.selectedTool = btn.dataset.tool;
                this.updateCursor();
            });
        });
        
        // Tiles
        document.querySelectorAll('.tile-btn').forEach(btn => {
            btn.addEventListener('click', () => {
                document.querySelectorAll('.tile-btn').forEach(b => b.classList.remove('active'));
                btn.classList.add('active');
                this.selectedTile = parseInt(btn.dataset.tile);
            });
        });
        
        // Brush size
        const brushSizeSlider = document.getElementById('brushSize');
        brushSizeSlider.addEventListener('input', (e) => {
            this.brushSize = parseInt(e.target.value);
            document.getElementById('brushSizeValue').textContent = this.brushSize;
        });
        
        // Zoom
        const zoomSlider = document.getElementById('zoomSlider');
        zoomSlider.addEventListener('input', (e) => {
            this.setZoom(parseFloat(e.target.value));
        });
        
        // View controls
        document.getElementById('resetViewBtn').addEventListener('click', () => this.resetView());
        document.getElementById('centerBtn').addEventListener('click', () => this.centerView());
        
        // Chunk grid toggle
        document.getElementById('chunkGridToggle').addEventListener('change', (e) => {
            this.showChunkGrid = e.target.checked;
            this.needsRedraw = true;
        });
        
        // Performance mode
        document.getElementById('performanceMode').addEventListener('change', (e) => {
            this.setPerformanceMode(e.target.value);
        });
        
        // Canvas events
        this.canvas.addEventListener('mousedown', (e) => this.onMouseDown(e));
        this.canvas.addEventListener('mousemove', (e) => this.onMouseMove(e));
        this.canvas.addEventListener('mouseup', (e) => this.onMouseUp(e));
        this.canvas.addEventListener('wheel', (e) => this.onWheel(e));
        this.canvas.addEventListener('contextmenu', (e) => e.preventDefault());
        
        // Keyboard shortcuts
        document.addEventListener('keydown', (e) => this.onKeyDown(e));
    }
    
    setPerformanceMode(mode) {
        this.performanceMode = mode;
        
        switch (mode) {
            case 'fast':
                this.tileSize = 1;
                this.maxVisibleChunks = 32;
                break;
            case 'balanced':
                this.tileSize = 2;
                this.maxVisibleChunks = 64;
                break;
            case 'quality':
                this.tileSize = 4;
                this.maxVisibleChunks = 128;
                break;
        }
        
        this.clearCache();
        this.needsRedraw = true;
    }
    
    clearCache() {
        this.imageDataCache.clear();
        this.dirtyChunks.clear();
        for (let i = 0; i < Math.ceil(this.mapWidth / this.chunkSize); i++) {
            for (let j = 0; j < Math.ceil(this.mapHeight / this.chunkSize); j++) {
                this.dirtyChunks.add(`${i},${j}`);
            }
        }
    }
    
    fillMapWithDefault() {
        // Fill with grass by default
        this.tiles.fill(0);
        this.clearCache();
        this.needsRedraw = true;
    }
    
    getChunkKey(chunkX, chunkY) {
        return `${chunkX},${chunkY}`;
    }
    
    getVisibleChunks() {
        const chunks = new Set();
        
        // Calculate visible area in tile coordinates
        const leftTile = Math.floor(-this.offsetX / (this.tileSize * this.zoom));
        const topTile = Math.floor(-this.offsetY / (this.tileSize * this.zoom));
        const rightTile = Math.ceil((this.canvas.width - this.offsetX) / (this.tileSize * this.zoom));
        const bottomTile = Math.ceil((this.canvas.height - this.offsetY) / (this.tileSize * this.zoom));
        
        // Convert to chunk coordinates
        const leftChunk = Math.max(0, Math.floor(leftTile / this.chunkSize));
        const topChunk = Math.max(0, Math.floor(topTile / this.chunkSize));
        const rightChunk = Math.min(Math.ceil(this.mapWidth / this.chunkSize) - 1, Math.floor(rightTile / this.chunkSize));
        const bottomChunk = Math.min(Math.ceil(this.mapHeight / this.chunkSize) - 1, Math.floor(bottomTile / this.chunkSize));
        
        for (let x = leftChunk; x <= rightChunk; x++) {
            for (let y = topChunk; y <= bottomChunk; y++) {
                chunks.add(this.getChunkKey(x, y));
            }
        }
        
        return chunks;
    }
    
    renderChunk(chunkX, chunkY) {
        const chunkKey = this.getChunkKey(chunkX, chunkY);
        
        if (!this.dirtyChunks.has(chunkKey) && this.imageDataCache.has(chunkKey)) {
            return this.imageDataCache.get(chunkKey);
        }
        
        const chunkCanvas = document.createElement('canvas');
        chunkCanvas.width = this.chunkSize * this.tileSize;
        chunkCanvas.height = this.chunkSize * this.tileSize;
        const chunkCtx = chunkCanvas.getContext('2d');
        chunkCtx.imageSmoothingEnabled = false;
        
        // Render tiles in this chunk
        for (let x = 0; x < this.chunkSize; x++) {
            for (let y = 0; y < this.chunkSize; y++) {
                const tileX = chunkX * this.chunkSize + x;
                const tileY = chunkY * this.chunkSize + y;
                
                if (tileX >= this.mapWidth || tileY >= this.mapHeight) continue;
                
                const tileIndex = tileY * this.mapWidth + tileX;
                const tileType = this.tiles[tileIndex];
                
                chunkCtx.fillStyle = this.tileColors[tileType] || this.tileColors[0];
                chunkCtx.fillRect(
                    x * this.tileSize,
                    y * this.tileSize,
                    this.tileSize,
                    this.tileSize
                );
            }
        }
        
        this.imageDataCache.set(chunkKey, chunkCanvas);
        this.dirtyChunks.delete(chunkKey);
        
        return chunkCanvas;
    }
    
    render() {
        if (!this.needsRedraw || this.isRendering) return;
        
        this.isRendering = true;
        this.needsRedraw = false;
        
        // Clear canvas
        this.ctx.fillStyle = '#1a1a1a';
        this.ctx.fillRect(0, 0, this.canvas.width, this.canvas.height);
        
        // Get visible chunks
        const visibleChunks = this.getVisibleChunks();
        this.visibleChunks = visibleChunks;
        
        // Limit number of chunks for performance
        let renderedCount = 0;
        for (const chunkKey of visibleChunks) {
            if (renderedCount >= this.maxVisibleChunks) break;
            
            const [chunkX, chunkY] = chunkKey.split(',').map(Number);
            const chunkCanvas = this.renderChunk(chunkX, chunkY);
            
            const screenX = this.offsetX + chunkX * this.chunkSize * this.tileSize * this.zoom;
            const screenY = this.offsetY + chunkY * this.chunkSize * this.tileSize * this.zoom;
            
            this.ctx.drawImage(
                chunkCanvas,
                screenX,
                screenY,
                chunkCanvas.width * this.zoom,
                chunkCanvas.height * this.zoom
            );
            
            renderedCount++;
        }
        
        // Draw chunk grid if enabled
        if (this.showChunkGrid) {
            this.drawChunkGrid();
        }
        
        this.renderMinimap();
        this.updateUI();
        
        this.isRendering = false;
    }
    
    drawChunkGrid() {
        this.ctx.strokeStyle = 'rgba(255, 255, 255, 0.3)';
        this.ctx.lineWidth = 1;
        
        const chunkPixelSize = this.chunkSize * this.tileSize * this.zoom;
        
        // Vertical lines
        for (let x = 0; x <= Math.ceil(this.mapWidth / this.chunkSize); x++) {
            const screenX = this.offsetX + x * chunkPixelSize;
            if (screenX >= 0 && screenX <= this.canvas.width) {
                this.ctx.beginPath();
                this.ctx.moveTo(screenX, 0);
                this.ctx.lineTo(screenX, this.canvas.height);
                this.ctx.stroke();
            }
        }
        
        // Horizontal lines
        for (let y = 0; y <= Math.ceil(this.mapHeight / this.chunkSize); y++) {
            const screenY = this.offsetY + y * chunkPixelSize;
            if (screenY >= 0 && screenY <= this.canvas.height) {
                this.ctx.beginPath();
                this.ctx.moveTo(0, screenY);
                this.ctx.lineTo(this.canvas.width, screenY);
                this.ctx.stroke();
            }
        }
    }
    
    renderMinimap() {
        const scale = 200 / Math.max(this.mapWidth, this.mapHeight);
        
        this.minimapCtx.fillStyle = '#2a2a2a';
        this.minimapCtx.fillRect(0, 0, 200, 200);
        
        // Sample the map at lower resolution for minimap
        const sampleRate = Math.max(1, Math.floor(this.mapWidth / 200));
        
        for (let x = 0; x < 200; x += 2) {
            for (let y = 0; y < 200; y += 2) {
                const mapX = Math.floor(x / scale);
                const mapY = Math.floor(y / scale);
                
                if (mapX < this.mapWidth && mapY < this.mapHeight) {
                    const tileIndex = mapY * this.mapWidth + mapX;
                    const tileType = this.tiles[tileIndex];
                    
                    this.minimapCtx.fillStyle = this.tileColors[tileType] || this.tileColors[0];
                    this.minimapCtx.fillRect(x, y, 2, 2);
                }
            }
        }
        
        // Draw viewport rectangle
        const viewLeft = -this.offsetX / (this.tileSize * this.zoom) * scale;
        const viewTop = -this.offsetY / (this.tileSize * this.zoom) * scale;
        const viewWidth = this.canvas.width / (this.tileSize * this.zoom) * scale;
        const viewHeight = this.canvas.height / (this.tileSize * this.zoom) * scale;
        
        this.minimapCtx.strokeStyle = '#007acc';
        this.minimapCtx.lineWidth = 2;
        this.minimapCtx.strokeRect(viewLeft, viewTop, viewWidth, viewHeight);
    }
    
    startRenderLoop() {
        const loop = (timestamp) => {
            // Calculate FPS
            if (timestamp - this.lastFrameTime >= 1000) {
                this.fps = this.frameCount;
                this.frameCount = 0;
                this.lastFrameTime = timestamp;
            }
            this.frameCount++;
            
            this.render();
            requestAnimationFrame(loop);
        };
        requestAnimationFrame(loop);
    }
    
    updateUI() {
        document.getElementById('zoomValue').textContent = Math.round(this.zoom * 100) + '%';
        document.getElementById('fps').textContent = this.fps;
        document.getElementById('chunkCount').textContent = 
            `${this.visibleChunks.size}/${Math.ceil(this.mapWidth / this.chunkSize) * Math.ceil(this.mapHeight / this.chunkSize)}`;
    }
    
    // Mouse and interaction methods
    getMouseTileCoords(e) {
        const rect = this.canvas.getBoundingClientRect();
        const mouseX = e.clientX - rect.left;
        const mouseY = e.clientY - rect.top;
        
        const tileX = Math.floor((mouseX - this.offsetX) / (this.tileSize * this.zoom));
        const tileY = Math.floor((mouseY - this.offsetY) / (this.tileSize * this.zoom));
        
        return { x: tileX, y: tileY, mouseX, mouseY };
    }
    
    onMouseDown(e) {
        const coords = this.getMouseTileCoords(e);
        
        if (e.button === 1 || (e.button === 0 && e.ctrlKey)) { // Middle mouse or Ctrl+click for panning
            this.isDragging = true;
            this.lastMouseX = coords.mouseX;
            this.lastMouseY = coords.mouseY;
            this.canvas.style.cursor = 'grabbing';
        } else if (e.button === 0) { // Left click for tools
            this.useTool(coords.x, coords.y);
        }
    }
    
    onMouseMove(e) {
        const coords = this.getMouseTileCoords(e);
        
        // Update mouse coordinates display
        document.getElementById('mouseCoords').textContent = `(${coords.x}, ${coords.y})`;
        
        if (coords.x >= 0 && coords.x < this.mapWidth && coords.y >= 0 && coords.y < this.mapHeight) {
            const tileIndex = coords.y * this.mapWidth + coords.x;
            const tileType = this.tiles[tileIndex];
            document.getElementById('currentTile').textContent = this.tileNames[tileType] || 'Unknown';
        }
        
        if (this.isDragging) {
            const deltaX = coords.mouseX - this.lastMouseX;
            const deltaY = coords.mouseY - this.lastMouseY;
            
            this.offsetX += deltaX;
            this.offsetY += deltaY;
            
            this.lastMouseX = coords.mouseX;
            this.lastMouseY = coords.mouseY;
            
            this.needsRedraw = true;
        } else if (e.buttons === 1 && !e.ctrlKey) { // Left mouse held for continuous painting
            this.useTool(coords.x, coords.y);
        }
    }
    
    onMouseUp(e) {
        this.isDragging = false;
        this.updateCursor();
    }
    
    onWheel(e) {
        e.preventDefault();
        
        const coords = this.getMouseTileCoords(e);
        const oldZoom = this.zoom;
        
        // Zoom in/out
        const zoomFactor = e.deltaY > 0 ? 0.9 : 1.1;
        this.setZoom(this.zoom * zoomFactor);
        
        // Zoom towards mouse position
        if (this.zoom !== oldZoom) {
            const zoomRatio = this.zoom / oldZoom;
            this.offsetX = coords.mouseX - (coords.mouseX - this.offsetX) * zoomRatio;
            this.offsetY = coords.mouseY - (coords.mouseY - this.offsetY) * zoomRatio;
            this.needsRedraw = true;
        }
    }
    
    setZoom(newZoom) {
        this.zoom = Math.max(0.1, Math.min(5.0, newZoom));
        document.getElementById('zoomSlider').value = this.zoom;
        this.needsRedraw = true;
    }
    
    updateCursor() {
        if (this.selectedTool === 'paint') {
            this.canvas.style.cursor = 'crosshair';
        } else if (this.selectedTool === 'fill') {
            this.canvas.style.cursor = 'crosshair';
        } else if (this.selectedTool === 'eyedropper') {
            this.canvas.style.cursor = 'crosshair';
        } else {
            this.canvas.style.cursor = 'default';
        }
    }
    
    useTool(tileX, tileY) {
        if (tileX < 0 || tileX >= this.mapWidth || tileY < 0 || tileY >= this.mapHeight) return;
        
        if (this.selectedTool === 'paint') {
            this.paintTiles(tileX, tileY);
        } else if (this.selectedTool === 'fill') {
            this.floodFill(tileX, tileY);
        } else if (this.selectedTool === 'eyedropper') {
            this.pickTile(tileX, tileY);
        }
    }
    
    paintTiles(centerX, centerY) {
        const radius = this.brushSize;
        const changed = new Set();
        
        for (let dx = -radius; dx <= radius; dx++) {
            for (let dy = -radius; dy <= radius; dy++) {
                if (dx * dx + dy * dy <= radius * radius) {
                    const x = centerX + dx;
                    const y = centerY + dy;
                    
                    if (x >= 0 && x < this.mapWidth && y >= 0 && y < this.mapHeight) {
                        const tileIndex = y * this.mapWidth + x;
                        if (this.tiles[tileIndex] !== this.selectedTile) {
                            this.tiles[tileIndex] = this.selectedTile;
                            
                            // Mark chunk as dirty
                            const chunkX = Math.floor(x / this.chunkSize);
                            const chunkY = Math.floor(y / this.chunkSize);
                            const chunkKey = this.getChunkKey(chunkX, chunkY);
                            this.dirtyChunks.add(chunkKey);
                            changed.add(chunkKey);
                        }
                    }
                }
            }
        }
        
        if (changed.size > 0) {
            this.needsRedraw = true;
        }
    }
    
    floodFill(startX, startY) {
        const startIndex = startY * this.mapWidth + startX;
        const targetTile = this.tiles[startIndex];
        
        if (targetTile === this.selectedTile) return;
        
        const stack = [{ x: startX, y: startY }];
        const visited = new Set();
        const changed = new Set();
        
        while (stack.length > 0) {
            const { x, y } = stack.pop();
            
            if (x < 0 || x >= this.mapWidth || y < 0 || y >= this.mapHeight) continue;
            
            const index = y * this.mapWidth + x;
            if (visited.has(index) || this.tiles[index] !== targetTile) continue;
            
            visited.add(index);
            this.tiles[index] = this.selectedTile;
            
            // Mark chunk as dirty
            const chunkX = Math.floor(x / this.chunkSize);
            const chunkY = Math.floor(y / this.chunkSize);
            const chunkKey = this.getChunkKey(chunkX, chunkY);
            this.dirtyChunks.add(chunkKey);
            changed.add(chunkKey);
            
            // Add neighbors
            stack.push({ x: x + 1, y });
            stack.push({ x: x - 1, y });
            stack.push({ x, y: y + 1 });
            stack.push({ x, y: y - 1 });
            
            // Prevent stack overflow on large areas
            if (visited.size > 100000) break;
        }
        
        if (changed.size > 0) {
            this.needsRedraw = true;
        }
    }
    
    pickTile(tileX, tileY) {
        const tileIndex = tileY * this.mapWidth + tileX;
        const tileType = this.tiles[tileIndex];
        
        this.selectedTile = tileType;
        
        // Update UI
        document.querySelectorAll('.tile-btn').forEach(btn => {
            btn.classList.remove('active');
            if (parseInt(btn.dataset.tile) === tileType) {
                btn.classList.add('active');
            }
        });
    }
    
    // File operations
    async loadMapFile(file) {
        document.getElementById('loadingIndicator').style.display = 'block';
        
        try {
            const arrayBuffer = await file.arrayBuffer();
            const dataView = new DataView(arrayBuffer);
            
            if (arrayBuffer.byteLength !== this.mapWidth * this.mapHeight * 2) {
                alert('Invalid map file size. Expected ' + (this.mapWidth * this.mapHeight * 2) + ' bytes.');
                return;
            }
            
            for (let i = 0; i < this.tiles.length; i++) {
                this.tiles[i] = dataView.getUint16(i * 2, true); // little-endian
            }
            
            this.clearCache();
            this.needsRedraw = true;
            
        } catch (error) {
            alert('Error loading map file: ' + error.message);
        } finally {
            document.getElementById('loadingIndicator').style.display = 'none';
        }
    }
    
    saveMap() {
        const buffer = new ArrayBuffer(this.tiles.length * 2);
        const dataView = new DataView(buffer);
        
        for (let i = 0; i < this.tiles.length; i++) {
            dataView.setUint16(i * 2, this.tiles[i], true); // little-endian
        }
        
        const blob = new Blob([buffer], { type: 'application/octet-stream' });
        const url = URL.createObjectURL(blob);
        
        const a = document.createElement('a');
        a.href = url;
        a.download = 'map.map';
        a.click();
        
        URL.revokeObjectURL(url);
    }
    
    async exportPNG() {
        // Create a high-resolution canvas for export
        const exportCanvas = document.createElement('canvas');
        exportCanvas.width = this.mapWidth;
        exportCanvas.height = this.mapHeight;
        const exportCtx = exportCanvas.getContext('2d');
        exportCtx.imageSmoothingEnabled = false;
        
        document.getElementById('loadingIndicator').style.display = 'block';
        document.getElementById('loadingIndicator').textContent = 'Exporting...';
        
        // Render full map in chunks to avoid blocking UI
        for (let chunkY = 0; chunkY < this.mapHeight; chunkY += 100) {
            for (let y = chunkY; y < Math.min(chunkY + 100, this.mapHeight); y++) {
                for (let x = 0; x < this.mapWidth; x++) {
                    const tileIndex = y * this.mapWidth + x;
                    const tileType = this.tiles[tileIndex];
                    
                    exportCtx.fillStyle = this.tileColors[tileType] || this.tileColors[0];
                    exportCtx.fillRect(x, y, 1, 1);
                }
            }
            
            // Update progress
            const progress = Math.round(chunkY / this.mapHeight * 100);
            document.getElementById('loadingIndicator').textContent = `Exporting... ${progress}%`;
            
            // Allow UI to update
            await new Promise(resolve => setTimeout(resolve, 1));
        }
        
        // Download the image
        exportCanvas.toBlob((blob) => {
            const url = URL.createObjectURL(blob);
            const a = document.createElement('a');
            a.href = url;
            a.download = 'map.png';
            a.click();
            URL.revokeObjectURL(url);
            
            document.getElementById('loadingIndicator').style.display = 'none';
        });
    }
    
    newMap() {
        if (confirm('Create a new map? This will clear all current changes.')) {
            this.fillMapWithDefault();
        }
    }
    
    resetView() {
        this.zoom = 0.25;
        this.offsetX = -this.mapWidth * this.tileSize * this.zoom / 4;
        this.offsetY = -this.mapHeight * this.tileSize * this.zoom / 4;
        this.needsRedraw = true;
    }
    
    centerView() {
        this.offsetX = (this.canvas.width - this.mapWidth * this.tileSize * this.zoom) / 2;
        this.offsetY = (this.canvas.height - this.mapHeight * this.tileSize * this.zoom) / 2;
        this.needsRedraw = true;
    }
    
    onKeyDown(e) {
        // Tile selection shortcuts
        if (e.key >= '1' && e.key <= '5') {
            const tileIndex = parseInt(e.key) - 1;
            if (tileIndex < this.tileColors.length) {
                this.selectedTile = tileIndex;
                document.querySelectorAll('.tile-btn').forEach((btn, i) => {
                    btn.classList.toggle('active', i === tileIndex);
                });
            }
        }
        
        // Tool shortcuts
        if (e.key.toLowerCase() === 'b') {
            this.selectedTool = 'paint';
            document.querySelectorAll('.tool-btn').forEach(btn => {
                btn.classList.toggle('active', btn.dataset.tool === 'paint');
            });
            this.updateCursor();
        } else if (e.key.toLowerCase() === 'f') {
            this.selectedTool = 'fill';
            document.querySelectorAll('.tool-btn').forEach(btn => {
                btn.classList.toggle('active', btn.dataset.tool === 'fill');
            });
            this.updateCursor();
        } else if (e.key.toLowerCase() === 'e') {
            this.selectedTool = 'eyedropper';
            document.querySelectorAll('.tool-btn').forEach(btn => {
                btn.classList.toggle('active', btn.dataset.tool === 'eyedropper');
            });
            this.updateCursor();
        }
    }
}

// Initialize the editor when the page loads
document.addEventListener('DOMContentLoaded', () => {
    new FastMapEditor();
});
