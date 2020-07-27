use image::{RgbImage};
use nalgebra::{Vector2, Vector4};
use super::shader::{Shader, VertexAttr};

fn baycentric2d(x : f32, y : f32, v : (Vector4<f32>, Vector4<f32>, Vector4<f32>)) -> (f32, f32, f32) {
    let c1 = ((v.1.y - v.2.y)*(x - v.2.x) + (v.2.x - v.1.x)*(y - v.2.y)) 
        / ((v.1.y - v.2.y) * (v.0.x - v.2.x) + (v.2.x - v.1.x) * (v.0.y - v.2.y));
    let c2 = ((v.2.y - v.0.y)*(x - v.2.x) + (v.0.x - v.2.x)*(y - v.2.y)) 
        / ((v.1.y - v.2.y) * (v.0.x - v.2.x) + (v.2.x - v.1.x) * (v.0.y - v.2.y));
    let c3 = 1. - c1 - c2; 
    (c1, c2, c3)
}

fn rasterize_triangle<'a>(vs : [(Vector4<f32>, Vec<VertexAttr>); 3], shader : &Box<dyn Shader + 'a>, z_buffer : &mut Vec<f32>, img : &mut RgbImage) {

    let img_bound = Vector2::new(img.width() as f32, img.height() as f32);
    let mut bbmin = Vector2::new(img_bound[0] - 1., img_bound[1] - 1.);
    let mut bbmax = Vector2::new(0., 0.);

    // Generate bounding box
    for i in 0..3 {
         for j in 0..2 {
            if vs[i].0[j] > bbmax[j] {
                if vs[i].0[j] < img_bound[j] {
                    bbmax[j] = vs[i].0[j];
                 } else {
                     bbmax[j] = img_bound[j];
                 }
            }
            if vs[i].0[j] < bbmin[j] {
                if vs[i].0[j] > 0. {
                    bbmin[j] = vs[i].0[j];
                } else {
                     bbmin[j] = 0.;
                }
            }
        }
    }

    if bbmin[0] < 0. {bbmin[0] = 0.}
    if bbmin[1] < 0. {bbmin[1] = 0.}
    if bbmax[0] < 0. {bbmax[0] = 0.}
    if bbmax[1] < 0. {bbmax[1] = 0.}
    if bbmax[0] <= bbmin[0] || bbmax[1] <= bbmin[1] {return}

    // Rasterize triangle pixels inside bounding box
    for x in f32::floor(bbmin[0]) as u32..f32::ceil(bbmax[0]) as u32{
         for y in f32::floor(bbmin[1]) as u32..f32::ceil(bbmax[1]) as u32 {
            let x_proper = x as f32 + 0.5;
            let y_proper = y as f32 + 0.5;
            let bc = baycentric2d(x_proper, y_proper, (vs[0].0, vs[1].0, vs[2].0));

            // Not in triangle
            if bc.0 < 0. || bc.1 < 0. || bc.2 < 0. || x * y > img.width() * img.height() {
                continue;
            }
            // In triangle
            // Get interpolated z value
            let w_reci = bc.0 * vs[0].0.w + bc.1 * vs[1].0.w + bc.2 * vs[2].0.w;
            let z_interpolated = (bc.0 * vs[0].0.z * vs[0].0.w + bc.1 * vs[1].0.z * vs[1].0.w + bc.2 * vs[2].0.z * vs[2].0.w) / w_reci;
            if z_interpolated > 1. || z_interpolated < -1. {continue};
            let z_buffer_idx = (x + y * img.width()) as usize;
            
            let (color, drop) = (*shader).fragment(bc, (vs[0].0.w, vs[1].0.w, vs[2].0.w), (&vs[0].1, &vs[1].1, &vs[2].1));

            if z_buffer[z_buffer_idx]  < z_interpolated && !drop {
                z_buffer[z_buffer_idx] = z_interpolated;
                // flip y value here
                img.put_pixel(x as u32, img.height() - y - 1 as u32, color);
            }
        }
    } 
}

pub fn rasterize<'a>(len : usize, shader : &Box<dyn Shader + 'a>, z_buf : &mut Vec<f32>, img : &mut RgbImage) {
    for i in 0..len {
        let v0 = (*shader).vertex(i as u32, 0);
        let v1 = (*shader).vertex(i as u32, 1);
        let v2 = (*shader).vertex(i as u32, 2);
        let t = [v0,v1,v2];
        rasterize_triangle(t, &shader, z_buf, img);
    }
}