# Midpoint / Rust Game Engine

![alt text](assets/image.png "Midpoint on Windows")

Vision: Place UX and AI at the center. Generate concepts and textures, then models, then animations, all with full control. Bring your story to life!

### Instructions

```
mkdir midpoint
cd midpoint
git clone https://github.com/alexthegoodman/common-floem.git
git clone https://github.com/alexthegoodman/midpoint-engine.git
git clone https://github.com/alexthegoodman/midpoint-editor.git
cd midpoint-editor
cargo run --release
```

### Roadmap

- Will be using a local Hunyuan3D-2 instance for 3D model generation (avoiding the server and files repos which enabled syncing and cloud generation with TripoSR)
- Will continue using Replicate for image generation, although likely without a server in between, or perhaps a lightweight local server (not the full commonos-server)

### Deprecated instructions

- Setup `commonos-server` (Node.js) Used for uploading and generating
- Setup `commonos-files` (Tauri + Vite) Also used for uploading and generating
- Then setup `midpoint-editor` (Rust + Floem + wgpu) Leverages `midpoint-engine` and Floem to implement an editor
