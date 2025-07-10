use egui::TextureHandle;
#[cfg(windows)]
use egui::ColorImage;
use std::collections::HashMap;

#[cfg(windows)]
use winapi::{
    shared::windef::HWND,
    um::{
        wingdi::{
            BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject,
            GetDIBits, SelectObject, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS,
            SRCCOPY,
        },
        winuser::{GetWindowDC, ReleaseDC, PrintWindow, IsWindowVisible, IsIconic, PW_RENDERFULLCONTENT},
    },
};

pub struct ThumbnailCache {
    textures: HashMap<isize, TextureHandle>,
}

impl ThumbnailCache {
    pub fn new() -> Self {
        Self {
            textures: HashMap::new(),
        }
    }
    
    #[cfg(windows)]
    pub fn get_or_create_thumbnail(
        &mut self,
        ctx: &egui::Context,
        hwnd: isize,
        size: (u32, u32),
    ) -> Option<&TextureHandle> {
        if self.textures.contains_key(&hwnd) {
            return self.textures.get(&hwnd);
        }
        
        let image = capture_window_thumbnail(hwnd, size)?;
        let texture = ctx.load_texture(
            format!("window_{}", hwnd),
            image,
            egui::TextureOptions::default(),
        );
        
        self.textures.insert(hwnd, texture);
        self.textures.get(&hwnd)
    }
    
    #[cfg(not(windows))]
    pub fn get_or_create_thumbnail(
        &mut self,
        _ctx: &egui::Context,
        _hwnd: isize,
        _size: (u32, u32),
    ) -> Option<&TextureHandle> {
        None
    }
    
    pub fn clear(&mut self) {
        self.textures.clear();
    }
}

#[cfg(windows)]
fn capture_window_thumbnail(hwnd: isize, target_size: (u32, u32)) -> Option<ColorImage> {
    unsafe {
        let hwnd = hwnd as HWND;
        
        // Check if window is visible and not minimized
        if IsWindowVisible(hwnd) == 0 || IsIconic(hwnd) != 0 {
            return None;
        }
        
        let mut rect = std::mem::zeroed();
        winapi::um::winuser::GetWindowRect(hwnd, &mut rect);
        
        let width = rect.right - rect.left;
        let height = rect.bottom - rect.top;
        
        if width <= 0 || height <= 0 {
            return None;
        }
        
        // Create a device context for the window
        let window_dc = GetWindowDC(hwnd);
        if window_dc.is_null() {
            return None;
        }
        
        // Create a compatible DC and bitmap
        let mem_dc = CreateCompatibleDC(window_dc);
        let bmp = CreateCompatibleBitmap(window_dc, width, height);
        let old_bmp = SelectObject(mem_dc, bmp as *mut _);
        
        // Try PrintWindow first for better results
        let success = PrintWindow(hwnd, mem_dc, PW_RENDERFULLCONTENT);
        
        if success == 0 {
            // Fallback to BitBlt if PrintWindow fails
            BitBlt(
                mem_dc,
                0,
                0,
                width,
                height,
                window_dc,
                0,
                0,
                SRCCOPY,
            );
        }
        
        // Get the bitmap data
        let mut bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width,
                biHeight: -height, // negative for top-down bitmap
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB,
                biSizeImage: 0,
                biXPelsPerMeter: 0,
                biYPelsPerMeter: 0,
                biClrUsed: 0,
                biClrImportant: 0,
            },
            bmiColors: [std::mem::zeroed()],
        };
        
        let size = (width * height * 4) as usize;
        let mut pixels = vec![0u8; size];
        
        GetDIBits(
            mem_dc,
            bmp,
            0,
            height as u32,
            pixels.as_mut_ptr() as *mut _,
            &mut bmi,
            DIB_RGB_COLORS,
        );
        
        // Cleanup
        SelectObject(mem_dc, old_bmp);
        DeleteObject(bmp as *mut _);
        DeleteDC(mem_dc);
        ReleaseDC(hwnd, window_dc);
        
        // Convert BGRA to RGBA
        for chunk in pixels.chunks_exact_mut(4) {
            chunk.swap(0, 2);
        }
        
        // Scale to target size
        let scaled = scale_image(
            &pixels,
            width as u32,
            height as u32,
            target_size.0,
            target_size.1,
        );
        
        Some(ColorImage::from_rgba_unmultiplied(
            [target_size.0 as usize, target_size.1 as usize],
            &scaled,
        ))
    }
}

fn scale_image(
    src: &[u8],
    src_width: u32,
    src_height: u32,
    dst_width: u32,
    dst_height: u32,
) -> Vec<u8> {
    let mut dst = vec![0u8; (dst_width * dst_height * 4) as usize];
    
    let x_ratio = src_width as f32 / dst_width as f32;
    let y_ratio = src_height as f32 / dst_height as f32;
    
    for y in 0..dst_height {
        for x in 0..dst_width {
            let src_x = (x as f32 * x_ratio) as u32;
            let src_y = (y as f32 * y_ratio) as u32;
            
            let src_idx = ((src_y * src_width + src_x) * 4) as usize;
            let dst_idx = ((y * dst_width + x) * 4) as usize;
            
            if src_idx + 3 < src.len() && dst_idx + 3 < dst.len() {
                dst[dst_idx..dst_idx + 4].copy_from_slice(&src[src_idx..src_idx + 4]);
            }
        }
    }
    
    dst
}