//! System cursor extraction and global arrow hide/restore for the GPU cursor bypass.
//!
//! Extraction reads the shape behind the cursor handle reported by `GetCursorInfo`, converts
//! it to premultiplied top-down RGBA8, and hands it to the renderer. Hiding installs a 1×1
//! transparent cursor as the system arrow via `SetSystemCursor` (which **persists across
//! process death**), so the original is snapshotted first and restored on disable, exit and
//! panic — see [`force_restore`].
//!
//! Non-Windows platforms have no extraction backend yet: the bypass draws nothing and never
//! hides the system cursor.

use fxcursor_render::CursorShape;
use std::sync::Arc;

/// What the pointer shows right now.
#[derive(Clone)]
pub enum CursorSnapshot {
    /// The OS hides the pointer (fullscreen video, "hide pointer while typing", games): the
    /// GPU copy must disappear too.
    Hidden,
    /// Shape to draw. Shared, so the per-frame call never copies the pixels.
    Shape(Arc<CursorShape>),
    /// No shape could be read at the moment: keep whatever is drawn.
    Unavailable,
}

/// Reads the cursor currently shown at the pointer position.
///
/// Cheap to call every frame: shapes are cached per source handle (a few entries, so moving
/// between an arrow and an I-beam does not re-extract), and the pixel conversion only runs on
/// a cache miss. Call [`invalidate_cache`] now and then to pick up in-place changes (a new
/// cursor scheme or pointer size keeps the same handles).
pub fn extract_current() -> CursorSnapshot {
    platform::extract_current()
}

/// Drops every cached shape; the next [`extract_current`] re-reads the OS cursor.
pub fn invalidate_cache() {
    platform::invalidate_cache();
}

/// Mirrors `want_hidden` for the system arrow: installs or removes the invisible cursor.
/// Idempotent — repeated calls with the same value are no-ops. Hiding is refused once
/// [`force_restore`] has run (shutdown), so a render frame racing the exit path cannot hide the
/// arrow again after it was restored.
pub fn set_arrow_hidden(want_hidden: bool) {
    platform::set_arrow_hidden(want_hidden);
}

/// Restores the system arrow if this process hid it and latches "shutting down" so it is never
/// hidden again. Safe to call from any state, any number of times (exit path, panic hook).
pub fn force_restore() {
    platform::force_restore();
}

#[cfg(not(windows))]
mod platform {
    use super::CursorSnapshot;

    pub fn extract_current() -> CursorSnapshot {
        CursorSnapshot::Unavailable
    }

    pub fn invalidate_cache() {}

    pub fn set_arrow_hidden(_want_hidden: bool) {}

    pub fn force_restore() {}
}

