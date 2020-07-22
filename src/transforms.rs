use nalgebra::{Matrix4, Vector3};

pub fn viewport(x : u32, y : u32) -> Matrix4<f32> {
    let mut m = Matrix4::<f32>::identity();
    let x = x as f32;
    let y = y as f32;
    m[(0,0)] = x / 2.;
    m[(0,3)] = (x - 1.) / 2.;
    m[(1,1)] = y / 2.;
    m[(1,3)] = (y - 1.) / 2.;
    m
}

pub fn camera(e : Vector3<f32>, g : Vector3<f32>, t : Vector3<f32>) -> Matrix4<f32> {
    let w = -g / Vector3::<f32>::norm(&g);
    let txw = t.cross(&w);
    let u = txw / Vector3::<f32>::norm(&txw);
    let v = w.cross(&u);
    let m1 = Matrix4::<f32>::new(u.x, u.y, u.z, 0.,
                                 v.x, v.y, v.z, 0.,
                                 w.x, w.y, w.z, 0.,
                                 0., 0., 0., 1.);
    let m2 = Matrix4::<f32>::new(1., 0., 0., -e.x,
                                 0., 1., 0., -e.y,
                                 0., 0., 1., -e.z,
                                 0., 0., 0., 1.);
    m1 * m2
}

pub fn perspective(l : f32, r : f32, b : f32, t : f32, n : f32, f : f32) -> Matrix4<f32> {
    Matrix4::<f32>::new(2.*n/(r-l), 0., (l+r)/(l-r), 0.,
                        0.,  2.*n/(t-b), (b+t)/(b-t), 0.,
                        0., 0., (f+n)/(n-f), 2.*f*n/(f-n),
                        0., 0., 1., 0.)
}