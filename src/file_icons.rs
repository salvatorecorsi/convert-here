//! Cached Windows file-type icons. Shell calls and bitmap conversion run on a
//! worker; only completed pixels are uploaded as egui textures on the UI thread.
use eframe::egui::{self, TextureHandle, TextureId};
use std::{
    collections::{HashMap, HashSet},
    path::Path,
    sync::mpsc,
};

const MAX_TYPES: usize = 256;
const ICON_SIZE: usize = 32;

struct IconPixels {
    rgba: Vec<u8>,
}

struct Request {
    extension: String,
    repaint: egui::Context,
}

struct Completed {
    extension: String,
    image: Option<IconPixels>,
}

pub struct FileIcons {
    cache: HashMap<String, Option<TextureHandle>>,
    pending: HashSet<String>,
    requests: mpsc::SyncSender<Request>,
    completed: mpsc::Receiver<Completed>,
}

impl Default for FileIcons {
    fn default() -> Self {
        let (request_tx, request_rx) = mpsc::sync_channel::<Request>(64);
        let (completed_tx, completed_rx) = mpsc::channel();
        let _ = std::thread::Builder::new()
            .name("file-type-icons".into())
            .spawn(move || {
                #[cfg(windows)]
                let _apartment = windows::Apartment::enter();
                while let Ok(request) = request_rx.recv() {
                    #[cfg(windows)]
                    let image = windows::load(&request.extension);
                    #[cfg(not(windows))]
                    let image = None;
                    if completed_tx
                        .send(Completed {
                            extension: request.extension,
                            image,
                        })
                        .is_err()
                    {
                        break;
                    }
                    request.repaint.request_repaint();
                }
            });
        Self {
            cache: HashMap::new(),
            pending: HashSet::new(),
            requests: request_tx,
            completed: completed_rx,
        }
    }
}

impl FileIcons {
    /// Returns the cached type icon, or schedules a lookup and returns `None`.
    /// The caller can show a Fluent document glyph while the icon is loading.
    /// This method does not access the file, network, registry or shell.
    pub fn get(&mut self, ctx: &egui::Context, path: &Path) -> Option<TextureId> {
        // Bound uploads per call so a large batch cannot monopolize a frame.
        for _ in 0..8 {
            let Ok(completed) = self.completed.try_recv() else {
                break;
            };
            self.pending.remove(&completed.extension);
            let texture = completed.image.map(|image| {
                ctx.load_texture(
                    format!("windows-file-type-{}", completed.extension),
                    egui::ColorImage::from_rgba_unmultiplied([ICON_SIZE, ICON_SIZE], &image.rgba),
                    egui::TextureOptions::LINEAR,
                )
            });
            self.cache.insert(completed.extension, texture);
        }
        let extension = type_key(path);
        if let Some(texture) = self.cache.get(&extension) {
            return texture.as_ref().map(TextureHandle::id);
        }
        if self.cache.len() + self.pending.len() < MAX_TYPES && !self.pending.contains(&extension) {
            let request = Request {
                extension: extension.clone(),
                repaint: ctx.clone(),
            };
            if self.requests.try_send(request).is_ok() {
                self.pending.insert(extension);
            }
        }
        None
    }
}

fn type_key(path: &Path) -> String {
    path.extension()
        .and_then(|extension| extension.to_str())
        .filter(|extension| {
            extension.len() <= 32
                && extension
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'+'))
        })
        .unwrap_or("")
        .to_ascii_lowercase()
}

/// Recover straight RGBA from an icon painted onto opaque black and white.
/// This preserves both modern premultiplied-alpha icons and legacy AND masks.
#[cfg(any(windows, test))]
fn recover_alpha(black: &[u8], white: &[u8]) -> Option<IconPixels> {
    if black.len() != white.len() || black.len() != ICON_SIZE * ICON_SIZE * 4 {
        return None;
    }
    let mut rgba = Vec::with_capacity(black.len());
    for (black, white) in black
        .as_chunks::<4>()
        .0
        .iter()
        .zip(white.as_chunks::<4>().0)
    {
        let remaining = (0..3)
            .map(|channel| white[channel].saturating_sub(black[channel]))
            .max()?;
        let alpha = 255 - remaining;
        for channel in [2, 1, 0] {
            rgba.push(if alpha == 0 {
                0
            } else {
                ((black[channel] as u32 * 255 + alpha as u32 / 2) / alpha as u32).min(255) as u8
            });
        }
        rgba.push(alpha);
    }
    Some(IconPixels { rgba })
}

