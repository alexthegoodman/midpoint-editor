# Midpoint / Rust Game Engine

![alt text](assets/image.png "Midpoint on Windows")

Vision: Place UX and AI at the center. Generate concepts and textures, then models, then animations, all with full control. Bring your story to life!

### Latest instructions
- Pull `midpoint-engine` beside `midpoint-editor`
- Use the relevant setup scripts to setup TripoSR (MIT) for generating 3D models
    - `./scripts/manage-triposr.ps1` for Windows
    - `./scripts/manage-triposr.sh` for Linux / MacOS

#### Linux / MacOS
##### First time setup
chmod +x manage-triposr.sh
./manage-triposr.sh install

##### Check status
./manage-triposr.sh status

##### Process image
./manage-triposr.sh run -i "examples/chair.png" -o "output"

#### Windows
##### First time setup
.\manage-triposr.ps1 -Install

##### Start the server
.\manage-triposr.ps1 -Start

##### Check status
.\manage-triposr.ps1 -Status

##### Stop the server
.\manage-triposr.ps1 -Stop

##### Process an image (manually)
First, import the functions:
. .\manage-triposr.ps1

Then, process an image without texture baking:
Invoke-TripoSR -InputImage "path/to/image.png" -OutputDir "output"

### Deprecated instructions

- Setup `commonos-server` (Node.js) Used for uploading and generating
- Setup `commonos-files` (Tauri + Vite) Also used for uploading and generating
- Then setup `midpoint-editor` (Rust + Floem + wgpu) Leverages `midpoint-engine` and Floem to implement an editor
