use image::{Rgb, RgbImage};

pub fn draw_line(x0 : u32, y0 : u32, x1 : u32, y1 : u32, img : &mut RgbImage, color : Rgb<u8>) {
    let width = img.width();
    let height = img.height();
    let (mut x0, mut y0, mut x1, mut y1) = ((width - x0) as u32, (height - y0) as u32,
     (width - x1) as u32, (height - y1) as u32);
    let mut steep = false;
    if i64::abs(x0 as i64 - x1 as i64) < i64::abs(y0 as i64 - y1 as i64) {
        std::mem::swap(&mut x0, &mut y0);
        std::mem::swap(&mut x1, &mut y1);
        steep = true;
    }
    if x0 > x1 {
        std::mem::swap(&mut x0, &mut x1);
        std::mem::swap(&mut y0, &mut y1);
    }
    for x in x0..x1 {
        let t = (x - x0) as f32 / (x1 - x0) as f32;
        let y = ((y0 as f32 * (1.0 - t)) + y1 as f32 * t) as u32;
        if steep {
            if x < width && y < height {
                img.put_pixel(y, x as u32, color)
            }
        } else {
            if x < width && y < height {
                img.put_pixel(x as u32, y, color)
            }
        }
    }
}

pub fn draw_wireframe(models : Vec<tobj::Model>, img : &mut RgbImage) {
    let width = img.width();
    let height = img.height();

    for model in models {
        let indices = model.mesh.indices;
        let positions = model.mesh.positions;
        for i in 0..(indices.len() / 3) {
            let f = [indices[i*3], indices[i*3+1], indices[i*3+2]];
            for j in 0 as usize..3 as usize {
                let v0 = (positions[(f[j] * 3) as usize], positions[(f[j] * 3 + 1) as usize],
                    positions[(f[j] * 3 + 2) as usize]);
                let v1i = (j + 1) % 3;
                let v1 = (positions[(f[v1i] * 3) as usize], positions[(f[v1i] * 3 + 1) as usize],
                positions[(f[v1i] * 3 + 2) as usize]);
                let x0 = ((v0.0 + 1.0) * (width as f32) / 2.0) as u32;
                let y0 = ((v0.1 + 1.0) * (height as f32) / 2.0) as u32;
                let x1 = ((v1.0 + 1.0) * (width as f32) / 2.0) as u32;
                let y1 = ((v1.1 + 1.0) * (height as f32) / 2.0) as u32;
                draw_line(x0, y0, x1, y1, img, Rgb([255,255,255]))
            }
        }
    }
}