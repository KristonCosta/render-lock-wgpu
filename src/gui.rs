use imgui_winit_support::WinitPlatform;
use crate::renderer::display::Display;

pub struct Gui {
    pub platform: imgui_winit_support::WinitPlatform,
    pub context: imgui::Context,
    pub renderer:imgui_wgpu::Renderer,
}

impl Gui {
    pub fn new(window: &winit::window::Window, display: &Display) -> Self {
        let mut context = imgui::Context::create();
        let mut platform = imgui_winit_support::WinitPlatform::init(&mut context);
        platform.attach_window(
            context.io_mut(),
            &window,
            imgui_winit_support::HiDpiMode::Default
        );

        let hidpi_factor = window.scale_factor();
        let font_size = (13.0 * hidpi_factor) as f32;
        context.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;
        context.fonts().add_font(&[
            imgui::FontSource::DefaultFontData {
                config: Some(
                    imgui::FontConfig {
                        oversample_h: 1,
                        pixel_snap_h: true,
                        size_pixels: font_size,
                        ..Default::default()
                    }
                ),
            }
        ]);

        let renderer_config = imgui_wgpu::RendererConfig {
            texture_format: display.swap_chain_descriptor.format,
            ..Default::default()
        };

        let renderer = imgui_wgpu::Renderer::new(&mut context, &display.device, &display.queue, renderer_config);

        Self {
            platform,
            context,
            renderer
        }
    }
}