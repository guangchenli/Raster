use nalgebra::{Vector3, Vector4, Matrix3, Matrix4};
use image::{Rgb, RgbImage};

macro_rules! unwrap_vertex_attr_1f {
    ($v : expr, $t : ident, $c : ident, $msg : expr) => {
        match $v {
            $t::$c(val) => val,
            _ => panic!($msg)
        };
    };
}

macro_rules! unwrap_vertex_attr_2f {
    ($v : expr, $t : ident, $c : ident, $msg : expr) => {
        match $v {
            $t::$c(v1, v2) => (v1, v2),
            _ => panic!($msg)
        };
    };
}

// Vertex attributes
pub enum VertexAttr {
    TextureCoord(f32, f32),
    LightIntensity(f32)
}

pub trait Shader {
    fn vertex(&self, t : u32, v : u32) -> (Vector4<f32>, Vec<VertexAttr>);
    fn fragment(&self, bc: (f32, f32, f32), ws : (f32, f32, f32),  attrs : (&Vec<VertexAttr>, &Vec<VertexAttr>, &Vec<VertexAttr>)) -> (Rgb<u8>, bool);
}

fn interpolate_tex(bc : (f32, f32, f32), ws : (f32, f32, f32) , uvs : ((f32, f32), (f32, f32), (f32, f32)), w_reci : f32) -> (f32, f32) {
    let u_s = bc.0 * (uvs.0).0 * ws.0 + bc.1 * (uvs.1).0 * ws.1 + bc.2 * (uvs.2).0 * ws.2;
    let v_s = bc.0 * (uvs.0).1 * ws.0 + bc.1 * (uvs.1).1 * ws.1 + bc.2 * (uvs.2).1 * ws.2;
    let u = u_s / w_reci;
    let v = v_s / w_reci;
    (u, v)
}

fn interpolate_one_attr(bc : (f32, f32, f32), ws : (f32, f32, f32), attrs : (f32, f32, f32), w_reci : f32) -> f32 {
    (bc.0 * attrs.0 * ws.0 + bc.1 * attrs.1 * ws.1 + bc.2 * attrs.2 * ws.2) / w_reci
}

fn clamp(r : f32, g : f32, b : f32) -> Rgb<u8> {
    let r = if r > 255. {255} else {r as u8};
    let g = if g > 255. {255} else {g as u8};
    let b = if b > 255. {255} else {b as u8};
    Rgb([r, g, b])
}

fn calc_gouraud_color(li_int : f32, ambient : f32, rgb : &Rgb<u8>) -> Rgb<u8> {
    let (r, g, b) = (rgb[0] as f32, rgb[1] as f32, rgb[2] as f32);
    let r = r * (li_int + ambient);
    let g = g * (li_int + ambient);
    let b = b * (li_int + ambient);
    clamp(r, g, b)
}

fn calc_blinnphong_color(diffuse_int : f32, spec_int : f32, ambient : f32, diffuse_rgb : &Rgb<u8>, spec_rgb : &Rgb<u8>) -> Rgb<u8>{
    let (dr, dg, db) = (diffuse_rgb[0] as f32, diffuse_rgb[1] as f32, diffuse_rgb[2] as f32);
    let (sr, sg, sb) = (spec_rgb[0] as f32, spec_rgb[1] as f32, spec_rgb[2] as f32);
    let r = dr * (diffuse_int + ambient) + sr * spec_int;
    let g = dg * (diffuse_int + ambient) + sg * spec_int;
    let b = db * (diffuse_int + ambient) + sb * spec_int;
    clamp(r, g, b)
}

// Most basic shader, only has ambient lighting
pub struct VanillaShader<'a> {
    pub m : Matrix4<f32>,
    pub indices :  &'a Vec<u32>,
    pub positions : &'a Vec<f32>,
    pub texcoords : &'a Vec<f32>,
    pub diffuse_height : u32,
    pub diffuse_width : u32,
    pub diffuse : &'a RgbImage
}