#[cfg(windows)]
mod windows {
    use super::{ICON_SIZE, IconPixels, recover_alpha};
    use std::{ffi::c_void, ptr};

    type Handle = *mut c_void;

    #[repr(C)]
    struct FileInfo {
        icon: Handle,
        index: i32,
        attributes: u32,
        display_name: [u16; 260],
        type_name: [u16; 80],
    }

    #[repr(C)]
    struct BitmapInfo {
        size: u32,
        width: i32,
        height: i32,
        planes: u16,
        bits: u16,
        compression: u32,
        image_size: u32,
        x_resolution: i32,
        y_resolution: i32,
        colours: u32,
        important_colours: u32,
        palette: [u32; 1],
    }

    #[link(name = "shell32")]
    unsafe extern "system" {
        fn SHGetFileInfoW(
            path: *const u16,
            attributes: u32,
            info: *mut FileInfo,
            size: u32,
            flags: u32,
        ) -> usize;
    }
    #[link(name = "user32")]
    unsafe extern "system" {
        fn DestroyIcon(icon: Handle) -> i32;
        fn DrawIconEx(
            context: Handle,
            x: i32,
            y: i32,
            icon: Handle,
            width: i32,
            height: i32,
            step: u32,
            brush: Handle,
            flags: u32,
        ) -> i32;
    }
    #[link(name = "gdi32")]
    unsafe extern "system" {
        fn CreateCompatibleDC(context: Handle) -> Handle;
        fn CreateDIBSection(
            context: Handle,
            info: *const BitmapInfo,
            usage: u32,
            pixels: *mut *mut c_void,
            section: Handle,
            offset: u32,
        ) -> Handle;
        fn SelectObject(context: Handle, object: Handle) -> Handle;
        fn DeleteObject(object: Handle) -> i32;
        fn DeleteDC(context: Handle) -> i32;
        fn GdiFlush() -> i32;
    }
    #[link(name = "ole32")]
    unsafe extern "system" {
        fn CoInitializeEx(reserved: *mut c_void, flags: u32) -> i32;
        fn CoUninitialize();
    }

    pub(super) struct Apartment(bool);
    impl Apartment {
        pub(super) fn enter() -> Self {
            Self(unsafe { CoInitializeEx(ptr::null_mut(), 2 | 4) } >= 0)
        }
    }
    impl Drop for Apartment {
        fn drop(&mut self) {
            if self.0 {
                unsafe {
                    CoUninitialize();
                }
            }
        }
    }

    struct Icon(Handle);
    impl Drop for Icon {
        fn drop(&mut self) {
            if !self.0.is_null() {
                unsafe {
                    DestroyIcon(self.0);
                }
            }
        }
    }

    struct Surface {
        context: Handle,
        bitmap: Handle,
        previous: Handle,
        pixels: *mut u8,
    }

    impl Surface {
        fn new() -> Option<Self> {
            let context = unsafe { CreateCompatibleDC(ptr::null_mut()) };
            if context.is_null() {
                return None;
            }
            let mut surface = Self {
                context,
                bitmap: ptr::null_mut(),
                previous: ptr::null_mut(),
                pixels: ptr::null_mut(),
            };
            let info = BitmapInfo {
                size: 40,
                width: ICON_SIZE as i32,
                height: -(ICON_SIZE as i32),
                planes: 1,
                bits: 32,
                compression: 0,
                image_size: 0,
                x_resolution: 0,
                y_resolution: 0,
                colours: 0,
                important_colours: 0,
                palette: [0],
            };
            let mut pixels = ptr::null_mut();
            surface.bitmap =
                unsafe { CreateDIBSection(context, &info, 0, &mut pixels, ptr::null_mut(), 0) };
            if surface.bitmap.is_null() || pixels.is_null() {
                return None;
            }
            surface.previous = unsafe { SelectObject(context, surface.bitmap) };
            if surface.previous.is_null() || surface.previous as isize == -1 {
                surface.previous = ptr::null_mut();
                return None;
            }
            surface.pixels = pixels.cast();
            Some(surface)
        }

