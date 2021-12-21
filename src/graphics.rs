use std::ptr;

use windows::Win32::{
    Foundation::{BOOL, HINSTANCE, HWND, PSTR, PWSTR},
    Graphics::{
        Direct3D::{
            Fxc::{D3DCompileFromFile, D3DCOMPILE_DEBUG, D3DCOMPILE_SKIP_OPTIMIZATION},
            ID3DBlob, D3D_DRIVER_TYPE_HARDWARE, D3D_FEATURE_LEVEL_11_0,
        },
        Direct3D11::{
            D3D11CreateDeviceAndSwapChain, ID3D11ClassInstance, ID3D11ClassLinkage, ID3D11Device,
            ID3D11DeviceContext, ID3D11RenderTargetView, ID3D11Resource, D3D11_BIND_VERTEX_BUFFER,
            D3D11_BUFFER_DESC, D3D11_CREATE_DEVICE_DEBUG, D3D11_CREATE_DEVICE_SINGLETHREADED,
            D3D11_SDK_VERSION, D3D11_SUBRESOURCE_DATA, D3D11_USAGE_DEFAULT,
        },
        Dxgi::{
            Common::{
                DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_MODE_DESC, DXGI_MODE_SCALING_UNSPECIFIED,
                DXGI_MODE_SCANLINE_ORDER_UNSPECIFIED, DXGI_RATIONAL, DXGI_SAMPLE_DESC,
            },
            IDXGISwapChain, DXGI_ERROR_DEVICE_REMOVED, DXGI_SWAP_CHAIN_DESC,
            DXGI_SWAP_EFFECT_DISCARD, DXGI_USAGE_RENDER_TARGET_OUTPUT,
        },
    },
};

use crate::{error::Win32Error, win32_common::ToWide};
pub type Result<T> = core::result::Result<T, Win32Error>;

pub struct Graphics {
    window_handle: HWND,
    device: ID3D11Device,
    swap_chain: IDXGISwapChain,
    device_context: ID3D11DeviceContext,
    render_target_view: ID3D11RenderTargetView,
    vertex_shader_blob: ID3DBlob,
    pixel_shader_blob: ID3DBlob,
}

impl Graphics {
    pub fn new(window_handle: HWND) -> Result<Self> {
        unsafe {
            let mut device: Option<ID3D11Device> = None;
            let mut swap_chain: Option<IDXGISwapChain> = None;
            let mut device_context: Option<ID3D11DeviceContext> = None;
            let swap_chain_description: DXGI_SWAP_CHAIN_DESC = {
                let buffer_descriptor = {
                    let refresh_rate = DXGI_RATIONAL {
                        Numerator: 0,
                        Denominator: 0,
                    };

                    DXGI_MODE_DESC {
                        Width: 0,
                        Height: 0,
                        RefreshRate: refresh_rate,
                        Format: DXGI_FORMAT_B8G8R8A8_UNORM,
                        ScanlineOrdering: DXGI_MODE_SCANLINE_ORDER_UNSPECIFIED,
                        Scaling: DXGI_MODE_SCALING_UNSPECIFIED,
                    }
                };

                let sample_descriptor = DXGI_SAMPLE_DESC {
                    Count: 1,
                    Quality: 0,
                };

                DXGI_SWAP_CHAIN_DESC {
                    BufferDesc: buffer_descriptor,
                    SampleDesc: sample_descriptor,
                    BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
                    BufferCount: 1,
                    OutputWindow: window_handle,
                    Windowed: BOOL(1),
                    SwapEffect: DXGI_SWAP_EFFECT_DISCARD,
                    Flags: 0,
                }
            };

            D3D11CreateDeviceAndSwapChain(
                None,
                D3D_DRIVER_TYPE_HARDWARE,
                HINSTANCE::default(),
                D3D11_CREATE_DEVICE_SINGLETHREADED | D3D11_CREATE_DEVICE_DEBUG,
                std::ptr::null(),
                0,
                D3D11_SDK_VERSION,
                &swap_chain_description,
                &mut swap_chain,
                &mut device,
                &mut D3D_FEATURE_LEVEL_11_0,
                &mut device_context,
            )
            .map_err(|e| win_error!(e))?;

            let buffer = swap_chain
                .as_mut()
                .unwrap()
                .GetBuffer::<ID3D11Resource>(0)
                .map_err(|e| win_error!(e))?;
            let render_target_view = {
                //     let target_view_desc = D3D11_RENDER_TARGET_VIEW_DESC {
                //         Format: DXGI_FORMAT_B8G8R8A8_UNORM,
                //         ViewDimension: D3D11_RTV_DIMENSION_TEXTURE2D,
                //         ..mem::zeroed()
                //     };

                device
                    .as_mut()
                    .unwrap()
                    .CreateRenderTargetView(buffer, ptr::null())
                    .map_err(|e| win_error!(e))?
            };

            // Create vertex shader
            let exe_path = std::env::current_exe().ok().unwrap();
            let asset_path = exe_path.parent().unwrap();
            let shaders_hlsl_path = asset_path.join("shaders.hlsl");
            let shaders_hlsl = shaders_hlsl_path.to_str().unwrap();
            println!("shader at: {shaders_hlsl}");

            let compile_flags = if cfg!(debug_assertions) {
                D3DCOMPILE_DEBUG | D3DCOMPILE_SKIP_OPTIMIZATION
            } else {
                0
            };

            let mut vertex_shader_blob = None;
            let vertex_shader_blob =
                // TODO Geert: MOVE THIS TO GFX INITIALIZATION. We don't want to compile from source each render!!
                D3DCompileFromFile(
                    PWSTR(shaders_hlsl.to_wide().as_mut_ptr()),
                    std::ptr::null_mut(),
                    None,
                    PSTR(b"VSMain\0".as_ptr() as *mut u8),
                    PSTR(b"vs_5_0\0".as_ptr() as *mut u8),
                    compile_flags,
                    0,
                    &mut vertex_shader_blob,
                    std::ptr::null_mut(),
                )
            .map(|()| vertex_shader_blob.unwrap())
            .map_err(|e| win_error!(e))?;

            // Create pixel shader
            let mut pixel_shader_blob = None;
            let pixel_shader_blob = D3DCompileFromFile(
                PWSTR(shaders_hlsl.to_wide().as_mut_ptr()),
                std::ptr::null_mut(),
                None,
                PSTR(b"PSMain\0".as_ptr() as *mut u8),
                PSTR(b"ps_5_0\0".as_ptr() as *mut u8),
                compile_flags,
                0,
                &mut pixel_shader_blob,
                std::ptr::null_mut(),
            )
            .map(|()| pixel_shader_blob.unwrap())
            .map_err(|e| win_error!(e))?;

            Ok(Self {
                window_handle,
                device: device.unwrap(),
                swap_chain: swap_chain.unwrap(),
                device_context: device_context.unwrap(),
                render_target_view,
                vertex_shader_blob,
                pixel_shader_blob,
            })
        }
    }

