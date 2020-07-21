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

// pub fn orthographic(l : f32, r : f32, b : f32, t : f32, n : f32, f : f32) -> Matrix4<f32> {
//     let mut m = Matrix4::<f32>::identity();
//     m[(0,0)] = 2. / (r - l);
//     m[(0,3)] = - (r + l) / (r - l);
//     m[(1,1)] = 2. / (t - b);
//     m[(1,3)] = -  (t + b) / (t - b);
//     m[(2,2)] = 2. / (n - f);
//     m[(2,3)] = - (n + f) / (n - f);
//     m
// }

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

pub fn model_view(e : Vector3<f32>, g : Vector3<f32>, t : Vector3<f32>) -> Matrix4<f32> {
    let z = (e - g).normalize();
    let x = t.cross(&z).normalize();
    let y = z.cross(&x).normalize();
    let mut minv = Matrix4::<f32>::identity();
    let mut tr = Matrix4::<f32>::identity();
    for i in 0..3 {
        minv[(0,i)] = x[i];
        minv[(1,i)] = y[i];
        minv[(2,i)] = z[i];
        tr[(i,3)] = -g[i];
    }
    minv * tr
}

// pub fn perspective(fov : f32, aspect_ratio : f32, n : f32, f : f32) -> Matrix4<f32> {
//     // let t1 = Matrix4::new(1., 0., 0., 0.,
//     //     0., 1., 0., 0.,
//     //     0., 0., 1., 1.,
//     //     0., 0., 0., 1.);
//     // let t2 = Matrix4::new(1., 0., 0., 0.,
//     //     0., 1., 0., 0.,
//     //     0., 0., 1., 1.,
//     //     0., 0., 0., 1.);
//     // $\frac{1}{\tan(fov/2)}$
//     let reci_tan_h_fov_r = 1. / f32::tan(fov.to_radians() / 2.);
//     let m = Matrix4::<f32>::new(reci_tan_h_fov_r / aspect_ratio, 0., 0., 0.,
//                         0., reci_tan_h_fov_r, 0., 0.,
//                         0., 0., (f32::abs(f) + f32::abs(n)) / (n - f), 2. * f32::abs(f) * f32::abs(n) / (f - n),
//                         0., 0., 1., 0.);
//     m
// }

pub fn perspective(l : f32, r : f32, b : f32, t : f32, n : f32, f : f32) -> Matrix4<f32> {
    Matrix4::<f32>::new(2.*n/(r-l), 0., (l+r)/(l-r), 0.,
                        0.,  2.*n/(t-b), (b+t)/(b-t), 0.,
                        0., 0., (f+n)/(n-f), 2.*f*n/(f-n),
                        0., 0., 1., 0.)
}

// pub fn persp_to_ortho(c : f32) -> Matrix4<f32> {
//     let t = Matrix4::new(1., 0., 0., 0.,
//         0., 1., 0., 0.,
//         0., 0., 1., -1.,
//         0., 0., 0., 1.);
//     Matrix4::new(c, 0., 0., 0.,
//         0., c, 0., 0.,
//         0., 0., 1., 0.,
//         0., 0., -1./c, 1.) * t
// }