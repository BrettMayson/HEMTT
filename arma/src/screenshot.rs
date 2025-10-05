use std::ptr;
use windows::Win32::Foundation::E_FAIL;
use windows::Win32::Graphics::Direct3D::*;
use windows::Win32::Graphics::Direct3D11::*;

pub unsafe fn capture_dx11_screen() -> Option<Vec<u8>> {
    let render_info = unsafe {
        let device_data_ptr =
            find_rv_function("RVExtensionGData") as *const *const RVExtensionRenderInfo;
        if device_data_ptr.is_null() {
            return None;
        }
        *device_data_ptr
    };
    if render_info.is_null() {
        return None;
    }

    let device: &ID3D11Device = &*render_info.d3d_device;
    let context: &ID3D11DeviceContext = &*render_info.d3d_device_context;

    // 1️⃣ Get the back buffer
    let swap_desc = D3D11_TEXTURE2D_DESC {
        Width: 0, // we will query later
        Height: 0,
        MipLevels: 1,
        ArraySize: 1,
        Format: DXGI_FORMAT_R8G8B8A8_UNORM,
        SampleDesc: DXGI_SAMPLE_DESC {
            Count: 1,
            Quality: 0,
        },
        Usage: D3D11_USAGE_DEFAULT,
        BindFlags: D3D11_BIND_RENDER_TARGET,
        CPUAccessFlags: 0,
        MiscFlags: 0,
    };

    // Actually, the proper way: enumerate render targets via context
    let mut backbuffer: Option<ID3D11Texture2D> = None;
    context.OMGetRenderTargets(1, &mut backbuffer as *mut _ as *mut _, ptr::null_mut());

    let backbuffer = match backbuffer {
        Some(bb) => bb,
        None => return None,
    };

    // 2️⃣ Create staging texture
    let mut desc = D3D11_TEXTURE2D_DESC::default();
    backbuffer.GetDesc(&mut desc);
    let staging_desc = D3D11_TEXTURE2D_DESC {
        Usage: D3D11_USAGE_STAGING,
        CPUAccessFlags: D3D11_CPU_ACCESS_READ,
        ..desc
    };

    let staging: ID3D11Texture2D = device.CreateTexture2D(&staging_desc, ptr::null())?;

    // 3️⃣ Copy resource
    context.CopyResource(&staging, &backbuffer);

    // 4️⃣ Map and read data
    let mut mapped = unsafe { std::mem::zeroed() };
    context.Map(&staging, 0, D3D11_MAP_READ, 0, &mut mapped)?;

    let row_pitch = mapped.RowPitch as usize;
    let height = desc.Height as usize;
    let data = unsafe {
        std::slice::from_raw_parts(mapped.pData as *const u8, row_pitch * height).to_vec()
    };

    context.Unmap(&staging, 0);

    Some(data)
}

pub fn screenshot() {
    let bytes = unsafe { capture_dx11_screen() };
}

unsafe fn find_rv_function(name: &str) -> *const () {
    let cname = CString::new(name).unwrap();
    #[cfg(target_os = "windows")]
    {
        GetProcAddress(GetModuleHandleA(None), cname.as_ptr())
            .map(|p| p as *const ())
            .unwrap_or(ptr::null())
    }
    #[cfg(target_os = "linux")]
    {
        let handle = dlopen(ptr::null(), RTLD_LAZY | RTLD_NOLOAD);
        if handle.is_null() {
            return ptr::null();
        }
        let symbol = dlsym(handle, cname.as_ptr());
        dlclose(handle);
        symbol as *const ()
    }
}

type RVExtensionGLockProc = unsafe extern "C" fn();
type RVExtensionGSetWHkProc = unsafe extern "C" fn();
#[repr(C)]
struct RVExtensionRenderInfo {
    d3d_device: *mut std::ffi::c_void,         // ID3D11Device*
    d3d_device_context: *mut std::ffi::c_void, // ID3D11DeviceContext*
}
