extern crate image;
extern crate tobj;

mod render;
mod transforms;

use image::{ImageBuffer, RgbImage};
use nalgebra::{Vector3, Matrix4};
use std::env;

fn main() {
    let args : Vec<String> = env::args().collect();
    let obj_path = &args[1];
    let tex_path = &args[2];

    let width = 800;
    let height = 800;

    let e = Vector3::new(0., 0., 0.);
    let g = Vector3::new(0., 0., -1.);
    let t = Vector3::new(0., 1., 0.);

    let mut img : RgbImage = ImageBuffer::new(width, height);
    let obj = tobj::load_obj(obj_path, true);
    assert!(obj.is_ok());
    let texture = image::open(tex_path);
    assert!(texture.is_ok());
    let (models, _) = obj.unwrap();

    let mut z_buf = vec![f32::MIN;(width * height) as usize];

    let m_vp = transforms::viewport(width, height);
    let m_per = transforms::perspective(-1., 1., -1., 1., -3., -4.);
    let t1 = Matrix4::new(1., 0., 0., 0.,
                0., 1., 0., 0.,
                0., 0., 1., -3.,
                0., 0., 0., 1.);
    let t2 = Matrix4::new(1., 0., 0., 0.,
                0., 1., 0., 0.,
                0., 0., 1., 3.,
                0., 0., 0., 1.);
    //let m_ortho = transforms::orthographic(-1., 1., -1., 1., 1., -1.);
    //let m_per_to_ortho = transforms::persp_to_ortho(3.);
    let m_cam = transforms::camera(e, g, t);
    let m = m_vp * t2 * m_per * t1 * m_cam;

    render::rasterize(models, &mut img, &texture.unwrap().to_rgb(), &mut z_buf, m);

    let mut z_max = f32::MIN;
    let mut z_min = f32::MAX;
    for i in z_buf {
        if i > z_max {z_max = i}
        if i < z_min && i != f32::MIN {z_min = i}
    }

    //println!("z max {} z min {}", z_max, z_min);

    img.save("test.png").unwrap();
}