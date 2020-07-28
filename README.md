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
```
cargo build
```

## Usage
```
cargo run <obj_model> <diffuse_texture> <spec_texture>
```