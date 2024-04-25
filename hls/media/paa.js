
let wasm;

const heap = new Array(32).fill(undefined);

heap.push(undefined, null, true, false);

function getObject(idx) { return heap[idx]; }

let heap_next = heap.length;

function dropObject(idx) {
  if (idx < 36) return;
  heap[idx] = heap_next;
  heap_next = idx;
}

function takeObject(idx) {
  const ret = getObject(idx);
  dropObject(idx);
  return ret;
}

let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });

cachedTextDecoder.decode();

let cachegetUint8Memory0 = null;
function getUint8Memory0() {
  if (cachegetUint8Memory0 === null || cachegetUint8Memory0.buffer !== wasm.memory.buffer) {
    cachegetUint8Memory0 = new Uint8Array(wasm.memory.buffer);
  }
  return cachegetUint8Memory0;
}

function getStringFromWasm0(ptr, len) {
  return cachedTextDecoder.decode(getUint8Memory0().subarray(ptr, ptr + len));
}

function addHeapObject(obj) {
  if (heap_next === heap.length) heap.push(heap.length + 1);
  const idx = heap_next;
  heap_next = heap[idx];

  heap[idx] = obj;
  return idx;
}
/**
*/
class ImageResult {

  static __wrap(ptr) {
    const obj = Object.create(ImageResult.prototype);
    obj.ptr = ptr;

    return obj;
  }

  free() {
    const ptr = this.ptr;
    this.ptr = 0;

    wasm.__wbg_imageresult_free(ptr);
  }
  /**
  * @param {Uint8Array} s
  */
  constructor(s) {
    var ret = wasm.imageresult_new(addHeapObject(s));
    return ImageResult.__wrap(ret);
  }
  /**
  * @returns {number}
  */
  data_ptr() {
    var ret = wasm.imageresult_data_ptr(this.ptr);
    return ret;
  }
  /**
  * @returns {number}
  */
  data_len() {
    var ret = wasm.imageresult_data_len(this.ptr);
    return ret >>> 0;
  }
}

async function load(module, imports) {
  if (typeof Response === 'function' && module instanceof Response) {

    if (typeof WebAssembly.instantiateStreaming === 'function') {
      try {
        return await WebAssembly.instantiateStreaming(module, imports);

      } catch (e) {
        if (module.headers.get('Content-Type') !== 'application/wasm') {
          console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

        } else {
          throw e;
        }
      }
    }

    const bytes = await module.arrayBuffer();
    return await WebAssembly.instantiate(bytes, imports);

  } else {

    const instance = await WebAssembly.instantiate(module, imports);

    if (instance instanceof WebAssembly.Instance) {
      return { instance, module };

    } else {
      return instance;
    }
  }
}

async function init(input) {
  const imports = {};
  imports.wbg = {};
  imports.wbg.__wbindgen_object_drop_ref = function (arg0) {
    takeObject(arg0);
  };
  imports.wbg.__wbg_buffer_eb5185aa4a8e9c62 = function (arg0) {
    var ret = getObject(arg0).buffer;
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_length_2e324c9c0e74a81d = function (arg0) {
    var ret = getObject(arg0).length;
    return ret;
  };
  imports.wbg.__wbg_new_3d94e83f0a6bf252 = function (arg0) {
    var ret = new Uint8Array(getObject(arg0));
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_set_d4d7629a896d4b3e = function (arg0, arg1, arg2) {
    getObject(arg0).set(getObject(arg1), arg2 >>> 0);
  };
  imports.wbg.__wbindgen_throw = function (arg0, arg1) {
    throw new Error(getStringFromWasm0(arg0, arg1));
  };
  imports.wbg.__wbindgen_memory = function () {
    var ret = wasm.memory;
    return addHeapObject(ret);
  };

  if (typeof input === 'string' || (typeof Request === 'function' && input instanceof Request) || (typeof URL === 'function' && input instanceof URL)) {
    input = fetch(input);
  }

  const { instance, module } = await load(await input, imports);

  wasm = instance.exports;
  init.__wbindgen_wasm_module = module;

  return wasm;
}