impl Shader for VanillaShader<'_> {

    fn vertex(&self, t : u32, v: u32) -> (Vector4<f32>, Vec<VertexAttr>) {
        let id = self.indices[(t * 3 + v) as usize] as usize;
        let vert = Vector4::new(self.positions[id*3], self.positions[id*3+1], self.positions[id*3+2], 1.);
        let tc = VertexAttr::TextureCoord(self.texcoords[id*2], self.texcoords[id*2 + 1]);
        let v = self.m * vert;
        let v = Vector4::new(v.x / v.w, v.y / v.w, v.z / v.w, 1. / v.w);
        (v, vec!(tc))
    }

    fn fragment(&self, bc: (f32, f32, f32), ws : (f32, f32, f32), attrs : (&Vec<VertexAttr>, &Vec<VertexAttr>, &Vec<VertexAttr>)) -> (Rgb<u8>, bool) {
        let uv0 = unwrap_vertex_attr_2f!(attrs.0[0], VertexAttr, TextureCoord, "Expecting TextureCoord!");
        let uv1 = unwrap_vertex_attr_2f!(attrs.1[0], VertexAttr, TextureCoord, "Expecting TextureCoord!");
        let uv2 = unwrap_vertex_attr_2f!(attrs.2[0], VertexAttr, TextureCoord, "Expecting TextureCoord!");
        let w_reci = bc.0 * ws.0 + bc.1 * ws.1 + bc.2 * ws.2;
        let (u, v) = interpolate_tex(bc, ws, (uv0, uv1, uv2), w_reci);
        let tx = (u * (self.diffuse_width - 1) as f32) as u32;
        let ty = self.diffuse_height - (f32::round(v * (self.diffuse_height - 1) as f32) as u32) - 1;
        let color = self.diffuse.get_pixel(tx, ty);
        (Rgb([color[0],color[1],color[2]]), false)
    }

}


// An implementation of GauraudShader
pub struct GouraudShader<'a> {
    pub mvp : Matrix4<f32>,
    pub model : Matrix3<f32>,
    pub indices : &'a Vec<u32>,
    pub positions : &'a Vec<f32>,
    pub texcoords : &'a Vec<f32>,
    pub normals : &'a Vec<f32>,
    pub diffuse : &'a RgbImage,
    pub light_source : &'a Vec<(Vector3<f32>, f32)>,
    pub ambient : f32
}

impl Shader for GouraudShader<'_> {

    fn vertex(&self, t : u32, v: u32) -> (Vector4<f32>, Vec<VertexAttr>) {
        let idx = self.indices[(t * 3 + v) as usize] as usize;
        // calculate normal vector
        // model in left hand coord, flip x y z val
        let n = Vector3::new(-self.normals[idx*3], -self.normals[idx*3+1], -self.normals[idx*3+2]);
        let n = (self.model * n).normalize();
        let v = Vector4::new(self.positions[idx*3], self.positions[idx*3+1], self.positions[idx*3+2], 1.);
        // calculate light intensity
        let mut vert_intensity = 0.;
        for (li_dir, li_intensity) in self.light_source.iter() {
            vert_intensity += n.dot(&li_dir.normalize()).max(0.) * li_intensity;
        }
        let vert_intensity = VertexAttr::LightIntensity(vert_intensity);
        let tc = VertexAttr::TextureCoord(self.texcoords[idx*2], self.texcoords[idx*2 + 1]);
        let v = self.mvp * v;
        let v = Vector4::new(v.x / v.w, v.y / v.w, v.z / v.w, 1. / v.w);
        (v, vec!(tc, vert_intensity))
    }

    fn fragment(&self, bc: (f32, f32, f32), ws : (f32, f32, f32), attrs : (&Vec<VertexAttr>, &Vec<VertexAttr>, &Vec<VertexAttr>)) -> (Rgb<u8>, bool) {
        let msg_texcoord = "Expecting TextureCoord!";
        let msg_li_int = "Expecting LightIntensity!";
        let uv0 = unwrap_vertex_attr_2f!(attrs.0[0], VertexAttr, TextureCoord, msg_texcoord);
        let uv1 = unwrap_vertex_attr_2f!(attrs.1[0], VertexAttr, TextureCoord, msg_texcoord);
        let uv2 = unwrap_vertex_attr_2f!(attrs.2[0], VertexAttr, TextureCoord, msg_texcoord);
        let li_int_v0 = unwrap_vertex_attr_1f!(attrs.0[1], VertexAttr, LightIntensity, msg_li_int);
        let li_int_v1 = unwrap_vertex_attr_1f!(attrs.1[1], VertexAttr, LightIntensity, msg_li_int);
        let li_int_v2 = unwrap_vertex_attr_1f!(attrs.2[1], VertexAttr, LightIntensity, msg_li_int);
        let w_reci = bc.0 * ws.0 + bc.1 * ws.1 + bc.2 * ws.2;
        let (u, v) = interpolate_tex(bc, ws, (uv0, uv1, uv2), w_reci);
        let li_int = interpolate_one_attr(bc, ws, (li_int_v0, li_int_v1, li_int_v2), w_reci);
        let tx = (u * (self.diffuse.width() - 1) as f32) as u32;
        let ty = self.diffuse.height()- (f32::round(v * (self.diffuse.height() - 1) as f32) as u32) - 1;
        let diffuse_color = self.diffuse.get_pixel(tx, ty);
        (calc_gouraud_color(li_int, self.ambient, diffuse_color), false)
    }

}

