use futures::executor::block_on;
use winit::window::Window;

use crate::{
    camera::{Camera, CameraMetadata},
    display::Display,
    instance::{Instance, InstanceRaw},
    mesh::DrawModel,
    pipeline::{Pipeline, SimplePipeline},
    scene::Scene,
};

pub struct Renderer<P: Pipeline> {
    camera_metadata: CameraMetadata,
    pub display: Display,
    pub pipeline: P,
}

impl<P: Pipeline> Renderer<P> {
    pub fn new(window: &Window) -> Self {
        let display = block_on(Display::new(window));
        let camera_metadata = CameraMetadata::new(&display);
        let pipeline = P::new(&display);

        Self {
            display,
            camera_metadata,
            pipeline,
        }
    }

    pub fn render(&mut self, scene: Box<Scene>, camera: &Camera) {
        self.pipeline.update_view_position(camera.position());
        self.pipeline
            .update_view_projection(camera.projection(&self.camera_metadata));

        self.pipeline.prepare(&self.display);
        let frame = self.display.swap_chain.get_current_frame().unwrap().output;
        let mut encoder =
            self.display
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });
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
        self.display.queue.submit(std::iter::once(encoder.finish()));
    }
}
