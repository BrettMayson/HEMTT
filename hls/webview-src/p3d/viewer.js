import * as THREE from 'three';
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js';
import { ViewportGizmo } from "three-viewport-gizmo";

// Global variables to store state
let scene, camera, renderer, controls, gizmo, lightHelper, directionalLight;
let lodArray = [];
let currentModel = null;
let lastPosition, lastRotation, lastScale;

const testPattern = new THREE.TextureLoader().load("data:image/png;base64,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");

const materialMap = new Map();
materialMap.set("Texture", new THREE.MeshStandardMaterial({ color: 0xcccccc }));
materialMap.set("Standard", new THREE.MeshStandardMaterial({ color: 0xcccccc }));
materialMap.set("Wireframe", new THREE.MeshBasicMaterial({ color: 0xcccccc, wireframe: true }));
materialMap.set("Flat", new THREE.MeshStandardMaterial({ color: 0xcccccc, flatShading: true }));
materialMap.set("Unlit", new THREE.MeshBasicMaterial({ color: 0xcccccc }));
materialMap.set("Depth", new THREE.MeshDepthMaterial());
materialMap.set("Normal", new THREE.MeshNormalMaterial());
materialMap.set("UV Test Pattern", new THREE.MeshStandardMaterial({ color: 0xcccccc, map: testPattern }));

const cullingNames = ["FrontSide", "BackSide", "DoubleSide"];

function setupRenderer() {

  renderer = new THREE.WebGLRenderer({ antialias: true });
  renderer.setPixelRatio(window.devicePixelRatio);
  renderer.setSize(window.innerWidth, window.innerHeight);
  renderer.setAnimationLoop(onRender);

  document.body.appendChild(renderer.domElement);
}

function setupScene() {

  scene = new THREE.Scene();
  scene.background = new THREE.Color(0x181a1b);
  scene.fog = new THREE.Fog(0x181a1b, 20, 100);

  camera = new THREE.PerspectiveCamera(75, window.innerWidth / window.innerHeight, 0.1, 1000);
  camera.position.set(4, 4, 4);
  camera.lookAt(0, 0, 0);

  controls = new OrbitControls(camera, renderer.domElement);

  gizmo = new ViewportGizmo(camera, renderer);
  gizmo.attachControls(controls);

  const hemiLight = new THREE.HemisphereLight(0xffffff, 0x8d8d8d, 5);
  scene.add(hemiLight);

  directionalLight = new THREE.DirectionalLight(0xffffff, 0.8);
  directionalLight.position.set(2, 5, 1);
  directionalLight.castShadow = true;
  scene.add(directionalLight);

  lightHelper = new THREE.DirectionalLightHelper(directionalLight, 1);
  scene.add(lightHelper);

  const gridHelper = new THREE.GridHelper(2000, 500, 0xffffff, 0x737373);
  gridHelper.material.opacity = 0.2;
  gridHelper.material.transparent = true;
  scene.add(gridHelper);

  const axesHelper = new THREE.AxesHelper(5);
  scene.add(axesHelper);

}

function onRender() {
  renderer.render(scene, camera);
  gizmo.render();
}

function loadSelectedLOD() {

  const lodLevel = document.getElementById('lodLevel');
  const index = parseInt(lodLevel.value);
  if (isNaN(index) || index < 0 || index >= lodArray.length) return;

  const materialSelect = document.getElementById('material');
  const materialKey = materialSelect.value;
  if (!materialMap.has(materialKey)) return;

  const cullingSelect = document.getElementById('culling');
  const culling = parseInt(cullingSelect.value);
  if (isNaN(culling) || culling < 0 || culling >= 3) return;

  if (currentModel) {
    lastPosition.copy(currentModel.position);
    lastRotation.copy(currentModel.rotation);
    lastScale.copy(currentModel.scale);

    scene.remove(currentModel);
    currentModel = null;
  }

  const selectedLOD = lodArray[index];
  updateStats(selectedLOD);

  const model = loadLODModel(selectedLOD, {
    material: materialKey,
    culling: culling,
  });

  model.position.copy(lastPosition);
  model.rotation.copy(lastRotation);
  model.scale.copy(lastScale);

  if (lastPosition.length() === 0) {
    const box = new THREE.Box3().setFromObject(model);
    const center = box.getCenter(new THREE.Vector3());
    model.position.sub(center);
  }

  scene.add(model);
  currentModel = model;
}

