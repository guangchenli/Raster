# Raster

This is a toy software rasterizer I wrote to learn how rasterization works.

Current Features:
- Z-buffering
- Perspective corrext texture mapping

TODO List:
- Add Blinn-Phong Shading
- Add tangent space normal mapping
- Add shadow mapping
- Add resource manager

## Build
```
cargo build
```

## Usage
```
cargo run <obj_model> <tga_texture>
```