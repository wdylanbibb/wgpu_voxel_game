use std::env;
// TODO:
//  - Allow for batch mesh editing by allowing chunks to mark areas as dirty -- and only updating the edited blocks
//  - Allow meshes to be marked as dirty and only update the buffers for dirty meshes every frame
//  - Infinite terrain
//  - Procedurally generated chunks
//  - Water/partially transparent blocks
fn main() {
    if cfg!(debug_assertions) {
        env::set_var("RUST_BACKTRACE", "1");
    }
    wgpu_voxel_game::run();
}
