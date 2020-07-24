mod render;
mod transforms;
mod shader;

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
    let diffuse_path = &args[2];

    let width = 1000;
    let height = 1000;

    // eye, gaze, top
    let e = Vector3::new(0., 0., 0.);
    let g = Vector3::new(0., 0., -1.);
    let t = Vector3::new(0., 1., 0.);

    let mut img : RgbImage = ImageBuffer::new(width, height);
    let obj = tobj::load_obj(obj_path, true);
    assert!(obj.is_ok());
    let diffuse = image::open(diffuse_path);
    assert!(diffuse.is_ok());
    let mut obj = obj.unwrap().0;
    let diffuse = diffuse.unwrap().to_rgb();

    let mut z_buf = vec![f32::MIN;(width * height) as usize];

    let m_vp = transforms::viewport(width, height);
    let m_per = transforms::perspective(-1., 1., -1., 1., -3., -5.);
    let model = Matrix4::new(1., 0., 0., 0.,
                0., 1., 0., 0.,
                0., 0., 1., -4.,
                0., 0., 0., 1.);
    let m_cam = transforms::camera(e, g, t);
    let m = m_vp * m_per * m_cam * model;

    let mesh = obj.remove(0).mesh;
    let len = mesh.indices.len() / 3;
    let id = mesh.indices;
    let pos = mesh.positions;
    let texcoords = mesh.texcoords;

    let s : Box<dyn shader::Shader> = Box::new(shader::VanillaShader {
        m : m,
        indices : id,
        positions : pos,
        texcoords : texcoords,
        diffuse_width : diffuse.width(),
        diffuse_height : diffuse.height(),
        diffuse : diffuse,
    });

    render::rasterize(len, &s, &mut z_buf, &mut img);

    img.save("test.png").unwrap();
}