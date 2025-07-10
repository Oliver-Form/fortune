class MapEditor {
    constructor() {
        this.canvas = document.getElementById('mapCanvas');
        this.ctx = this.canvas.getContext('2d');
        this.minimap = document.getElementById('minimap');
        this.minimapCtx = this.minimap.getContext('2d');
        
        // Map data
        this.mapWidth = 4096;
        this.mapHeight = 4096;
        this.mapData = new Uint16Array(this.mapWidth * this.mapHeight);
        this.fillMap(0); // Fill with grass initially
        
        // View state
        this.viewX = 0;
        this.viewY = 0;
        this.zoom = 1;
        this.tileSize = 1; // Size of each tile in pixels at 100% zoom
        
        // Tool state
        this.currentTool = 'paint';
        this.selectedTile = 0;
        this.brushSize = 1;
        this.isDrawing = false;
        this.lastDrawPos = null;
        
        // Tile colors
        this.tileColors = {
            0: '#4CAF50', // Grass - Green
            1: '#DEB887', // Desert - Burlywood
            2: '#4682B4', // Water - Steel Blue
            3: '#696969', // Rock - Dim Gray
            4: '#F4A460'  // Sand - Sandy Brown
        };
        
        this.tileNames = {
            0: 'Grass',
            1: 'Desert', 
            2: 'Water',
            3: 'Rock',
            4: 'Sand'
        };
        
        this.initializeEventListeners();
        this.resizeCanvas();
        this.render();
        this.updateMinimap();
    }
    
    initializeEventListeners() {
        // Canvas events
        this.canvas.addEventListener('mousedown', this.onMouseDown.bind(this));
        this.canvas.addEventListener('mousemove', this.onMouseMove.bind(this));
        this.canvas.addEventListener('mouseup', this.onMouseUp.bind(this));
        this.canvas.addEventListener('wheel', this.onWheel.bind(this));
        this.canvas.addEventListener('contextmenu', e => e.preventDefault());
        
        // Tool buttons
        document.querySelectorAll('.tool-btn').forEach(btn => {
            btn.addEventListener('click', (e) => {
                document.querySelectorAll('.tool-btn').forEach(b => b.classList.remove('active'));
                btn.classList.add('active');
                this.currentTool = btn.dataset.tool;
            });
        });
        
        // Tile buttons
        document.querySelectorAll('.tile-btn').forEach(btn => {
            btn.addEventListener('click', (e) => {
                document.querySelectorAll('.tile-btn').forEach(b => b.classList.remove('active'));
                btn.classList.add('active');
                this.selectedTile = parseInt(btn.dataset.tile);
            });
        });
        
        // Controls
        document.getElementById('brushSize').addEventListener('input', (e) => {
            this.brushSize = parseInt(e.target.value);
            document.getElementById('brushSizeValue').textContent = this.brushSize;
        });
        
        document.getElementById('zoom').addEventListener('input', (e) => {
            this.zoom = parseFloat(e.target.value);
            document.getElementById('zoomValue').textContent = Math.round(this.zoom * 100) + '%';
            this.render();
            this.updateMinimap();
        });
        
        // File operations
        document.getElementById('loadMap').addEventListener('change', this.loadMapFile.bind(this));
        document.getElementById('saveMap').addEventListener('click', this.saveMap.bind(this));
        document.getElementById('newMap').addEventListener('click', this.newMap.bind(this));
        document.getElementById('exportPng').addEventListener('click', this.exportPng.bind(this));
        
        // Window resize
        window.addEventListener('resize', this.resizeCanvas.bind(this));
        
        // Keyboard shortcuts
        document.addEventListener('keydown', (e) => {
            switch(e.key) {
                case '1': this.selectTile(0); break;
                case '2': this.selectTile(1); break;
                case '3': this.selectTile(2); break;
                case '4': this.selectTile(3); break;
                case '5': this.selectTile(4); break;
                case 'b': this.selectTool('paint'); break;
                case 'f': this.selectTool('fill'); break;
                case 'c': this.selectTool('circle'); break;
                case 'l': this.selectTool('line'); break;
                case 'e': this.selectTool('eyedropper'); break;
            }
        });
    }
    
    selectTile(tileType) {
        this.selectedTile = tileType;
        document.querySelectorAll('.tile-btn').forEach(b => b.classList.remove('active'));
        document.querySelector(`[data-tile="${tileType}"]`).classList.add('active');
    }
    
    selectTool(tool) {
        this.currentTool = tool;
        document.querySelectorAll('.tool-btn').forEach(b => b.classList.remove('active'));
        document.querySelector(`[data-tool="${tool}"]`).classList.add('active');
    }
    
    resizeCanvas() {
        const container = this.canvas.parentElement;
        this.canvas.width = container.clientWidth;
        this.canvas.height = container.clientHeight;
        this.render();
    }
    
    fillMap(tileType) {
        this.mapData.fill(tileType);
    }
    
    getTile(x, y) {
        if (x < 0 || x >= this.mapWidth || y < 0 || y >= this.mapHeight) {
            return 0;
        }
        return this.mapData[y * this.mapWidth + x];
    }
    
    setTile(x, y, tileType) {
        if (x < 0 || x >= this.mapWidth || y < 0 || y >= this.mapHeight) {
            return;
        }
        this.mapData[y * this.mapWidth + x] = tileType;
    }
    
    screenToWorld(screenX, screenY) {
        const rect = this.canvas.getBoundingClientRect();
        const canvasX = screenX - rect.left;
        const canvasY = screenY - rect.top;
        
        const worldX = Math.floor((canvasX / this.zoom) + this.viewX);
        const worldY = Math.floor((canvasY / this.zoom) + this.viewY);
        
        return { x: worldX, y: worldY };
    }
    
    onMouseDown(e) {
        const pos = this.screenToWorld(e.clientX, e.clientY);
        this.isDrawing = true;
        this.lastDrawPos = pos;
        
        if (this.currentTool === 'eyedropper') {
            const tileType = this.getTile(pos.x, pos.y);
            this.selectTile(tileType);
            return;
        }
        
        this.useTool(pos, e);
    }
    
    onMouseMove(e) {
        const pos = this.screenToWorld(e.clientX, e.clientY);
        this.updateMouseInfo(pos);
        
        if (this.isDrawing && this.currentTool === 'paint') {
            this.paintBrush(pos);
        }
    }
    
    onMouseUp(e) {
        this.isDrawing = false;
        this.lastDrawPos = null;
        this.render();
        this.updateMinimap();
    }
    
    onWheel(e) {
        e.preventDefault();
        const rect = this.canvas.getBoundingClientRect();
        const mouseX = e.clientX - rect.left;
        const mouseY = e.clientY - rect.top;
        
        const oldZoom = this.zoom;
        const zoomFactor = e.deltaY > 0 ? 0.9 : 1.1;
        this.zoom = Math.max(0.1, Math.min(5, this.zoom * zoomFactor));
        
        // Zoom towards mouse position
        const zoomRatio = this.zoom / oldZoom;
        this.viewX += (mouseX / oldZoom) * (1 - zoomRatio);
        this.viewY += (mouseY / oldZoom) * (1 - zoomRatio);
        
        document.getElementById('zoom').value = this.zoom;
        document.getElementById('zoomValue').textContent = Math.round(this.zoom * 100) + '%';
        
        this.render();
        this.updateMinimap();
    }
    
    useTool(pos, e) {
        switch (this.currentTool) {
            case 'paint':
                this.paintBrush(pos);
                break;
            case 'fill':
                this.floodFill(pos.x, pos.y, this.selectedTile);
                break;
            case 'circle':
                // TODO: Implement circle tool
                break;
            case 'line':
                // TODO: Implement line tool
                break;
        }
        this.render();
        this.updateMinimap();
    }
    
    paintBrush(pos) {
        const radius = Math.floor(this.brushSize / 2);
        for (let dy = -radius; dy <= radius; dy++) {
            for (let dx = -radius; dx <= radius; dx++) {
                if (dx * dx + dy * dy <= radius * radius) {
                    this.setTile(pos.x + dx, pos.y + dy, this.selectedTile);
                }
            }
        }
    }
    
    floodFill(startX, startY, newTile) {
        const originalTile = this.getTile(startX, startY);
        if (originalTile === newTile) return;
        
        const stack = [{x: startX, y: startY}];
        const visited = new Set();
        
        while (stack.length > 0) {
            const {x, y} = stack.pop();
            const key = `${x},${y}`;
            
            if (visited.has(key)) continue;
            if (this.getTile(x, y) !== originalTile) continue;
            
            visited.add(key);
            this.setTile(x, y, newTile);
            
            // Add neighbors
            stack.push({x: x + 1, y: y});
            stack.push({x: x - 1, y: y});
            stack.push({x: x, y: y + 1});
            stack.push({x: x, y: y - 1});
            
            // Prevent infinite loops on large areas
            if (visited.size > 100000) break;
        }
    }
    
    updateMouseInfo(pos) {
        const tileType = this.getTile(pos.x, pos.y);
        const tileName = this.tileNames[tileType] || 'Unknown';
        
        document.getElementById('mapInfo').innerHTML = `
            <p>Size: ${this.mapWidth}x${this.mapHeight}</p>
            <p>Mouse: (${pos.x}, ${pos.y})</p>
            <p>Tile: ${tileName}</p>
            <p>Zoom: ${Math.round(this.zoom * 100)}%</p>
        `;
    }
    
    render() {
        this.ctx.fillStyle = '#1a202c';
        this.ctx.fillRect(0, 0, this.canvas.width, this.canvas.height);
        
        const tileSize = this.zoom;
        const startX = Math.max(0, Math.floor(this.viewX));
        const startY = Math.max(0, Math.floor(this.viewY));
        const endX = Math.min(this.mapWidth, startX + Math.ceil(this.canvas.width / tileSize) + 1);
        const endY = Math.min(this.mapHeight, startY + Math.ceil(this.canvas.height / tileSize) + 1);
        
        for (let y = startY; y < endY; y++) {
            for (let x = startX; x < endX; x++) {
                const tileType = this.getTile(x, y);
                const color = this.tileColors[tileType] || '#000000';
                
                this.ctx.fillStyle = color;
                this.ctx.fillRect(
                    (x - this.viewX) * tileSize,
                    (y - this.viewY) * tileSize,
                    tileSize + 1,
                    tileSize + 1
                );
            }
        }
        
        // Draw grid when zoomed in
        if (this.zoom >= 4) {
            this.ctx.strokeStyle = 'rgba(255, 255, 255, 0.1)';
            this.ctx.lineWidth = 1;
            
            for (let x = startX; x <= endX; x++) {
                const screenX = (x - this.viewX) * tileSize;
                this.ctx.beginPath();
                this.ctx.moveTo(screenX, 0);
                this.ctx.lineTo(screenX, this.canvas.height);
                this.ctx.stroke();
            }
            
            for (let y = startY; y <= endY; y++) {
                const screenY = (y - this.viewY) * tileSize;
                this.ctx.beginPath();
                this.ctx.moveTo(0, screenY);
                this.ctx.lineTo(this.canvas.width, screenY);
                this.ctx.stroke();
            }
        }
    }
    
    updateMinimap() {
        const scale = 200 / Math.max(this.mapWidth, this.mapHeight);
        this.minimapCtx.fillStyle = '#1a202c';
        this.minimapCtx.fillRect(0, 0, 200, 200);
        
        // Sample the map at lower resolution for performance
        const sampleRate = Math.max(1, Math.floor(this.mapWidth / 200));
        
        for (let y = 0; y < this.mapHeight; y += sampleRate) {
            for (let x = 0; x < this.mapWidth; x += sampleRate) {
                const tileType = this.getTile(x, y);
                const color = this.tileColors[tileType] || '#000000';
                
                this.minimapCtx.fillStyle = color;
                this.minimapCtx.fillRect(
                    x * scale,
                    y * scale,
                    sampleRate * scale + 1,
                    sampleRate * scale + 1
                );
            }
        }
        
        // Draw viewport indicator
        const viewportIndicator = document.getElementById('viewport-indicator');
        const viewWidth = this.canvas.width / this.zoom;
        const viewHeight = this.canvas.height / this.zoom;
        
        viewportIndicator.style.left = (this.viewX * scale) + 'px';
        viewportIndicator.style.top = (this.viewY * scale) + 'px';
        viewportIndicator.style.width = (viewWidth * scale) + 'px';
        viewportIndicator.style.height = (viewHeight * scale) + 'px';
    }
    
    loadMapFile(e) {
        const file = e.target.files[0];
        if (!file) return;
        
        const reader = new FileReader();
        reader.onload = (event) => {
            const buffer = event.target.result;
            const data = new Uint16Array(buffer);
            
            if (data.length === this.mapWidth * this.mapHeight) {
                this.mapData = data;
                this.render();
                this.updateMinimap();
                console.log('Map loaded successfully');
            } else {
                alert('Invalid map file format');
            }
        };
        reader.readAsArrayBuffer(file);
    }
    
    saveMap() {
        const buffer = this.mapData.buffer.slice();
        const blob = new Blob([buffer], { type: 'application/octet-stream' });
        const url = URL.createObjectURL(blob);
        
        const a = document.createElement('a');
        a.href = url;
        a.download = 'map.map';
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
        URL.revokeObjectURL(url);
    }
    
    newMap() {
        if (confirm('Create a new map? This will clear the current map.')) {
            this.fillMap(0);
            this.viewX = 0;
            this.viewY = 0;
            this.zoom = 1;
            document.getElementById('zoom').value = 1;
            document.getElementById('zoomValue').textContent = '100%';
            this.render();
            this.updateMinimap();
        }
    }
    
    exportPng() {
        // Create a temporary canvas for export
        const exportCanvas = document.createElement('canvas');
        const exportCtx = exportCanvas.getContext('2d');
        
        // Use a reasonable size for export (max 2048x2048)
        const maxSize = 2048;
        const scale = Math.min(maxSize / this.mapWidth, maxSize / this.mapHeight);
        
        exportCanvas.width = this.mapWidth * scale;
        exportCanvas.height = this.mapHeight * scale;
        
        // Render the entire map
        for (let y = 0; y < this.mapHeight; y++) {
            for (let x = 0; x < this.mapWidth; x++) {
                const tileType = this.getTile(x, y);
                const color = this.tileColors[tileType] || '#000000';
                
                exportCtx.fillStyle = color;
                exportCtx.fillRect(x * scale, y * scale, scale + 1, scale + 1);
            }
        }
        
        // Download the image
        exportCanvas.toBlob((blob) => {
            const url = URL.createObjectURL(blob);
            const a = document.createElement('a');
            a.href = url;
            a.download = 'map.png';
            document.body.appendChild(a);
            a.click();
            document.body.removeChild(a);
            URL.revokeObjectURL(url);
        });
    }
}

// Initialize the editor when the page loads
document.addEventListener('DOMContentLoaded', () => {
    window.mapEditor = new MapEditor();
});
