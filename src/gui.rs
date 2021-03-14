use crate::{display::*, timestep};
use imgui_winit_support::WinitPlatform;
use std::time::Instant;
struct Layout {
    clock: timestep::TimeStep,
}

pub struct Gui {
    pub platform: imgui_winit_support::WinitPlatform,
    pub context: imgui::Context,
    pub renderer: imgui_wgpu::Renderer,
    layout: Layout,
}

impl Gui {
    pub fn new(window: &winit::window::Window, display: &Display) -> Self {
        let mut context = imgui::Context::create();
        let mut platform = imgui_winit_support::WinitPlatform::init(&mut context);
        platform.attach_window(
            context.io_mut(),
            &window,
            imgui_winit_support::HiDpiMode::Default,
        );

        let hidpi_factor = window.scale_factor();
        let font_size = (13.0 * hidpi_factor) as f32;
        context.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;
        context
            .fonts()
            .add_font(&[imgui::FontSource::DefaultFontData {
                config: Some(imgui::FontConfig {
                    oversample_h: 1,
                    pixel_snap_h: true,
                    size_pixels: font_size,
                    ..Default::default()
                }),
            }]);

        let renderer_config = imgui_wgpu::RendererConfig {
            texture_format: display.swap_chain_descriptor.format,
            ..Default::default()
        };

        let renderer = imgui_wgpu::Renderer::new(
            &mut context,
            &display.device,
            &display.queue,
            renderer_config,
        );

        let layout = Layout {
            clock: timestep::TimeStep::new(),
        };

        Self {
            platform,
            context,
            renderer,
            layout,
        }
    }

    pub fn render(
        &mut self,
        window: &winit::window::Window,
        frame: &wgpu::SwapChainTexture,
        encoder: &mut wgpu::CommandEncoder,
        display: &Display,
    ) {
        let current_time = Instant::now();
        let dt = current_time.duration_since(self.layout.clock.last_time);
        self.layout.clock.delta();
        self.context.io_mut().update_delta_time(dt);
        self.platform
            .prepare_frame(self.context.io_mut(), window)
            .unwrap();
        let fps = self.layout.clock.frame_rate;
        let ui = self.context.frame();
        {
            let window = imgui::Window::new(imgui::im_str!("Hello Imgui from WGPU!"));
            window
                .size([300.0, 100.0], imgui::Condition::FirstUseEver)
                .build(&ui, || {
                    ui.text(imgui::im_str!("Hello world!"));
                    ui.text(imgui::im_str!(
                        "This is a demo of imgui-rs using imgui-wgpu!"
                    ));
                    ui.separator();
                    ui.text(imgui::im_str!("FPS: ({:.1})", fps,));
                });
        }

        self.platform.prepare_render(&ui, &window);

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: &frame.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        });
        self.renderer
            .render(ui.render(), &display.queue, &display.device, &mut pass);
    }
}
