use image::{Rgb, RgbImage};
use nalgebra::{Vector2, Vector3, Vector4, Matrix4};

fn baycentric2d(x : f32, y : f32, v : &Vec<Vector3<f32>>) -> Vector3<f32> {
    let c1 = (x*(v[1].y - v[2].y) + (v[2].x - v[1].x)*y + v[1].x*v[2].y - v[2].x*v[1].y) / (v[0].x*(v[1].y - v[2].y) + (v[2].x - v[1].x)*v[0].y + v[1].x*v[2].y - v[2].x*v[1].y);
    let c2 = (x*(v[2].y - v[0].y) + (v[0].x - v[2].x)*y + v[2].x*v[0].y - v[0].x*v[2].y) / (v[1].x*(v[2].y - v[0].y) + (v[0].x - v[2].x)*v[1].y + v[2].x*v[0].y - v[0].x*v[2].y);
    let c3 = (x*(v[0].y - v[1].y) + (v[1].x - v[0].x)*y + v[0].x*v[1].y - v[1].x*v[0].y) / (v[2].x*(v[0].y - v[1].y) + (v[1].x - v[0].x)*v[2].y + v[0].x*v[1].y - v[1].x*v[0].y);
    Vector3::new(c1, c2, c3)
}

fn in_triangle(x : f32, y : f32, vs : &Vec<Vector3<f32>>) -> bool {
    let mut c = [0., 0., 0.];
    for i in 0..3 {
        let v1 = Vector2::new(vs[(i+1)%3].x - vs[i].x, vs[(i+1)%3].y - vs[i].y);
        let v2 = Vector2::new(x - vs[i].x, y - vs[i].y);
        // Calculate cross product of v1 and v2
        c[i] = v1[0] * v2[1] - v2[0] * v1[1];
    }
    (c[0] < 0. && c[1] < 0. && c[2] < 0.) || (c[0] > 0. && c[1] > 0. && c[2] > 0.)
}

fn rasterize_triangle(vs : Vec<Vector3<f32>>, vts : Vec<Vector2<f32>>, z_buffer : &mut Vec<f32>, 
      img : &mut RgbImage, tex : &RgbImage) {
    let mut bbmin = Vector2::new(f32::MAX, f32::MAX);
    let mut bbmax = Vector2::new(f32::MIN, f32::MIN);
    let img_bound = Vector2::new(img.width() as f32, img.height() as f32);

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
                    bbmin[j] = img_bound[j];
                }
            }
        }
    }

    //println!("minx {} maxx {} miny {} maxy {}", bbmin[0], bbmin[1], bbmax[0], bbmax[1]);

    if bbmin[0] < 0. {bbmin[0] = 0.}
    if bbmin[1] < 0. {bbmin[1] = 0.}
    if bbmax[0] < 0. {bbmax[0] = 0.}
    if bbmax[1] < 0. {bbmax[1] = 0.}
    if bbmax[0] < bbmin[0] || bbmax[1] < bbmin[1] {return}

    // Rasterize triangle pixels inside bounding box
    for x in f32::floor(bbmin[0]) as u32..f32::ceil(bbmax[0]) as u32{
        for y in f32::floor(bbmin[1]) as u32..f32::ceil(bbmax[1]) as u32 {
            let x_proper = x as f32 + 0.5;
            let y_proper = y as f32 + 0.5;
            let bc = baycentric2d(x_proper, y_proper, &vs);
            // Not in triangle
            if !in_triangle(x_proper, y_proper, &vs) || x * y > img.width() * img.height() {
                continue;
            }
            // In triangle
            // Get interpolated z value
            let w_reciprocal =  1.0/(bc.x + bc.y + bc.z);
            let z_interpolated = (bc.x * vs[0].z + bc.y * vs[1].z + bc.z * vs[2].z) * w_reciprocal;
            if z_interpolated < 0. {continue};
            //let z_interpolated = w_reciprocal / (bc.x * vs[0].z + bc.y * vs[1].z + bc.z * vs[2].z);
            let z_buffer_idx = (x + y * img.width()) as usize;
            // TODO: Make it perspective correct
            // TODO: Fix points at the back of camera
            // println!("{} {} {}", vs[0].z, vs[1].z, vs[2].z);
            let tc = (vts[0] * bc.x + vts[1] * bc.y + vts[2] * bc.z) * w_reciprocal;
            //let tc = (vts[0] * bc.x / vs[0].z + vts[1] * bc.y / vs[1].z + vts[2] * bc.z / vs[2].z) * w_reciprocal * z_interpolated;
            //let tx = (tc.x * img.width() as f32) as u32;
            let tx = (tc.x * tex.width() as f32) as u32;
            let ty = tex.height() - ((tc.y * tex.height() as f32) as u32) - 1;
            //println!("{}", tc);
            //println!("{} {}", tx, ty);
            let color = tex.get_pixel(tx, ty);

            if z_buffer[z_buffer_idx]  < z_interpolated {
                z_buffer[z_buffer_idx] = z_interpolated;
                //let zc = ((z_interpolated + 1.) * 64.) as u8;
                // flip y value here
                img.put_pixel(x as u32, img.height() - y - 1 as u32, image::Rgb([color[0],color[1],color[2]]));
            }
        }
    } 
}

fn world_to_screen(v : &Vector3<f32> ,m : Matrix4<f32>) -> Vector3<f32> {
    let va = Vector4::new(v.x, v.y, v.z, 1.);
    let p = m * va;
    Vector3::new(p.x/p.w, p.y/p.w, p.z/p.w)
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

pub fn rasterize(models : Vec<tobj::Model>, img : &mut RgbImage, tex : &RgbImage,
      z_buf : &mut Vec<f32>, m : Matrix4<f32>) {

    for model in models {
        let indices = model.mesh.indices;
        let positions = model.mesh.positions;
        let texcoords = model.mesh.texcoords;
        for i in 0..(indices.len() / 3) {
            let vs_idx = (indices[i*3], indices[i*3+1], indices[i*3+2]);
            let mut triangle = idx_to_triangle(vs_idx, &positions);
            let tex_tirangle = idx_to_tex_triangle(vs_idx, &texcoords);
            for i in 0..triangle.len() {
                triangle[i] = world_to_screen(&triangle[i], m);
            }
            // let color = Rgb([rng.gen::<u8>(),rng.gen::<u8>(),rng.gen::<u8>()]);
            rasterize_triangle(triangle, tex_tirangle, z_buf, img, tex);
        }
    }
}