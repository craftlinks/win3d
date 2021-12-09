use std::ptr;

use windows::Win32::{
    Foundation::{BOOL, HINSTANCE, HWND},
    Graphics::{
        Direct3D::{D3D_DRIVER_TYPE_HARDWARE, D3D_FEATURE_LEVEL_11_0},
        Direct3D11::{
            D3D11CreateDeviceAndSwapChain, ID3D11Device, ID3D11DeviceContext,
            ID3D11RenderTargetView, ID3D11Resource, D3D11_CREATE_DEVICE_SINGLETHREADED,
            D3D11_SDK_VERSION, D3D11_CREATE_DEVICE_DEBUG,
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

use crate::error::Win32Error;
pub type Result<T> = core::result::Result<T, Win32Error>;

pub struct Graphics {
    window_handle: HWND,
    device: ID3D11Device,
    swap_chain: IDXGISwapChain,
    device_context: ID3D11DeviceContext,
    render_target_view: ID3D11RenderTargetView,
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

            Ok(Self {
                window_handle,
                device: device.unwrap(),
                swap_chain: swap_chain.unwrap(),
                device_context: device_context.unwrap(),
                render_target_view,
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
}