function updateStats(lod) {
  const statsDiv = document.getElementById('stats');
  if (!lod) {
    statsDiv.innerHTML = '';
    return;
  }

  statsDiv.innerHTML = `
    <p>Version: ${lod.version_major}.${lod.version_minor}</p>
    <p>Resolution: ${lod.resolution}</p>
    <p>Points: ${lod.points.length}</p>
    <p>Faces: ${lod.faces.length}</p>
    <p>Face Normals: ${lod.face_normals.length}</p>
  `;
}

function getLODTypeFromResolution(resolution) {
  const resolutionMap = {
    // View positions
    1000: "View Gunner",
    1100: "View Pilot",
    1200: "View Cargo",
    // Shadow volumes
    10000: "Shadow Volume 0",
    10010: "Shadow Volume 10",
    11000: "Shadow Buffer 0",
    11010: "Shadow Buffer 10",
    // Geometry types
    1e13: "Geometry",
    2e13: "Geometry Buoyancy",
    4e13: "Geometry PysX",
    // Memory and special geometries
    1e15: "Memory",
    2e15: "Land Contact",
    3e15: "Roadway",
    4e15: "Paths",
    5e15: "Hit-points",
    6e15: "View Geometry",
    7e15: "Fire Geometry",
    8e15: "View Cargo Geom.",
    9e15: "View Cargo Fire Geom.",
    // Commander, pilot, gunner views
    1e16: "View Commander",
    1.1e16: "View Commander Geom.",
    1.2e16: "View Commander Fire Geom.",
    1.3e16: "View Pilot Geom.",
    1.4e16: "View Pilot Fire Geom.",
    1.5e16: "View Gunner Geom.",
    1.6e16: "View Gunner Fire Geom.",
    // Additional types
    1.7e16: "Sub Parts",
    1.8e16: "Shadow Volume - View Cargo",
    1.9e16: "Shadow Volume - View Pilot",
    2e16: "Shadow Volume - View Gunner",
    2.1e16: "Wreck"
  };

  if (resolution >= 20000 && resolution < 30000) {
    return `Edit ${Math.floor(resolution - 20000)}`;
  }
  if (resolutionMap[resolution]) {
    return resolutionMap[resolution];
  }
  if (resolution > 1000) {
    return `Unknown ${resolution}`;
  }
  return `Resolution ${resolution}`;
}

function loadLODModel(json, options = {}) {
  const group = new THREE.Group();

  const points = json.points.map(point => new THREE.Vector3(
    point.coords[0],
    point.coords[1],
    point.coords[2]
  ));

  const faceGroups = {};

  json.faces.forEach(face => {
    const key = `${face.texture}|${face.material}`;
    if (!faceGroups[key]) {
      faceGroups[key] = {
        texture: face.texture,
        material: face.material,
        vertices: [],
        indices: [],
        uvs: [],
        normals: []
      };
    }

    const group = faceGroups[key];
    const indexOffset = group.vertices.length / 3;

    face.vertices.forEach(vertex => {
      const point = points[vertex.point_index];
      group.vertices.push(point.x, point.y, point.z);

      group.uvs.push(vertex.uv[0], 1 - vertex.uv[1]);

      if (vertex.normal_index < json.face_normals.length) {
        const normal = json.face_normals[vertex.normal_index];
        // Flip the normals to convert from DirectX to OpenGL format
        group.normals.push(normal[0], -normal[1], normal[2]);
      } else {
        group.normals.push(0, -1, 0);
      }
    });

    for (let i = 0; i < face.vertices.length - 2; i++) {
      group.indices.push(
        indexOffset,
        indexOffset + i + 2,
        indexOffset + i + 1,
      );
    }
  });

  Object.keys(faceGroups).forEach(key => {
    const faceGroup = faceGroups[key];
    const geometry = new THREE.BufferGeometry();

    geometry.setAttribute(
      'position',
      new THREE.Float32BufferAttribute(faceGroup.vertices, 3)
    );

    geometry.setAttribute(
      'uv',
      new THREE.Float32BufferAttribute(faceGroup.uvs, 2)
    );

    geometry.setAttribute(
      'normal',
      new THREE.Float32BufferAttribute(faceGroup.normals, 3)
    );

    geometry.setIndex(faceGroup.indices);

    const material = materialMap.get(options.material).clone();
    material.side = options.side;

    if (faceGroup.material) {
      material.name = faceGroup.material;
    }

    if (faceGroup.texture && options.material === "Texture") {
      requestTexture(faceGroup.texture).then(texture => {
        if (texture) {
          texture.anisotropy = renderer.capabilities.getMaxAnisotropy();
          material.map = texture;
          material.needsUpdate = true;
        }
      }).catch(err => {
        console.error(`Error loading texture for ${mesh.name}:`, err);
      });
    }

    const mesh = new THREE.Mesh(geometry, material);
    mesh.castShadow = true;
    mesh.receiveShadow = true;
    mesh.name = `${faceGroup.texture || 'unknown'}_${faceGroup.material || 'unknown'}`;

    group.add(mesh);

  });

  group.userData = {
    version_major: json.version_major,
    version_minor: json.version_minor,
    unknown_flags: json.unknown_flags,
    resolution: json.resolution
  };

  return group;
}

