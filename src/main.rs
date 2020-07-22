mod render;
mod transforms;

use image::{ImageBuffer, RgbImage};
use nalgebra::{Vector3, Matrix4};
use std::env;

fn main() {
    let args : Vec<String> = env::args().collect();
    if env::args().len() != 3 {
        println!("Expecting 2 arguments, exiting...");
        return
    }
    let obj_path = &args[1];
    let tex_path = &args[2];

    let width = 1000;
    let height = 1000;

    // eye, gaze, top
    let e = Vector3::new(0., 0., 0.);
    let g = Vector3::new(0., 0., -1.);
    let t = Vector3::new(0., 1., 0.);

    let mut img : RgbImage = ImageBuffer::new(width, height);
    let obj = tobj::load_obj(obj_path, true);
    assert!(obj.is_ok());
    let texture = image::open(tex_path);
    assert!(texture.is_ok());
    let (obj, _) = obj.unwrap();
    let texture = texture.unwrap().to_rgb();

    let mut z_buf = vec![f32::MIN;(width * height) as usize];

    let m_vp = transforms::viewport(width, height);
    let m_per = transforms::perspective(-1., 1., -1., 1., -3., -5.);
    let model = Matrix4::new(1., 0., 0., -0.9,
                0., 1., 0., 0.,
                0., 0., 1., -4.,
                0., 0., 0., 1.);
    let m_cam = transforms::camera(e, g, t);

    let m = m_vp * m_per * m_cam * model;

    render::rasterize(obj, &mut img, &texture, &mut z_buf, m);

    let mut z_max = f32::MIN;
    let mut z_min = f32::MAX;
    for i in z_buf {
        if i > z_max {z_max = i}
        if i < z_min && i != f32::MIN {z_min = i}
    }

    img.save("test.png").unwrap();
}