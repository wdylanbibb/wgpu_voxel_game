  ## Recommended MVP

  Build a single-player procedural voxel world where the player can:

  - Explore using an FPS camera with mouse look.
  - Walk, jump, collide with terrain, and use gravity.
  - Break and place blocks using voxel ray-casting.
  - Select from 4–6 visually distinct block types.
  - Reload the page without losing edits.
  - Run both natively and in the browser.

  The world should have:

  - Deterministic procedural terrain from a seed.
  - Chunk-based storage, such as 16×16×16 blocks per chunk.
  - Multiple chunks loaded around the player.
  - Hidden-face elimination at minimum.
  - Preferably greedy meshing as the main technical showcase.
  - Frustum culling and distance-based chunk unloading.
  - Directional lighting, fog, and a sky color.
  - Separate texture atlas regions for different block types.

  ## Resume-grade polish

  A project becomes resume-worthy through presentation and evidence as much as feature count.

  Include:

  - A publicly accessible web build.
  - A 30–60 second gameplay video or GIF.
  - A clear README with controls and architecture.
  - A diagram of the world → chunk → mesh → GPU pipeline.
  - Documented performance measurements.
  - A debug overlay showing FPS, visible chunks, triangles, and remesh time.
  - At least a few tests for coordinate conversion, chunk indexing, and ray traversal.
  - Automated native and WASM builds if practical.

  Set concrete targets, then report the actual results. Reasonable targets might be:

  - Stable 60 FPS at 1080p on your development machine.
  - At least a 128×64×128 active voxel region.
  - Chunk remeshing that does not visibly freeze gameplay.
  - Initial browser load within a few seconds after assets are cached.

  Don’t claim universal performance—name the browser, GPU, resolution, render distance, and observed results.

  ## Best technical centerpiece

  If you implement only one impressive optimization, make it greedy meshing.

  A naive chunk can generate six faces per solid voxel. Hidden-face elimination removes internal faces, while greedy meshing combines adjacent
  coplanar faces into larger quads. Showing a comparison such as:

  Naive:        48,000 triangles
  Face culled:   9,200 triangles
  Greedy mesh:   2,100 triangles

  makes your engineering contribution immediately understandable. Use measurements from your actual project.

  ## Persistence

  For the browser version, save:

  - The terrain seed.
  - Player position.
  - Only blocks changed from generated terrain.

  Store that small delta in localStorage or IndexedDB. Saving the entire generated world is unnecessary for an MVP.
