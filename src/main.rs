use wgpu_voxel_game::run;

fn main() {
    pollster::block_on(run());
}