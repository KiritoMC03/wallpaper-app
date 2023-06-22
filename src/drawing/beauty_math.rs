use std::f64::consts::PI;
use winapi::shared::windef::HDC;

use super::primitives::{draw_line, create_solid_pen, close_draw_lines};

static mut ORIG_X: f64 = 0.0;
static mut ORIG_Y: f64 = 0.0;

//---------------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Default)]
pub struct Galaxy {
    pub x: f64,
    pub y: f64,
    pub color : u32,
    pub diameter: f64,
    pub max_diameter: f64,
    pub curvature: i32,
    pub theta: f64,
    pub theta_step: f64,
    pub is_max_radius: bool,
    pub hptr_x: f64, // hypotrochoid x anchor (see: http://en.wikipedia.org/wiki/Hypotrochoid)
    pub hptr_y: f64, // hypotrochoid y anchor
}

impl Galaxy {
    pub fn new(mouse_x: f64, mouse_y: f64, screen_w: usize, screen_h: usize, color: u32) -> Galaxy {
        Galaxy {
            x: mouse_x,
            y: mouse_y,
            color,
            diameter: 9.0,
            max_diameter: 450.0,
            curvature: 10,
            theta: 0.0,
            theta_step: 360.0 * PI / 180.0,
            is_max_radius: false,
            hptr_x: (mouse_x / (screen_w * 999 >> 0) as f64) / 999.0,
            hptr_y: (mouse_y / (screen_h * 999 >> 0) as f64) / 999.0,
        }
    }

    pub const fn empty() -> Galaxy {
        Galaxy {
            x: 0.0,
            y: 0.0,
            color: 0,
            diameter: 0.0,
            max_diameter: 0.0,
            curvature: 0,
            theta: 0.0,
            theta_step: 0.0,
            is_max_radius: false,
            hptr_x: 0.0,
            hptr_y: 0.0,
        }
    }
}

/// Not support multithread now
pub fn draw_galaxy_step_inc(hdc: HDC, galaxy: &mut Galaxy) {
    let mut prev_x = 0.0;
    let mut prev_y = 0.0;
    let draw_lines_data = create_solid_pen(hdc, galaxy.color);
    for curv_step in (0..galaxy.curvature).rev() {
        if galaxy.diameter > galaxy.max_diameter || galaxy.is_max_radius {
            if !galaxy.is_max_radius {
                galaxy.is_max_radius = true;
            }
            if galaxy.diameter < 0.1 {
                galaxy.is_max_radius = false;
            }
            galaxy.theta -= galaxy.theta_step;
            galaxy.diameter -= 0.1;
        }

        if !galaxy.is_max_radius {
            galaxy.theta += galaxy.theta_step;
            galaxy.diameter += 0.1;
        }

        let hx = galaxy.hptr_x;
        let hy = galaxy.hptr_y;
        let q = (hx / hy - 1.0) * galaxy.theta; // create hypotrochoid

        unsafe{
            let curvature = curv_step as f64 / galaxy.curvature as f64;
            let h_delta = hx - hy;
            let cur_x = h_delta * galaxy.theta.cos() + galaxy.diameter * q.cos() + (ORIG_X + (galaxy.x - ORIG_X) * curvature) - h_delta;
            let cur_y = h_delta * galaxy.theta.sin() - galaxy.diameter * q.sin() + (ORIG_Y + (galaxy.y - ORIG_Y) * curvature);

            if prev_x != 0.0 {
                draw_line(hdc, (prev_x as i32, prev_y as i32), (cur_x as i32, cur_y as i32));
            }

            prev_x = cur_x;
            prev_y = cur_y;
        }
    }
    close_draw_lines(draw_lines_data);
    unsafe {
        ORIG_X = galaxy.x;
        ORIG_Y = galaxy.y;
    };
}

/// Not support multithread now
//pub fn draw_galaxy_step(hdc: HDC, galaxy: &mut Galaxy) {
//    let mut prev_x = 0.0;
//    let mut prev_y = 0.0;
//    for curv_step in (0..galaxy.curvature).rev() {
//        if galaxy.diameter > galaxy.max_diameter || galaxy.is_max_radius {
//            if !galaxy.is_max_radius {
//                galaxy.is_max_radius = true;
//            }
//            if galaxy.diameter < 0.1 {
//                galaxy.is_max_radius = false;
//            }
//            galaxy.theta -= galaxy.theta_step;
//            galaxy.diameter -= 0.1;
//        }
//
//        if !galaxy.is_max_radius {
//            galaxy.theta += galaxy.theta_step;
//            galaxy.diameter += 0.1;
//        }
//
//        let hx = galaxy.hptr_x;
//        let hy = galaxy.hptr_y;
//        let q = (hx / hy - 1.0) * galaxy.theta; // create hypotrochoid
//
//        unsafe{
//            let curvature = curv_step as f64 / galaxy.curvature as f64;
//            let h_delta = hx - hy;
//            let cur_x = h_delta * galaxy.theta.cos() + galaxy.diameter * q.cos() + (ORIG_X + (galaxy.x - ORIG_X) * curvature) - h_delta;
//            let cur_y = h_delta * galaxy.theta.sin() - galaxy.diameter * q.sin() + (ORIG_Y + (galaxy.y - ORIG_Y) * curvature);
//
//            if prev_x != 0.0 {
//                draw_line(hdc, (prev_x as i32, prev_y as i32), (cur_x as i32, cur_y as i32), galaxy.color);
//            }
//
//            prev_x = cur_x;
//            prev_y = cur_y;
//        }
//    }
//    unsafe {
//        ORIG_X = galaxy.x;
//        ORIG_Y = galaxy.y;
//    };
//}

//---------------------------------------------------------------------------------------------------------------------------

/// `pixels` must be initialized with <b>width * height</b> size
pub fn calc_mandelbrot(width: usize, height: usize, max_iter: u32, pixels: &mut Vec<u32>) {
    for y in 0..height {
        for x in 0..width {
            let cx = (x as f64 - width as f64 / 2.0) * 4.0 / width as f64;
            let cy = (y as f64 - height as f64 / 2.0) * 4.0 / height as f64;

            let color_value = mandelbrot(cx, cy, max_iter) % 256;
            let color = (color_value << 16) | (color_value << 8) | color_value;

            pixels[y * width + x] = color;
        }
    }
}

#[inline(always)]
fn mandelbrot(cx: f64, cy: f64, max_iter: u32) -> u32 {
    let mut x = 0.0;
    let mut y = 0.0;
    let mut i = 0;

    while x * x + y * y <= 4.0 && i < max_iter {
        let x_temp = x * x - y * y + cx;
        y = 2.0 * x * y + cy;
        x = x_temp;
        i += 1;
    }

    i
}