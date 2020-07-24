use nalgebra::{Vector4, Matrix4};
use image::{Rgb, RgbImage};

// Vertex attributes
pub enum VertexAttrb {
    TextureCoord(f32, f32),
    _Intensity(f32)
}

pub trait Shader {
    fn vertex(&self, t : u32, v : u32) -> (Vector4<f32>, Vec<VertexAttrb>);
    fn fragment(&self, bc: (f32, f32, f32), ws : (f32, f32, f32),  attrs : (&Vec<VertexAttrb>, &Vec<VertexAttrb>, &Vec<VertexAttrb>)) -> (Rgb<u8>, bool);
}

fn interpolate_tex(bc : (f32, f32, f32), ws : (f32, f32, f32) , uvs : ((f32, f32), (f32, f32), (f32, f32)), w_reci : f32) -> (f32, f32) {
    let u_s = bc.0 * (uvs.0).0 * ws.0 + bc.1 * (uvs.1).0 * ws.1 + bc.2 * (uvs.2).0 * ws.2;
    let v_s = bc.0 * (uvs.0).1 * ws.0 + bc.1 * (uvs.1).1 * ws.1 + bc.2 * (uvs.2).1 * ws.2;
    let u = u_s / w_reci;
    let v = v_s / w_reci;
    (u, v)
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

    fn vertex(&self, t : u32, v: u32) -> (Vector4<f32>, Vec<VertexAttrb>) {
        let id = self.indices[(t * 3 + v) as usize] as usize;
        let vert = Vector4::new(self.positions[id*3], self.positions[id*3+1], self.positions[id*3+2], 1.);
        let tc = VertexAttrb::TextureCoord(self.texcoords[id*2], self.texcoords[id*2 + 1]);
        let v = self.m * vert;
        let v = Vector4::new(v.x / v.w, v.y / v.w, v.z / v.w, 1. / v.w);
        (v, vec!(tc))
    }

    fn fragment(&self, bc: (f32, f32, f32), ws : (f32, f32, f32), attrs : (&Vec<VertexAttrb>, &Vec<VertexAttrb>, &Vec<VertexAttrb>)) -> (Rgb<u8>, bool) {
        let panic_txt = "Expecting TextureCoord!";
        let uv0 = match attrs.0[0] {
            VertexAttrb::TextureCoord(u, v) => (u, v),
            _ => panic!(panic_txt)
        };
        let uv1 = match attrs.1[0] {
            VertexAttrb::TextureCoord(u, v) => (u, v),
            _ => panic!(panic_txt)
        };
        let uv2 = match attrs.2[0] {
            VertexAttrb::TextureCoord(u, v) => (u, v),
            _ => panic!(panic_txt)
        };
        let w_reci = bc.0 * ws.0 + bc.1 * ws.1 + bc.2 * ws.2;
        let (u, v) = interpolate_tex(bc, ws, (uv0, uv1, uv2), w_reci);
        let tx = (u * (self.diffuse_width - 1) as f32) as u32;
        let ty = self.diffuse_height - (f32::round(v * (self.diffuse_height - 1) as f32) as u32) - 1;
        let color = self.diffuse.get_pixel(tx, ty);
        (Rgb([color[0],color[1],color[2]]), false)
    }

}