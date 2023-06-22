use std::ptr::null_mut;

use winapi::um::wingdi::{
    SelectObject,
    DeleteObject,
    CreateSolidBrush,
    CreatePen,
    MoveToEx,
    LineTo, CreateCompatibleDC, CreateCompatibleBitmap, BitBlt, SRCCOPY, DeleteDC, Ellipse,
};

use winapi::um::wingdi::PS_SOLID;

use winapi::um::winuser::{
    PAINTSTRUCT,
    FillRect,
};

use winapi::shared::windef::{
    HDC,
    COLORREF, HBITMAP, RECT, HBRUSH, HPEN, HGDIOBJ,
};

pub struct DrawFrameData {
    pub hdc: HDC,
    h_bmp_mem: HBITMAP,
    h_old_bmp_mem: HBITMAP,
}

pub struct SolidPenData {
    hdc: HDC,
    pen: HPEN,
    old_pen: HGDIOBJ,
}

/// Return (brush, old_brush)
pub fn change_solid_brush(hdc: HDC, color: u32) -> (HBRUSH, HBRUSH) {
    let brush: HBRUSH = unsafe { CreateSolidBrush(color) };
    let old_brush = unsafe { SelectObject(hdc, brush as _) } as HBRUSH;
    (brush, old_brush)
}

pub fn revert_brush(hdc: HDC, brush: HBRUSH, old_brush: HBRUSH) {
    unsafe {
        SelectObject(hdc, old_brush as _);
        DeleteObject(brush as _);
    }
}

pub fn open_draw_frame(hdc: HDC, width: i32, height: i32) -> DrawFrameData {
    unsafe {
        let h_mem_dc = CreateCompatibleDC(hdc);
        let h_bmp_mem = CreateCompatibleBitmap(hdc, width, height);
        let h_old_bmp_mem = SelectObject(h_mem_dc, h_bmp_mem as _) as HBITMAP;

        DrawFrameData { hdc: h_mem_dc, h_bmp_mem, h_old_bmp_mem }
    }
}

pub fn close_draw_frame(hdc: HDC, width: i32, height: i32, draw_frame_data: DrawFrameData) {
    unsafe {
        BitBlt(hdc, 0, 0, width, height, draw_frame_data.hdc, 0, 0, SRCCOPY);

        SelectObject(draw_frame_data.hdc, draw_frame_data.h_old_bmp_mem as _);
        DeleteObject(draw_frame_data.h_bmp_mem as _);
        DeleteDC(draw_frame_data.hdc);
    }
}

pub fn create_solid_pen(hdc: HDC, color: COLORREF) -> SolidPenData {
    let pen = unsafe { CreatePen(PS_SOLID as i32, 2, color) };
    let old_pen = unsafe { SelectObject(hdc, pen as _) };
    SolidPenData {
        hdc,
        pen,
        old_pen,
    }
}

pub fn draw_line(hdc: HDC, from: (i32, i32), to: (i32, i32)) {
    unsafe { MoveToEx(hdc, from.0, from.1, null_mut()) };
    unsafe { LineTo(hdc, to.0, to.1) };
}

pub fn close_draw_lines(data: SolidPenData) {
    unsafe { SelectObject(data.hdc, data.old_pen) };
    unsafe { DeleteObject(data.pen as _) };
}

/// Use current selected brush
pub fn draw_circle(hdc: HDC, x: i32, y: i32, radius: i32) {
    let left = x - radius;
    let top = y - radius;
    let right = x + radius;
    let bottom = y + radius;

    let rect = RECT {
        left,
        top,
        right,
        bottom,
    };

    unsafe {
        Ellipse(hdc, rect.left, rect.top, rect.right, rect.bottom);
    }
}

pub fn draw_fullscreen_rect(hdc: HDC, ps: &PAINTSTRUCT, color: COLORREF) {
    let rect = &ps.rcPaint;
    unsafe {
        let brush = CreateSolidBrush(color);
        FillRect(hdc, rect, brush);
        DeleteObject(brush as _);
    }
}

pub fn draw_spiral(hdc: HDC) {
    let mut angle = 0.0f32;
    let radius_mul = 10.0f32;
    let start_x : f32 = 1920.0 / 2.0;
    let start_y : f32 = 1080.0 / 2.0;

    let white_color = 0xFFFFFF;

    let brush: HBRUSH = unsafe { CreateSolidBrush(white_color) };
    let old_brush = unsafe { SelectObject(hdc, brush as _) };

    for i in 0..1000 {
        // Compute radius based on angle
        let radius = angle.powf(0.8);

        // Convert polar coordinates to Cartesian coordinates
        let x = start_x + radius * angle.cos() * radius_mul;
        let y = start_y + radius * angle.sin() * radius_mul;
        draw_circle(hdc, x as i32, y as i32, 3);

        // Increment the angle for the next iteration
        let c = ((i / 500) as f32).powf(0.4) + 1f32;
        let p = 0.05 / c;
        angle += p;
    }

    unsafe {
        SelectObject(hdc, old_brush);
        winapi::um::wingdi::DeleteObject(brush as _);
    }

    todo!("Add custom parameters!");
}