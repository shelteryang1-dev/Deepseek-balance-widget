use anyhow::Result;
use windows::Win32::Foundation::{COLORREF, RECT, SIZE};
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::UI::WindowsAndMessaging::*;

pub struct RenderedIcon {
    pub rgba: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

pub fn render_balance_icon(text: &str) -> Result<RenderedIcon> {
    unsafe { gdi_text_icon(text.trim()) }
}

pub fn render_status_icon(status: &str) -> Result<RenderedIcon> {
    unsafe { gdi_text_icon(status) }
}

unsafe fn gdi_text_icon(text: &str) -> Result<RenderedIcon> {
    let icon_w = GetSystemMetrics(SM_CXSMICON).max(16) as u32;
    let icon_h = GetSystemMetrics(SM_CYSMICON).max(16) as u32;
    let font_h = icon_h as i32;

    let hdc_screen = GetDC(None);
    if hdc_screen.is_invalid() { anyhow::bail!("GetDC"); }
    let hdc_mem = CreateCompatibleDC(hdc_screen);
    if hdc_mem.is_invalid() { ReleaseDC(None, hdc_screen); anyhow::bail!("CreateCompatibleDC"); }

    let hbmp = CreateCompatibleBitmap(hdc_screen, icon_w as i32, icon_h as i32);
    if hbmp.is_invalid() {
        let _ = DeleteDC(hdc_mem);
        ReleaseDC(None, hdc_screen);
        anyhow::bail!("CreateCompatibleBitmap");
    }
    let hbmp_old = SelectObject(hdc_mem, hbmp);

    // Black background → transparent
    let hbr = CreateSolidBrush(COLORREF(0));
    let full = RECT { left: 0, top: 0, right: icon_w as i32, bottom: icon_h as i32 };
    FillRect(hdc_mem, &full, hbr);
    let _ = DeleteObject(hbr);

    let hfont = CreateFontW(
        font_h, 0, 0, 0, 700, 0, 0, 0, 1, 0, 0, 5, 0,
        windows::core::w!("Segoe UI"),
    );
    let hfont_old = SelectObject(hdc_mem, hfont);
    SetBkMode(hdc_mem, TRANSPARENT);
    SetTextColor(hdc_mem, COLORREF(0xFFFFFF));

    // Measure text width and font ascent for precise centering.
    let wide: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
    let mut sz = SIZE::default();
    GetTextExtentPoint32W(hdc_mem, &wide, &mut sz);
    let mut tm = TEXTMETRICA::default();
    GetTextMetricsA(hdc_mem, &mut tm);

    // X: horizontal center
    let x = ((icon_w as i32 - sz.cx) / 2).max(0);
    // Y: TextOutW uses baseline, so offset by ascent to center vertically.
    let y = ((icon_h as i32 - sz.cy) / 2).max(0) + tm.tmAscent;
    TextOutW(hdc_mem, x, y, &wide);

    SelectObject(hdc_mem, hfont_old);
    let _ = DeleteObject(hfont);

    // Read BGRA pixels from the bitmap
    let size = (icon_w * icon_h) as usize;
    let mut bits = vec![0u32; size];
    let mut bmi = BITMAPINFO::default();
    bmi.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
    bmi.bmiHeader.biWidth = icon_w as i32;
    bmi.bmiHeader.biHeight = -(icon_h as i32);
    bmi.bmiHeader.biPlanes = 1;
    bmi.bmiHeader.biBitCount = 32;
    GetDIBits(
        hdc_mem, hbmp, 0, icon_h,
        Some(bits.as_mut_ptr() as *mut _),
        &mut bmi as *mut _,
        DIB_RGB_COLORS,
    );

    SelectObject(hdc_mem, hbmp_old);
    let _ = DeleteObject(hbmp);
    let _ = DeleteDC(hdc_mem);
    ReleaseDC(None, hdc_screen);

    // Black → transparent; white text → white with brightness as alpha
    let mut rgba = Vec::with_capacity(size * 4);
    for p in &bits {
        let r = ((p >> 16) & 0xFF) as u32;
        let g = ((p >> 8) & 0xFF) as u32;
        let b = (p & 0xFF) as u32;
        let a = ((r + g + b) / 3) as u8;
        rgba.extend_from_slice(&[255, 255, 255, a]);
    }

    Ok(RenderedIcon { rgba, width: icon_w, height: icon_h })
}