    pub fn present_frame(&self) -> Result<()> {
        unsafe {
            self.swap_chain.Present(1, 0).map_err(|e| {
                if let Some(hresult) = e.win32_error() {
                    if hresult == DXGI_ERROR_DEVICE_REMOVED.0 {
                        println!("{:?}", self.device.GetDeviceRemovedReason());
                    }
                }
                win_error!(e)
            })?;
        }

        Ok(())
    }

    pub fn clear_buffer(&mut self, red: f32, green: f32, blue: f32) {
        let colorrgba = [red, green, blue, 1.0].as_ptr();
        unsafe {
            self.device_context
                .ClearRenderTargetView(&self.render_target_view, colorrgba);
        }
    }

    pub fn draw_test_triangle(&mut self) -> Result<()> {
        let bd = D3D11_BUFFER_DESC {
            ByteWidth: core::mem::size_of::<[Vertex; 3]>() as u32,
            Usage: D3D11_USAGE_DEFAULT,
            BindFlags: D3D11_BIND_VERTEX_BUFFER.0,
            CPUAccessFlags: 0,
            MiscFlags: 0,
            StructureByteStride: core::mem::size_of::<Vertex>() as u32,
        };

        let sd = D3D11_SUBRESOURCE_DATA {
            pSysMem: unsafe { std::mem::transmute(TRIANGLE.as_mut_ptr()) },
            SysMemPitch: 0,
            SysMemSlicePitch: 0,
        };

        let vertex_buffer = unsafe {
            self.device
                .CreateBuffer(&bd, &sd)
                .map_err(|e| win_error!(e))?
        };

        // Bind vertex buffer to pipeline
        let stride = core::mem::size_of::<Vertex>() as u32;
        let offset = 0;

        unsafe {
            self.device_context
                .IASetVertexBuffers(0, 1, &Some(vertex_buffer), &stride, &offset)
        };

        let vertex_shader = unsafe {
            let class_linkage = self
                .device
                .CreateClassLinkage()
                .map_err(|e| win_error!(e))?;
            self.device
                .CreateVertexShader(
                    self.vertex_shader_blob.GetBufferPointer(),
                    self.vertex_shader_blob.GetBufferSize(),
                    class_linkage,
                )
                .map_err(|e| win_error!(e))?
        };

        let class_instance: Option<ID3D11ClassInstance> = None;
        unsafe {
            self.device_context
                .VSSetShader(&vertex_shader, &class_instance, 0);
        }

        let pixel_shader = unsafe {
            let class_linkage = self
                .device
                .CreateClassLinkage()
                .map_err(|e| win_error!(e))?;
            self.device
                .CreatePixelShader(
                    self.pixel_shader_blob.GetBufferPointer(),
                    self.pixel_shader_blob.GetBufferSize(),
                    class_linkage,
                )
                .map_err(|e| win_error!(e))?
        };

        let class_instance: Option<ID3D11ClassInstance> = None;
        unsafe {
            self.device_context
                .PSSetShader(&pixel_shader, &class_instance, 0);
        }

        unsafe {
            self.device_context
                .Draw(core::mem::size_of::<[Vertex; 3]> as u32, 0);
        }
        Ok(())
    }
}

struct Vertex {
    x: f32,
    y: f32,
}

const TRIANGLE: [Vertex; 3] = [
    Vertex { x: 0.0, y: 0.5 },
    Vertex { x: 0.5, y: 0.5 },
    Vertex { x: -0.5, y: -0.5 },
];