pub struct BlinnPhongShader<'a> {
    pub mvp : Matrix4<f32>,
    pub model : Matrix3<f32>,
    pub e : Vector3<f32>,
    pub indices : &'a Vec<u32>,
    pub positions : &'a Vec<f32>,
    pub texcoords : &'a Vec<f32>,
    pub normals : &'a Vec<f32>,
    pub diffuse : &'a RgbImage,
    pub spec : &'a RgbImage,
    pub light_source : &'a Vec<(Vector3<f32>, f32)>,
    pub ambient : f32,
    pub phong_exp : f32
}

// A Blinn-Phong Shader
impl Shader for BlinnPhongShader<'_> {

    fn vertex(&self, t : u32, v: u32) -> (Vector4<f32>, Vec<VertexAttr>) {
        let idx = self.indices[(t * 3 + v) as usize] as usize;
        // calculate normal vector
        // model in left hand coord, flip x y z val
        let n = Vector3::new(-self.normals[idx*3], -self.normals[idx*3+1], -self.normals[idx*3+2]);
        let n = (self.model * n).normalize();
        let v = Vector4::new(self.positions[idx*3], self.positions[idx*3+1], self.positions[idx*3+2], 1.);
        // calculate diffuse and specular light intensity
        let mut diffuse_intensity = 0.;
        let mut spec_intensity = 0.;
        for (li_dir, intensity) in self.light_source.iter() {
            let l = li_dir;
            let h = (self.e + -l).normalize();
            diffuse_intensity += n.dot(&l).max(0.) * intensity;
            spec_intensity += h.dot(&n).abs().powf(self.phong_exp) * intensity;
        }
        //println!("{}", spec_intensity);
        let diffuse_intensity = VertexAttr::LightIntensity(diffuse_intensity);
        let spec_intensity = VertexAttr::LightIntensity(spec_intensity);
        let tc = VertexAttr::TextureCoord(self.texcoords[idx*2], self.texcoords[idx*2 + 1]);
        let v = self.mvp * v;
        let v = Vector4::new(v.x / v.w, v.y / v.w, v.z / v.w, 1. / v.w);
        (v, vec!(tc, diffuse_intensity, spec_intensity))
    }

    fn fragment(&self, bc: (f32, f32, f32), ws : (f32, f32, f32), attrs : (&Vec<VertexAttr>, &Vec<VertexAttr>, &Vec<VertexAttr>)) -> (Rgb<u8>, bool) {
        let msg_texcoord = "Expecting TextureCoord!";
        let msg_li_int = "Expecting LightIntensity!";
        let uv0 = unwrap_vertex_attr_2f!(attrs.0[0], VertexAttr, TextureCoord, msg_texcoord);
        let uv1 = unwrap_vertex_attr_2f!(attrs.1[0], VertexAttr, TextureCoord, msg_texcoord);
        let uv2 = unwrap_vertex_attr_2f!(attrs.2[0], VertexAttr, TextureCoord, msg_texcoord);
        let diffuse_int_v0 = unwrap_vertex_attr_1f!(attrs.0[1], VertexAttr, LightIntensity, msg_li_int);
        let diffuse_int_v1 = unwrap_vertex_attr_1f!(attrs.1[1], VertexAttr, LightIntensity, msg_li_int);
        let diffuse_int_v2 = unwrap_vertex_attr_1f!(attrs.2[1], VertexAttr, LightIntensity, msg_li_int);
        let spec_int_v0 = unwrap_vertex_attr_1f!(attrs.0[2], VertexAttr, LightIntensity, msg_li_int);
        let spec_int_v1 = unwrap_vertex_attr_1f!(attrs.1[2], VertexAttr, LightIntensity, msg_li_int);
        let spec_int_v2 = unwrap_vertex_attr_1f!(attrs.2[2], VertexAttr, LightIntensity, msg_li_int);
        let w_reci = bc.0 * ws.0 + bc.1 * ws.1 + bc.2 * ws.2;
        let (u, v) = interpolate_tex(bc, ws, (uv0, uv1, uv2), w_reci);
        let diffuse_int = interpolate_one_attr(bc, ws, (diffuse_int_v0, diffuse_int_v1, diffuse_int_v2), w_reci);
        let spec_int = interpolate_one_attr(bc, ws, (spec_int_v0, spec_int_v1, spec_int_v2), w_reci);
        let tx = (u * (self.diffuse.width() - 1) as f32) as u32;
        let ty = self.diffuse.height() - (f32::round(v * (self.diffuse.height() - 1) as f32) as u32) - 1;
        let diffuse_color = self.diffuse.get_pixel(tx, ty);
        let spec_color = self.spec.get_pixel(tx, ty);
        (calc_blinnphong_color(diffuse_int, spec_int, self.ambient, diffuse_color, spec_color), false)
    }

}