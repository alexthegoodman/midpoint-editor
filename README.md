# Midpoint / Rust Game Engine

![alt text](assets/image.png "Midpoint on Windows")

Vision: Place UX and AI at the center. Generate concepts and textures, then models, then animations, all with full control. Bring your story to life!

### Instructions

- `cargo run --release`

### Roadmap

- Will be using a local Hunyuan3D-2 instance for 3D model generation (avoiding the server and files repos which enabled syncing and cloud generation with TripoSR)
- Will continue using Replicate for image generation, although likely without a server in between, or perhaps a lightweight local server (not the full commonos-server)

### Deprecated instructions

- Setup `commonos-server` (Node.js) Used for uploading and generating
- Setup `commonos-files` (Tauri + Vite) Also used for uploading and generating
- Then setup `midpoint-editor` (Rust + Floem + wgpu) Leverages `midpoint-engine` and Floem to implement an editor
