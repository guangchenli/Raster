mod render;
mod transforms;
mod shader;

use image::{ImageBuffer, RgbImage};
use nalgebra::{Vector3, Matrix4, Matrix3};
use std::env;

fn main() {
    let args : Vec<String> = env::args().collect();
    if env::args().len() != 4 {
        println!("Expecting 3 arguments, exiting...");
        return
    }
    let obj_path = &args[1];
    let diffuse_path = &args[2];
    let spec_path = &args[3];

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
    let spec = image::open(spec_path);
    assert!(spec.is_ok());
    let (obj, _) = obj.unwrap();
    let diffuse = diffuse.unwrap().to_rgb();
    let spec = spec.unwrap().to_rgb();

    let mut z_buf = vec![f32::MIN;(width * height) as usize];

    let m_vp = transforms::viewport(width, height);
    let m_per = transforms::perspective(-1., 1., -1., 1., -3., -5.);
    let model_affine = Matrix4::new(1., 0., 0., 0.,
                0., 1., 0., 0.,
                0., 0., 1., -4.,
                0., 0., 0., 1.);
    let model = Matrix3::new(1., 0., 0.,
                0., 1., 0.,
                0., 0., 1.);
    let m_cam = transforms::camera(e, g, t);
    let m = m_vp * m_per * m_cam * model_affine;

    let len = obj[0].mesh.indices.len() / 3;
    let mesh = &obj[0].mesh;
    let id = &mesh.indices;
    let pos = &mesh.positions;
    let texcoords = &mesh.texcoords;
    let normals = &mesh.normals;
    let light_source = vec!((Vector3::new(0., 0., -1.), 0.8));

    // let s : Box<dyn shader::Shader> = Box::new(shader::VanillaShader {
    //     m : m,
    //     indices : id,
    //     positions : pos,
    //     texcoords : texcoords,
    //     diffuse_width : diffuse.width(),
    //     diffuse_height : diffuse.height(),
    //     diffuse : diffuse,
    // });
    
    // let s_l = shader::GouraudShader {
    //     mvp : m,
    //     model : model,
    //     indices : id,
    //     positions : pos,
    //     texcoords : texcoords,
    //     diffuse : &diffuse,
    //     normals : normals,
    //     ambient : 0.2,
    //     light_source : &light_source
    // };

    let s_l = shader::BlinnPhongShader {
        mvp : m,
        model : model,
        indices : id,
        positions : pos,
        texcoords : texcoords,
        diffuse : &diffuse,
        spec : &spec,
        normals : normals,
        ambient : 0.2,
        light_source : &light_source,
        phong_exp : 1.
    };

    let b : Box<dyn shader::Shader> = Box::new(s_l);

    render::rasterize(len, &b, &mut z_buf, &mut img);
    img.save("out.png").unwrap();
}