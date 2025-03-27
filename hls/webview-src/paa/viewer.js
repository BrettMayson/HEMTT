(function() {
  const vscode = acquireVsCodeApi();
  const paaData = window.paaData;
  
  let currentZoom = 1;
  let currentLevel = 0;
  let baseWidth = 0;
  let baseHeight = 0;
  let initialLoad = true;
  let isDragging = false;
  let lastMouseX = 0;
  let lastMouseY = 0;
  
  function initialize() {
    try {
      if (!paaData || !paaData.maps || !paaData.maps.length) {
        reportError('Invalid PAA data received');
        return;
      }
      
      populateMipmapSelector();
      updateFormatInfo();
      setupZoomControls();
      setupDragAndScrollControls();
      displayMipmap(0);

      document.getElementById('mipmapLevel').addEventListener('change', (event) => {
        displayMipmap(parseInt(event.target.value));
      });
      
      const container = document.getElementById('imageContainer');
      container.addEventListener('wheel', (event) => {
        // Handle Ctrl+scroll for zooming
        if (event.ctrlKey) {
          event.preventDefault();
          
          const zoomFactor = event.deltaY > 0 ? 0.9 : 1.1;
          
          // Just update the zoom and let browser handle positioning
          updateZoom(currentZoom * zoomFactor);
          return;
        }
        
        // Handle regular scrolling (let it work normally)
        // Shift+scroll for horizontal scrolling is handled by the browser automatically
      });
    } catch (error) {
      reportError(`Initialization error: ${error.message}`);
    }
  }
  
  function setupDragAndScrollControls() {
    const container = document.getElementById('imageContainer');
    
    // Enable dragging with mouse
    container.addEventListener('mousedown', (event) => {
      // Only handle left mouse button
      if (event.button !== 0) return;
      
      isDragging = true;
      lastMouseX = event.clientX;
      lastMouseY = event.clientY;
      container.style.cursor = 'grabbing';
      event.preventDefault();
    });
    
    document.addEventListener('mousemove', (event) => {
      if (!isDragging) return;
      
      const deltaX = event.clientX - lastMouseX;
      const deltaY = event.clientY - lastMouseY;
      
      container.scrollLeft -= deltaX;
      container.scrollTop -= deltaY;
      
      lastMouseX = event.clientX;
      lastMouseY = event.clientY;
    });
    
    document.addEventListener('mouseup', () => {
      if (isDragging) {
        isDragging = false;
        container.style.cursor = 'default';
      }
    });
    
    // Add keyboard navigation
    container.tabIndex = 0; // Make container focusable
    container.addEventListener('keydown', (event) => {
      const scrollStep = 50;
      switch (event.key) {
        case 'ArrowLeft':
          container.scrollLeft -= scrollStep;
          event.preventDefault();
          break;
        case 'ArrowRight':
          container.scrollLeft += scrollStep;
          event.preventDefault();
          break;
        case 'ArrowUp':
          container.scrollTop -= scrollStep;
          event.preventDefault();
          break;
        case 'ArrowDown':
          container.scrollTop += scrollStep;
          event.preventDefault();
          break;
      }
    });
    
    // Add mouse leave detection to prevent stuck dragging
    document.addEventListener('mouseleave', () => {
      if (isDragging) {
        isDragging = false;
        container.style.cursor = 'default';
      }
    });
  }
  
  function populateMipmapSelector() {
    const selector = document.getElementById('mipmapLevel');
    
    selector.innerHTML = '';
    
    paaData.maps.forEach((_, index) => {
      const option = document.createElement('option');
      option.value = index;
      option.textContent = `Level ${index}`;
      selector.appendChild(option);
    });
  }
  
  function updateFormatInfo() {
    document.getElementById('formatInfo').textContent = paaData.format || 'Unknown';
  }
  
  function setupZoomControls() {
    const controls = document.getElementById('controls');
    const zoomControls = document.createElement('div');
    zoomControls.innerHTML = `
      <button id="zoomOut">-</button>
      <span id="zoomLevel">100%</span>
      <button id="zoomIn">+</button>
      <button id="zoomReset">Reset</button>
      <button id="zoomFit">Fit</button>
    `;
    controls.appendChild(zoomControls);
    
    document.getElementById('zoomIn').addEventListener('click', () => setZoom(currentZoom * 1.2));
    document.getElementById('zoomOut').addEventListener('click', () => setZoom(currentZoom * 0.8));
    document.getElementById('zoomReset').addEventListener('click', () => setZoom(1));
    document.getElementById('zoomFit').addEventListener('click', () => setZoom(calculateFitZoom()));
  }
  
  function calculateFitZoom() {
    const container = document.getElementById('imageContainer');
    const containerWidth = container.clientWidth;
    const containerHeight = container.clientHeight;
    
    // Calculate zoom factors for width and height
    const widthZoom = containerWidth / baseWidth;
    const heightZoom = containerHeight / baseHeight;
    
    // Return the smaller of the two to ensure the entire image fits
    return Math.min(widthZoom, heightZoom) * 0.95; // 5% margin
  }

  // Simple zoom update without repositioning
  function updateZoom(zoom) {
    zoom = Math.max(0.1, Math.min(100, zoom));
    currentZoom = zoom;
    
    document.getElementById('zoomLevel').textContent = `${Math.round(zoom * 100)}%`;
    
    const img = document.getElementById('paaImage');
    
    const levelFactor = Math.pow(2, currentLevel);
    const width = (baseWidth / levelFactor) * zoom;
    const height = (baseHeight / levelFactor) * zoom;
    
    img.style.width = `${width}px`;
    img.style.height = `${height}px`;
  }

  // Only use for button zooming and mipmap changes where we want to reposition
  function setZoom(zoom) {
    zoom = Math.max(0.1, Math.min(100, zoom));
    currentZoom = zoom;
    
    document.getElementById('zoomLevel').textContent = `${Math.round(zoom * 100)}%`;
    
    const img = document.getElementById('paaImage');
    const container = document.getElementById('imageContainer');
    
    // Remember scroll position relative to image center
    const containerWidth = container.clientWidth;
    const containerHeight = container.clientHeight;
    const scrollXRatio = (container.scrollLeft + containerWidth / 2) / img.width;
    const scrollYRatio = (container.scrollTop + containerHeight / 2) / img.height;
    
    const levelFactor = Math.pow(2, currentLevel);
    const width = (baseWidth / levelFactor) * zoom;
    const height = (baseHeight / levelFactor) * zoom;
    
    img.style.width = `${width}px`;
    img.style.height = `${height}px`;
    
    // Restore scroll position to maintain the same view center
    setTimeout(() => {
      container.scrollLeft = scrollXRatio * width - containerWidth / 2;
      container.scrollTop = scrollYRatio * height - containerHeight / 2;
    }, 0);
  }
  
  function zoomAtPoint(zoom, mouseX, mouseY) {
    zoom = Math.max(0.1, Math.min(100, zoom));
    
    const img = document.getElementById('paaImage');
    const container = document.getElementById('imageContainer');
    
    // Calculate position ratios before zooming
    const beforeX = mouseX / img.width;
    const beforeY = mouseY / img.height;
    
    // Get the original image dimensions
    const beforeWidth = img.width;
    const beforeHeight = img.height;
    
    // Update zoom level display
    document.getElementById('zoomLevel').textContent = `${Math.round(zoom * 100)}%`;
    currentZoom = zoom;
    
    // Calculate new dimensions
    const levelFactor = Math.pow(2, currentLevel);
    const width = (baseWidth / levelFactor) * zoom;
    const height = (baseHeight / levelFactor) * zoom;
    
    // Apply new dimensions
    img.style.width = `${width}px`;
    img.style.height = `${height}px`;
    
    // Calculate how much the image dimensions changed
    const widthDiff = width - beforeWidth;
    const heightDiff = height - beforeHeight;
    
    // Adjust scroll position to keep mouse point fixed
    container.scrollLeft += (widthDiff * beforeX);
    container.scrollTop += (heightDiff * beforeY);
  }
  
  function displayMipmap(level) {
    if (level < 0 || level >= paaData.maps.length) {
      reportError(`Invalid mipmap level: ${level}`);
      return;
    }

    const newZoom = currentZoom * Math.pow(2, level - currentLevel);
    currentLevel = level;
    
    const img = document.getElementById('paaImage');
    img.src = `data:image/png;base64,${paaData.maps[level]}`;
    
    img.onload = () => {
      document.getElementById('dimensionsInfo').textContent = `${img.naturalWidth}×${img.naturalHeight}`;
      
      if (baseWidth === 0 && baseHeight === 0) {
        if (level === 0) {
          baseWidth = img.naturalWidth;
          baseHeight = img.naturalHeight;
        } else {
          baseWidth = img.naturalWidth * Math.pow(2, level);
          baseHeight = img.naturalHeight * Math.pow(2, level);
        }
      }

      if (initialLoad) {
        initialLoad = false;
        setZoom(calculateFitZoom());
      } else {
        setZoom(newZoom);
      }
    };
  }
  
  function reportError(message) {
    vscode.postMessage({
      command: 'error',
      text: message
    });
    console.error(message);
  }
  
  document.addEventListener('DOMContentLoaded', initialize);
})();
