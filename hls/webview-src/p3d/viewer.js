import * as THREE from 'three';
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js';
import { ViewportGizmo } from "three-viewport-gizmo";

// Global variables to store state
let scene, camera, renderer, controls, gizmo, lightHelper, directionalLight;
let lodArray = [];
let currentModel = null;
let lastPosition, lastRotation, lastScale;

const materialMap = new Map();
materialMap.set("Standard", new THREE.MeshStandardMaterial({ color: 0xcccccc }));
materialMap.set("Wireframe", new THREE.MeshBasicMaterial({ color: 0xcccccc, wireframe: true }));
materialMap.set("Flat", new THREE.MeshStandardMaterial({ color: 0xcccccc, flatShading: true }));
materialMap.set("Unlit", new THREE.MeshBasicMaterial({ color: 0xcccccc }));
materialMap.set("Depth", new THREE.MeshDepthMaterial());
materialMap.set("Normal", new THREE.MeshNormalMaterial());

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
    material: materialMap.get(materialKey),
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

      group.uvs.push(vertex.uv[0], vertex.uv[1]);

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

    const material = options.material.clone();
    material.side = options.side;

    if (faceGroup.material) {
      material.name = faceGroup.material;
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

    currentModel.traverse((child) => {
      if (child.isMesh) {
        child.material = materialMap.get(materialKey);
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
