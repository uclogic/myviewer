#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui::{self, ColorImage, Event, Vec2, ViewportBuilder};
use egui_plot::{PlotImage, PlotPoint};
use image::ImageReader;
use std::env;

fn main() -> eframe::Result {
    let args: Vec<String> = env::args().collect();
    let path = if args.len() > 1 { &args[1] } else { "" };

    let img = ImageReader::open(path)
        .expect("Failed to load image")
        .decode()
        .expect("Failed to decode image");
    let size = [img.width() as _, img.height() as _];
    let image_buffer = img.to_rgba8();
    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &image_buffer);

    let scaling = Scaling::new(1920.0, 1080.0);
    let (scaled_a, scaled_b) = scaling.scale(size[0] as f32, size[1] as f32);

    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_inner_size(Vec2::new(scaled_a, scaled_b)),
        ..Default::default()
    };

    eframe::run_native(
        "Image Viewer",
        options,
        Box::new(|cc| {
            // This gives us image support:
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::new(MyApp::new(color_image, cc)))
        }),
    )
}

#[derive(Default)]
struct MyApp {
    zoom_speed: f32,
    texture: Option<egui::TextureHandle>,
}

impl MyApp {
    fn new(color_image: ColorImage, cc: &eframe::CreationContext<'_>) -> Self {
        let mut app = MyApp {
            zoom_speed: 1.0,
            texture: None,
        };

        let _texture: &egui::TextureHandle = app.texture.get_or_insert_with(|| {
            cc.egui_ctx.input_mut(|i| i.max_texture_side = 6000);
            cc.egui_ctx
                .load_texture("my-image", color_image, Default::default())
        });

        app
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let (scroll, pointer_down, _modifiers) = ui.input(|i| {
                let scroll = i.events.iter().find_map(|e| match e {
                    Event::MouseWheel {
                        unit: _,
                        delta,
                        modifiers: _,
                    } => Some(*delta),
                    _ => None,
                });
                (scroll, i.pointer.primary_down(), i.modifiers)
            });

            egui_plot::Plot::new("plot")
                .show_background(false)
                .allow_zoom(false)
                .allow_drag(false)
                .allow_scroll(false)
                .show_grid(false)
                .show_axes(false)
                .data_aspect(1.0)
                .show_x(false)
                .show_y(false)
                .label_formatter(|name, value| { "".to_owned()})
                .show(ui, |plot_ui| {
                    if let Some(mut scroll) = scroll {
                        scroll = Vec2::splat(scroll.x + scroll.y);
                        let zoom_factor = Vec2::from([
                            (scroll.x * self.zoom_speed / 10.0).exp(),
                            (scroll.y * self.zoom_speed / 10.0).exp(),
                        ]);
                        plot_ui.zoom_bounds_around_hovered(zoom_factor);
                    }
                    if plot_ui.response().hovered() && pointer_down {
                        let pointer_translate = -plot_ui.pointer_coordinate_drag_delta();
                        plot_ui.translate_bounds(pointer_translate);
                    }

                    let size = self.texture.as_ref().unwrap().size_vec2();
                    let tex = self.texture.as_mut().unwrap();
                    plot_ui.image(PlotImage::new(tex, PlotPoint::new(0.0, 0.0), size));
                });
        });
    }
}


struct Scaling {
    max_x: f32,
    max_y: f32,
}

impl Scaling {
    fn new(max_x: f32, max_y: f32) -> Self {
        Scaling { max_x, max_y }
    }

    fn scale(&self, a: f32, b: f32) -> (f32, f32) {
        let mut scale_factor = 1.0;

        if a > self.max_x && b > self.max_y {
            // 如果a和b都超过最大值，取较小的缩放比例
            scale_factor = self.max_x / a;
            scale_factor = f32::min(scale_factor, self.max_y / b);
        } else if a > self.max_x {
            // 只有a超过最大值，缩放a，保持b的比例
            scale_factor = self.max_x / a;
        } else if b > self.max_y {
            // 只有b超过最大值，缩放b，保持a的比例
            scale_factor = self.max_y / b;
        } else {
            return (self.max_x, self.max_y);
        }

        (a * scale_factor, b * scale_factor)
    }
}