function initViewer() {

  setupRenderer();
  setupScene();

  if (typeof window.modelData === "string") {
    window.modelData = JSON.parse(window.modelData);
  }

  lodArray = window.modelData.lods || [window.modelData];

  lastPosition = new THREE.Vector3();
  lastRotation = new THREE.Euler();
  lastScale = new THREE.Vector3(1, 1, 1);

  const lightXSlider = document.getElementById('lightXSlider');
  const lightYSlider = document.getElementById('lightYSlider');
  const lightZSlider = document.getElementById('lightZSlider');

  function updateLightPosition() {
    directionalLight.position.set(
      parseFloat(lightXSlider.value),
      parseFloat(lightYSlider.value),
      parseFloat(lightZSlider.value)
    );
    lightHelper.update();
  }

  lightXSlider.addEventListener('input', updateLightPosition);
  lightYSlider.addEventListener('input', updateLightPosition);
  lightZSlider.addEventListener('input', updateLightPosition);

  document.addEventListener("keydown", (event) => {

    if (event.code === "KeyF") {
      controls.reset();
    }
  });

  const cullingSelect = document.getElementById('culling');

  for (let index = 0; index < cullingNames.length; index++) {

    const name = cullingNames[index];

    const option = document.createElement('option');
    option.value = index;
    option.textContent = name;

    cullingSelect.appendChild(option);
  }

  cullingSelect.addEventListener('change', (e) => {

    if (!currentModel) return;

    const culling = parseInt(e.target.value);
    if (isNaN(culling) || culling < 0 || culling >= 3) return;

    currentModel.traverse((child) => {
      if (child.isMesh) {
        child.material.side = culling;
      }
    });
  });

  const materialSelect = document.getElementById('material');

  for (const name of materialMap.keys()) {

    const option = document.createElement('option');
    option.value = name;
    option.textContent = name;

    materialSelect.appendChild(option);
  }

  materialSelect.addEventListener('change', (e) => {

    if (!currentModel) return;

    const materialKey = e.target.value;
    if (!materialMap.has(materialKey)) return;

    if (materialKey === "Texture") {

      loadSelectedLOD();
      return;
    }

    currentModel.traverse((child) => {
      if (child.isMesh) {
        child.material = materialMap.get(materialKey).clone();
      }
    });
  });

  const lodLevel = document.getElementById('lodLevel');

  if (lodArray.length > 0) {

    for (let index = 0; index < lodArray.length; index++) {

      const lod = lodArray[index];
      const option = document.createElement('option');
      option.value = index;
      const lodType = getLODTypeFromResolution(lod.resolution);
      option.textContent = `LOD ${index} - ${lodType} (${lod.points.length} points, ${lod.faces.length} faces)`;

      lodLevel.appendChild(option);
    }

    loadSelectedLOD();
  }

  lodLevel.addEventListener('change', loadSelectedLOD);

  window.addEventListener('resize', () => {
    camera.aspect = window.innerWidth / window.innerHeight;
    camera.updateProjectionMatrix();
    renderer.setSize(window.innerWidth, window.innerHeight);
    gizmo.update();
  });

}

window.addEventListener('DOMContentLoaded', initViewer);

const textureRequests = new Map();
async function requestTexture(path) {
  console.log(`Requesting texture: ${path}`);
  return new Promise((resolve, reject) => {
    const id = crypto.randomUUID();
    textureRequests.set(id, { resolve, reject });
    window.vscode.postMessage({
      command: 'requestTexture',
      texture: path,
      id: id
    });
  });
}

window.addEventListener('message', event => {
  const message = event.data;
  if (message.command === 'textureResponse') {
    const request = textureRequests.get(message.id);
    if (request) {
      textureRequests.delete(message.id);
      if (message.error) {
        request.reject(new Error(message.error));
      } else if (message.data) {
        const texture = new THREE.TextureLoader().load(`data:image/png;base64,${message.data}`);
        request.resolve(texture);
      } else {
        request.resolve(null);
      }
    }
  }
});
