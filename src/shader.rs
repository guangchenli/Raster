use nalgebra::{Vector3, Vector4, Matrix4};
use image::{Rgb, RgbImage};

macro_rules! unwrap_vertex_attr_1f {
    ($v : expr, $t : ident, $c : ident, $msg : literal) => {
        match $v {
            $t::$c(val) => val,
            _ => panic!($msg)
        };
    };
}

macro_rules! unwrap_vertex_attr_2f {
    ($v : expr, $t : ident, $c : ident, $msg : literal) => {
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

pub struct VanillaShader {
    pub m : Matrix4<f32>,
    pub indices :  Vec<u32>,
    pub positions : Vec<f32>,
    pub texcoords : Vec<f32>,
    pub diffuse_height : u32,
    pub diffuse_width : u32,
    pub diffuse : RgbImage
}

impl Shader for VanillaShader {

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

pub struct GouraudShader {
    pub m : Matrix4<f32>,
    pub indices :  Vec<u32>,
    pub positions : Vec<f32>,
    pub texcoords : Vec<f32>,
    pub normals : Vec<f32>,
    pub diffuse_height : u32,
    pub diffuse_width : u32,
    pub diffuse : RgbImage,
    pub light_source : Vec<(Vector3<f32>, f32)>,
    pub ambient : f32
}

// WARNIMG!: This Gouraud shader is wrong, it does not count model transformation
// Only for test purpose
impl Shader for GouraudShader {

    fn vertex(&self, t : u32, v: u32) -> (Vector4<f32>, Vec<VertexAttr>) {
        let idx = self.indices[(t * 3 + v) as usize] as usize;
        let n = Vector3::new(self.normals[idx*3], self.normals[idx*3+1], self.normals[idx*3+2]);
        let v = Vector4::new(self.positions[idx*3], self.positions[idx*3+1], self.positions[idx*3+2], 1.);
        // calculate light intensity
        let mut vert_intensity = 0.;
        for (li_dir, li_intensity) in self.light_source.iter() {
            vert_intensity += n.dot(&li_dir.normalize()).max(0.) * li_intensity;
        }
        let vert_intensity = VertexAttr::LightIntensity(vert_intensity);
        let tc = VertexAttr::TextureCoord(self.texcoords[idx*2], self.texcoords[idx*2 + 1]);
        let v = self.m * v;
        let v = Vector4::new(v.x / v.w, v.y / v.w, v.z / v.w, 1. / v.w);
        (v, vec!(tc, vert_intensity))
    }

    fn fragment(&self, bc: (f32, f32, f32), ws : (f32, f32, f32), attrs : (&Vec<VertexAttr>, &Vec<VertexAttr>, &Vec<VertexAttr>)) -> (Rgb<u8>, bool) {
        let uv0 = unwrap_vertex_attr_2f!(attrs.0[0], VertexAttr, TextureCoord, "Expecting TextureCoord!");
        let uv1 = unwrap_vertex_attr_2f!(attrs.1[0], VertexAttr, TextureCoord, "Expecting TextureCoord!");
        let uv2 = unwrap_vertex_attr_2f!(attrs.2[0], VertexAttr, TextureCoord, "Expecting TextureCoord!");
        let li_int_v0 = unwrap_vertex_attr_1f!(attrs.0[1], VertexAttr, LightIntensity, "Expecting LightIntensity!");
        let li_int_v1 = unwrap_vertex_attr_1f!(attrs.1[1], VertexAttr, LightIntensity, "Expecting LightIntensity!");
        let li_int_v2 = unwrap_vertex_attr_1f!(attrs.2[1], VertexAttr, LightIntensity, "Expecting LightIntensity!");
        let w_reci = bc.0 * ws.0 + bc.1 * ws.1 + bc.2 * ws.2;
        let (u, v) = interpolate_tex(bc, ws, (uv0, uv1, uv2), w_reci);
        let li_int = interpolate_one_attr(bc, ws, (li_int_v0, li_int_v1, li_int_v2), w_reci);
        let tx = (u * (self.diffuse_width - 1) as f32) as u32;
        let ty = self.diffuse_height - (f32::round(v * (self.diffuse_height - 1) as f32) as u32) - 1;
        let diffuse_color = self.diffuse.get_pixel(tx, ty);
        (calc_gouraud_color(li_int, self.ambient, diffuse_color), false)
    }

}