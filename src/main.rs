use wgpu_render::run_app;

fn main() {
    pollster::block_on(run_app());
}