#[cfg(windows)]
mod platform {
    use super::CursorSnapshot;
    use fxcursor_render::CursorShape;
    use std::sync::atomic::{AtomicBool, AtomicIsize, Ordering};
    use std::sync::{Arc, Mutex};
    use windows_sys::Win32::Graphics::Gdi::{
        DeleteObject, GetDC, GetDIBits, GetObjectW, ReleaseDC, BITMAP, BITMAPINFO,
        BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, HBITMAP, HDC, RGBQUAD,
    };
    use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        CopyIcon, CreateIcon, DestroyIcon, GetCursorInfo, GetIconInfo, LoadCursorW,
        SetSystemCursor, SystemParametersInfoW, CURSORINFO, CURSOR_SHOWING, HCURSOR, HICON,
        ICONINFO, IDC_ARROW, OCR_NORMAL, SPI_SETCURSORS, SPIF_UPDATEINIFILE,
    };

    /// True while this process has replaced the system arrow with the invisible cursor.
    static HIDDEN: AtomicBool = AtomicBool::new(false);
    /// Set by [`force_restore`]: the process is exiting (or panicked), never hide again.
    static SHUTTING_DOWN: AtomicBool = AtomicBool::new(false);
    /// Serialises hide/restore: the render thread and the exit / panic paths can race.
    static SWAP_LOCK: Mutex<()> = Mutex::new(());
    /// Saved original arrow (`CopyIcon`), consumed by the matching restore.
    static ORIGINAL: AtomicIsize = AtomicIsize::new(0);
    /// Recently extracted shapes by source handle, most recent first.
    static CACHE: Mutex<Vec<(isize, Arc<CursorShape>)>> = Mutex::new(Vec::new());
    /// Distinct cursors kept (arrow, I-beam, hand, resize arrows, ...).
    const CACHE_ENTRIES: usize = 8;

    pub fn invalidate_cache() {
        CACHE.lock().unwrap_or_else(|p| p.into_inner()).clear();
    }

    pub fn extract_current() -> CursorSnapshot {
        let mut ci = CURSORINFO {
            cbSize: std::mem::size_of::<CURSORINFO>() as u32,
            flags: 0,
            hCursor: std::ptr::null_mut(),
            ptScreenPos: unsafe { std::mem::zeroed() },
        };
        if unsafe { GetCursorInfo(&mut ci) } == 0 {
            return CursorSnapshot::Unavailable;
        }
        if ci.flags & CURSOR_SHOWING == 0 || ci.hCursor.is_null() {
            return CursorSnapshot::Hidden;
        }
        let shown = ci.hCursor as isize;

        // `SetSystemCursor` replaces the *contents* of the system arrow: while it is hidden
        // the pointer still reports the stock arrow handle (now showing the 1×1 invisible
        // image), so reading that handle yields nothing. The real shape is the saved original.
        let stock_arrow = unsafe { LoadCursorW(std::ptr::null_mut(), IDC_ARROW) } as isize;
        let is_arrow = shown == stock_arrow;
        let source: HCURSOR = if is_arrow && HIDDEN.load(Ordering::Relaxed) {
            let original = ORIGINAL.load(Ordering::Relaxed);
            if original == 0 {
                return CursorSnapshot::Unavailable;
            }
            original as HCURSOR
        } else {
            ci.hCursor
        };
        let key = source as isize;

        {
            let mut cache = CACHE.lock().unwrap_or_else(|p| p.into_inner());
            if let Some(i) = cache.iter().position(|(k, _)| *k == key) {
                let entry = cache.remove(i);
                let shape = entry.1.clone();
                cache.insert(0, entry);
                return CursorSnapshot::Shape(shape);
            }
        }

        let Some(shape) = extract_icon(source, key, is_arrow) else {
            return CursorSnapshot::Unavailable;
        };
        let shape = Arc::new(shape);
        let mut cache = CACHE.lock().unwrap_or_else(|p| p.into_inner());
        cache.insert(0, (key, shape.clone()));
        cache.truncate(CACHE_ENTRIES);
        CursorSnapshot::Shape(shape)
    }

    pub fn force_restore() {
        SHUTTING_DOWN.store(true, Ordering::SeqCst);
        set_arrow_hidden(false);
    }

    pub fn set_arrow_hidden(want_hidden: bool) {
        let _guard = SWAP_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        if want_hidden && SHUTTING_DOWN.load(Ordering::SeqCst) {
            return;
        }
        if HIDDEN.load(Ordering::Relaxed) == want_hidden {
            return;
        }

        if want_hidden {
            // Snapshot the live arrow BEFORE touching the system cursor slot.
            let arrow = unsafe { LoadCursorW(std::ptr::null_mut(), IDC_ARROW) };
            let saved: HCURSOR = if arrow.is_null() {
                std::ptr::null_mut()
            } else {
                unsafe { CopyIcon(arrow as HICON) as HCURSOR }
            };

            let invisible = create_invisible_icon();
            if invisible.is_null() {
                return;
            }
            // `SetSystemCursor` consumes the handle it is given.
            if unsafe { SetSystemCursor(invisible, OCR_NORMAL) } == 0 {
                unsafe { DestroyIcon(invisible) };
                return;
            }
            ORIGINAL.store(saved as isize, Ordering::Relaxed);
            HIDDEN.store(true, Ordering::Relaxed);
        } else {
            let original = ORIGINAL.swap(0, Ordering::Relaxed) as HICON;
            unsafe {
                if !original.is_null() {
                    // Hand the snapshot to the system (it takes ownership)…
                    SetSystemCursor(original, OCR_NORMAL);
                }
                // …and force every system cursor slot back to its stock shape in case
                // another application raced us while the arrow was replaced.
                SystemParametersInfoW(SPI_SETCURSORS, 0, std::ptr::null_mut(), SPIF_UPDATEINIFILE);
            }
            HIDDEN.store(false, Ordering::Relaxed);
            invalidate_cache();
        }
    }

    /// Builds a 1×1 fully transparent cursor: AND mask all-ones (transparent), XOR all-zero.
    fn create_invisible_icon() -> HICON {
        let and = [0xFFu32]; // 1bpp row, DWORD-aligned: every bit set → transparent
        let xor = [0x00u32]; // 32bpp colour pixel (unused, AND wins)
        unsafe {
            CreateIcon(
                GetModuleHandleW(std::ptr::null()),
                1,
                1,
                1,
                32,
                and.as_ptr() as *const u8,
                xor.as_ptr() as *const u8,
            )
        }
    }

    /// Converts a cursor/icon handle to premultiplied, top-down RGBA8.
    fn extract_icon(hicon: HICON, source_key: isize, is_arrow: bool) -> Option<CursorShape> {
        let mut info = ICONINFO {
            fIcon: 0,
            xHotspot: 0,
            yHotspot: 0,
            hbmMask: std::ptr::null_mut(),
            hbmColor: std::ptr::null_mut(),
        };
        if unsafe { GetIconInfo(hicon, &mut info) } == 0 {
            return None;
        }

        let extracted = unsafe { extract_bitmaps(info.hbmColor, info.hbmMask) };

        unsafe {
            if !info.hbmColor.is_null() {
                DeleteObject(info.hbmColor as _);
            }
            if !info.hbmMask.is_null() {
                DeleteObject(info.hbmMask as _);
            }
        }

        let (width, height, pixels) = extracted?;
        Some(CursorShape {
            width,
            height,
            hotspot: (info.xHotspot, info.yHotspot),
            is_arrow,
            source_key,
            pixels,
        })
    }

    /// Reads the ICONINFO bitmaps into premultiplied top-down RGBA8.
    ///
    /// * Colour cursors: 32bpp DIB from `hbm_color`; alpha taken from the colour DIB when
    ///   present, otherwise punched through with the 1bpp AND mask from `hbm_mask`.
    /// * Monochrome cursors (`hbm_color == null`): `hbm_mask` stacks the AND half above the
    ///   XOR half; AND marks transparency, XOR bit 1 = white, 0 = black.
    unsafe fn extract_bitmaps(
        hbm_color: HBITMAP,
        hbm_mask: HBITMAP,
    ) -> Option<(u32, u32, Vec<u8>)> {
        unsafe {
            let hdc = GetDC(std::ptr::null_mut());
            if hdc.is_null() {
                return None;
            }
            let result = read_dib_pair(hdc, hbm_color, hbm_mask);
            ReleaseDC(std::ptr::null_mut(), hdc);
            result
        }
    }

    unsafe fn read_dib_pair(
        hdc: HDC,
        hbm_color: HBITMAP,
        hbm_mask: HBITMAP,
    ) -> Option<(u32, u32, Vec<u8>)> {
        unsafe {
            if !hbm_color.is_null() {
                read_color_cursor(hdc, hbm_color, hbm_mask)
            } else if !hbm_mask.is_null() {
                read_mono_cursor(hdc, hbm_mask)
            } else {
                None
            }
        }
    }

    fn bitmap_info(width: u32, height: u32, bit_count: u16) -> BITMAPINFO {
        BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width as i32,
                // Negative height → top-down rows.
                biHeight: -(height as i32),
                biPlanes: 1,
                biBitCount: bit_count,
                biCompression: BI_RGB,
                biSizeImage: 0,
                biXPelsPerMeter: 0,
                biYPelsPerMeter: 0,
                biClrUsed: 0,
                biClrImportant: 0,
            },
            bmiColors: [RGBQUAD {
                rgbBlue: 0,
                rgbGreen: 0,
                rgbRed: 0,
                rgbReserved: 0,
            }; 1],
        }
    }

    unsafe fn object_size(hbm: HBITMAP) -> Option<BITMAP> {
        unsafe {
            let mut bm: BITMAP = std::mem::zeroed();
            if GetObjectW(
                hbm as _,
                std::mem::size_of::<BITMAP>() as i32,
                &mut bm as *mut _ as _,
            ) == 0
            {
                None
            } else {
                Some(bm)
            }
        }
    }

    unsafe fn read_color_cursor(
        hdc: HDC,
        hbm_color: HBITMAP,
        hbm_mask: HBITMAP,
    ) -> Option<(u32, u32, Vec<u8>)> {
        unsafe {
            let bm = object_size(hbm_color)?;
            let width = bm.bmWidth.max(1) as u32;
            let height = bm.bmHeight.max(1) as u32;

            let mut color_bits = vec![0u32; (width * height) as usize];
            let mut bmi = bitmap_info(width, height, 32);
            if GetDIBits(
                hdc,
                hbm_color,
                0,
                height,
                color_bits.as_mut_ptr() as *mut _,
                &mut bmi,
                DIB_RGB_COLORS,
            ) == 0
            {
                return None;
            }

            // AND mask (1bpp, same dimensions) for cursors whose colour DIB carries no alpha.
            let and_bits = if !hbm_mask.is_null() {
                read_and_mask(hdc, hbm_mask, width, height)
            } else {
                None
            };
            let color_has_alpha = color_bits.iter().any(|&px| (px >> 24) & 0xFF != 0);

            let mut pixels = Vec::with_capacity((width * height * 4) as usize);
            for i in 0..(width * height) as usize {
                let px = color_bits[i];
                let mut a = ((px >> 24) & 0xFF) as u8;
                if !color_has_alpha {
                    a = if and_bits.as_ref().is_some_and(|m| m[i]) {
                        0
                    } else {
                        255
                    };
                }
                let b = (px & 0xFF) as u8;
                let g = ((px >> 8) & 0xFF) as u8;
                let r = ((px >> 16) & 0xFF) as u8;
                let af = a as u32;
                pixels.extend_from_slice(&[
                    ((r as u32 * af) / 255) as u8,
                    ((g as u32 * af) / 255) as u8,
                    ((b as u32 * af) / 255) as u8,
                    a,
                ]);
            }

            // Fully transparent extraction (an invisible cursor): reject so the caller keeps
            // its previous shape.
            if pixels.iter().skip(3).step_by(4).all(|&a| a == 0) {
                return None;
            }
            Some((width, height, pixels))
        }
    }

    /// Reads a 1bpp AND mask top-down; `true` = fully transparent pixel.
    unsafe fn read_and_mask(
        hdc: HDC,
        hbm_mask: HBITMAP,
        width: u32,
        height: u32,
    ) -> Option<Vec<bool>> {
        unsafe {
            let words_per_row = width.div_ceil(32) as usize;
            let mut bits = vec![0u32; words_per_row * height as usize];
            let mut bmi = bitmap_info(width, height, 1);
            if GetDIBits(
                hdc,
                hbm_mask,
                0,
                height,
                bits.as_mut_ptr() as *mut _,
                &mut bmi,
                DIB_RGB_COLORS,
            ) == 0
            {
                return None;
            }
            Some(
                (0..(width * height) as usize)
                    .map(|i| {
                        let row = i / width as usize;
                        let col = i % width as usize;
                        let word = bits[row * words_per_row + col / 32];
                        (word >> (31 - col % 32)) & 1 == 1
                    })
                    .collect(),
            )
        }
    }

    /// Monochrome cursor: `hbm_mask` is `height*2` rows — AND half first, XOR half second.
    unsafe fn read_mono_cursor(
        hdc: HDC,
        hbm_mask: HBITMAP,
    ) -> Option<(u32, u32, Vec<u8>)> {
        unsafe {
            let bm = object_size(hbm_mask)?;
            let width = bm.bmWidth.max(1) as u32;
            let height = (bm.bmHeight.max(2) / 2) as u32;
            let words_per_row = width.div_ceil(32) as usize;

            let mut packed = vec![0u32; words_per_row * (height * 2) as usize];
            let mut bmi = bitmap_info(width, height * 2, 1);
            if GetDIBits(
                hdc,
                hbm_mask,
                0,
                height * 2,
                packed.as_mut_ptr() as *mut _,
                &mut bmi,
                DIB_RGB_COLORS,
            ) == 0
            {
                return None;
            }

            let bit = |row: usize, col: usize| -> bool {
                let word = packed[row * words_per_row + col / 32];
                (word >> (31 - col % 32)) & 1 == 1
            };

            let mut pixels = Vec::with_capacity((width * height * 4) as usize);
            for row in 0..height as usize {
                for col in 0..width as usize {
                    if bit(row, col) {
                        // AND = 1 → transparent.
                        pixels.extend_from_slice(&[0, 0, 0, 0]);
                    } else {
                        // AND = 0 → XOR decides black/white.
                        let v = if bit(row + height as usize, col) {
                            0xFF
                        } else {
                            0x00
                        };
                        pixels.extend_from_slice(&[v, v, v, 255]);
                    }
                }
            }
            Some((width, height, pixels))
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn bitmap_info_requests_top_down_rows() {
            let bmi = bitmap_info(24, 32, 32);
            assert_eq!(bmi.bmiHeader.biWidth, 24);
            assert_eq!(bmi.bmiHeader.biHeight, -32);
            assert_eq!(bmi.bmiHeader.biBitCount, 32);
            assert_eq!(bmi.bmiHeader.biCompression, BI_RGB);
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn restore_is_safe_without_hiding() {
        // force_restore/set_arrow_hidden(false) must be callable from a clean state (exit
        // path). Only `false` is exercised: hiding would change the real system cursor.
        super::force_restore();
        super::set_arrow_hidden(false);
        super::set_arrow_hidden(false);
    }

    #[test]
    fn extraction_never_panics() {
        // Non-Windows: always Unavailable. Windows: any variant, depending on the session.
        let _ = super::extract_current();
        super::invalidate_cache();
        let _ = super::extract_current();
    }
}
