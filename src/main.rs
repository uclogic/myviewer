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
    println!("size is : {:?}", size);
    let image_buffer = img.to_rgba8();
    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &image_buffer);

    let max_size = (1920, 1080);
    // 根据maxsize缩放size
    let result_size = if size[0] > max_size.0 || size[1] > max_size.1 {
        let ratio = size[0] as f32 / size[1] as f32;
        let new_width = max_size.0 as f32;
        let new_height = new_width / ratio;
        let new_height = new_height.round() as usize;
        (max_size.0, new_height)
    } else {
        (size[0], size[1])
    };

    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_inner_size(Vec2::new(result_size.0 as f32, result_size.1 as f32)),
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
