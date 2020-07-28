# Raster

This is a toy software rasterizer I wrote to learn how rasterization works.

Current Features:
- Z-buffering
- Perspective corrext texture mapping
- Programmable vertex & fragment shader
- Gouraud shading
- Blinn-Phong shading

TODO List:
- Add tangent space normal mapping
- Add shadow mapping
- Add resource manager

## Build
First make sure that you have [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) installed, then simply run:
```
cargo build
```
on your terminal.
## Usage
Model must be in Wavefront obj format, make sure that your model includes tangent vector of each vertex.
```
cargo run <obj_model> <diffuse_texture> <spec_texture>
```