        fn draw(&mut self, icon: Handle, background: u8) -> Option<Vec<u8>> {
            {
                let pixels = unsafe {
                    std::slice::from_raw_parts_mut(self.pixels, ICON_SIZE * ICON_SIZE * 4)
                };
                for pixel in pixels.as_chunks_mut::<4>().0 {
                    pixel.copy_from_slice(&[background, background, background, 255]);
                }
            }
            let success = unsafe {
                DrawIconEx(
                    self.context,
                    0,
                    0,
                    icon,
                    ICON_SIZE as i32,
                    ICON_SIZE as i32,
                    0,
                    ptr::null_mut(),
                    3 | 0x10,
                )
            };
            unsafe {
                GdiFlush();
            }
            if success == 0 {
                None
            } else {
                Some(unsafe {
                    std::slice::from_raw_parts(self.pixels, ICON_SIZE * ICON_SIZE * 4).to_vec()
                })
            }
        }
    }

    impl Drop for Surface {
        fn drop(&mut self) {
            unsafe {
                if !self.previous.is_null() {
                    SelectObject(self.context, self.previous);
                }
                if !self.bitmap.is_null() {
                    DeleteObject(self.bitmap);
                }
                if !self.context.is_null() {
                    DeleteDC(self.context);
                }
            }
        }
    }

    pub(super) fn load(extension: &str) -> Option<IconPixels> {
        // USEFILEATTRIBUTES queries the registered type association, without
        // touching any real input, network share, thumbnail or per-file handler.
        let filename = if extension.is_empty() {
            "file".to_owned()
        } else {
            format!("file.{extension}")
        };
        let wide: Vec<u16> = filename.encode_utf16().chain(Some(0)).collect();
        let mut info: FileInfo = unsafe { std::mem::zeroed() };
        let result = unsafe {
            SHGetFileInfoW(
                wide.as_ptr(),
                0x80,
                &mut info,
                std::mem::size_of::<FileInfo>() as u32,
                0x100 | 0x10,
            )
        };
        let icon = Icon(info.icon);
        if result == 0 || icon.0.is_null() {
            return None;
        }
        let mut surface = Surface::new()?;
        let black = surface.draw(icon.0, 0)?;
        let white = surface.draw(icon.0, 255)?;
        recover_alpha(&black, &white)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_keys_use_type_associations_without_file_access() {
        assert_eq!(type_key(Path::new(r"C:\missing\REPORT.PDF")), "pdf");
        assert_eq!(type_key(Path::new("no-extension")), "");
        assert_eq!(type_key(Path::new("file.odd:type")), "");
    }

    #[test]
    fn two_backgrounds_preserve_transparent_opaque_and_translucent_pixels() {
        let mut black = vec![0u8; ICON_SIZE * ICON_SIZE * 4];
        let mut white = vec![255u8; black.len()];
        black[4..8].copy_from_slice(&[30, 60, 200, 255]);
        white[4..8].copy_from_slice(&[30, 60, 200, 255]);
        black[8..12].copy_from_slice(&[16, 32, 64, 255]);
        white[8..12].copy_from_slice(&[143, 159, 191, 255]);
        let image = recover_alpha(&black, &white).unwrap();
        assert_eq!(&image.rgba[..4], &[0, 0, 0, 0]);
        assert_eq!(&image.rgba[4..8], &[200, 60, 30, 255]);
        assert_eq!(&image.rgba[8..12], &[128, 64, 32, 128]);
    }

    #[cfg(windows)]
    #[test]
    fn shell_type_icon_produces_real_pixels_with_alpha() {
        let _apartment = windows::Apartment::enter();
        let icon =
            windows::load("pdf").expect("Windows must provide a PDF or generic file type icon");
        assert_eq!(icon.rgba.len(), ICON_SIZE * ICON_SIZE * 4);
        assert!(
            icon.rgba
                .as_chunks::<4>()
                .0
                .iter()
                .any(|pixel| pixel[3] != 0)
        );
        assert!(
            icon.rgba
                .as_chunks::<4>()
                .0
                .iter()
                .any(|pixel| pixel[3] == 0)
        );
    }
}
