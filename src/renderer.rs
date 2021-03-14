use futures::executor::block_on;
use winit::window::Window;

use crate::{
    camera::{Camera, Projection},
    display::Display,
    instance::{Instance, InstanceRaw},
    mesh::DrawModel,
    pipeline::{Pipeline, SimplePipeline},
    scene::Scene,
};

pub struct Renderer<P: Pipeline> {
    camera_metadata: Projection,
    pub display: Display,
    pub pipeline: P,
}

impl<P: Pipeline> Renderer<P> {
    pub fn new(window: &Window) -> Self {
        let display = block_on(Display::new(window));
        let camera_metadata = Projection::new(
            display.swap_chain_descriptor.width,
            display.swap_chain_descriptor.height,
            cgmath::Deg(45.0),
            0.1,
            100.0,
        );
        let pipeline = P::new(&display);

        Self {
            display,
            camera_metadata,
            pipeline,
        }
    }

    pub fn render(
        &mut self,
        frame: &wgpu::SwapChainTexture,
        encoder: &mut wgpu::CommandEncoder,
        scene: Box<Scene>,
        camera: &Camera,
    ) {
        self.pipeline.update_view_position(camera.position());
        self.pipeline
            .update_view_projection(camera.projection(&self.camera_metadata));

        self.pipeline.prepare(&self.display);

        {
            let mut current_scene = Some(scene);
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &frame.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: self.pipeline.depth_stencil_attachment(),
            });
            self.pipeline.bind(&mut render_pass);

            while current_scene.is_some() {
                let scene = current_scene.unwrap();
                let instance_data = scene
                    .instances
                    .iter()
                    .map(Instance::to_raw)
                    .collect::<Vec<_>>();

                scene
                    .model
                    .load_instance_buffers(&self.display, instance_data);

                render_pass.draw_model_instanced(scene.model, 0..scene.instances.len() as u32);
                current_scene = scene.next;
            }
        }
    }
}
