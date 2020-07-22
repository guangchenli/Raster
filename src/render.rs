use image::{Rgb, RgbImage};
use nalgebra::{Vector2, Vector3, Vector4, Matrix4};

fn baycentric2d(x : f32, y : f32, v : &Vec<Vector4<f32>>) -> Vector3<f32> {
    let c1 = ((v[1].y - v[2].y)*(x - v[2].x) + (v[2].x - v[1].x)*(y - v[2].y)) 
        / ((v[1].y - v[2].y) * (v[0].x - v[2].x) + (v[2].x - v[1].x) * (v[0].y - v[2].y));
    let c2 = ((v[2].y - v[0].y)*(x - v[2].x) + (v[0].x - v[2].x)*(y - v[2].y)) 
        / ((v[1].y - v[2].y) * (v[0].x - v[2].x) + (v[2].x - v[1].x) * (v[0].y - v[2].y));
    let c3 = 1. - c1 - c2; 
    Vector3::new(c1, c2, c3)
}

fn world_to_screen(v : &Vector3<f32> ,m : Matrix4<f32>) -> Vector4<f32> {
    let va = Vector4::new(v.x, v.y, v.z, 1.);
    let p = m * va;
    Vector4::new(p.x/p.w, p.y/p.w, p.z/p.w, 1./p.w)
}

fn idx_to_triangle(idx : (u32, u32, u32), p : &Vec<f32>) -> Vec<Vector3<f32>> {
    vec![Vector3::new(p[(idx.0 * 3) as usize], p[(idx.0 * 3 + 1) as usize], p[(idx.0 * 3 + 2) as usize]), 
    Vector3::new(p[(idx.1 * 3) as usize], p[(idx.1 * 3 + 1) as usize], p[(idx.1 * 3 + 2) as usize]),
    Vector3::new(p[(idx.2 * 3) as usize], p[(idx.2 * 3 + 1) as usize], p[(idx.2 * 3 + 2) as usize])]
}

fn idx_to_tex_triangle(idx : (u32, u32, u32), p : &Vec<f32>) -> Vec<Vector2<f32>> {
    vec![Vector2::new(p[(idx.0 * 2) as usize], p[(idx.0 * 2 + 1) as usize]), 
    Vector2::new(p[(idx.1 * 2) as usize], p[(idx.1 * 2 + 1) as usize]),
    Vector2::new(p[(idx.2 * 2) as usize], p[(idx.2 * 2 + 1) as usize])]
}

fn rasterize_triangle(vs : Vec<Vector4<f32>>, vts : Vec<Vector2<f32>>, z_buffer : &mut Vec<f32>, 
    img : &mut RgbImage, tex : &RgbImage) {

  let img_bound = Vector2::new(img.width() as f32, img.height() as f32);
  let mut bbmin = Vector2::new(img_bound[0] - 1., img_bound[1] - 1.);
  let mut bbmax = Vector2::new(0., 0.);

  // Generate bounding box
  for i in 0..3 {
      for j in 0..2 {
          if vs[i][j] > bbmax[j] {
              if vs[i][j] < img_bound[j] {
                  bbmax[j] = vs[i][j];
              } else {
                  bbmax[j] = img_bound[j];
              }
          }
          if vs[i][j] < bbmin[j] {
              if vs[i][j] > 0. {
                  bbmin[j] = vs[i][j];
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
          let bc = baycentric2d(x_proper, y_proper, &vs);
          let reci = bc.x * 1. * vs[0].w + bc.y * 1. * vs[1].w + bc.z * 1. * vs[2].w;
          // Not in triangle
          if bc[0] < 0. || bc[1] < 0. || bc[2] < 0. || x * y > img.width() * img.height() {
              continue;
          }
          // In triangle
          // Get interpolated z value
          let z_interpolated = (bc.x * vs[0].z * vs[0].w + bc.y * vs[1].z * vs[1].w + bc.z * vs[2].z * vs[2].w) / reci;
          if z_interpolated > 1. || z_interpolated < -1. {continue};
          let z_buffer_idx = (x + y * img.width()) as usize;
          let u_s = bc.x * vts[0].x * vs[0].w + bc.y * vts[1].x * vs[1].w + bc.z * vts[2].x * vs[2].w;
          let v_s = bc.x * vts[0].y * vs[0].w + bc.y * vts[1].y * vs[1].w + bc.z * vts[2].y * vs[2].w;
          let u = u_s / reci;
          let v = v_s / reci;
          let tx = (u * tex.width() as f32) as u32;
          let ty = tex.height() - ((v * tex.height() as f32) as u32);
          let color = tex.get_pixel(tx, ty);

          if z_buffer[z_buffer_idx]  < z_interpolated {
              z_buffer[z_buffer_idx] = z_interpolated;
              // flip y value here
              img.put_pixel(x as u32, img.height() - y - 1 as u32, Rgb([color[0],color[1],color[2]]));
          }
      }
  } 
}

pub fn rasterize(models : Vec<tobj::Model>, img : &mut RgbImage, tex : &RgbImage,
      z_buf : &mut Vec<f32>, m : Matrix4<f32>) {

    for model in models {
        let indices = model.mesh.indices;
        let positions = model.mesh.positions;
        let texcoords = model.mesh.texcoords;
        for i in 0..(indices.len() / 3) {
            let vs_idx = (indices[i*3], indices[i*3+1], indices[i*3+2]);
            let triangle = idx_to_triangle(vs_idx, &positions);
            let mut triangle_screen = Vec::new();
            let tex_tirangle = idx_to_tex_triangle(vs_idx, &texcoords);
            for i in 0..triangle.len() {
                triangle_screen.push(world_to_screen(&triangle[i], m));
            }

            rasterize_triangle(triangle_screen, tex_tirangle, z_buf, img, tex);
        }
    }
}