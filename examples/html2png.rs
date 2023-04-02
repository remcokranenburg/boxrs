extern crate boxrs;
extern crate image;

use std::default::Default;
use std::env;
use std::io::{Read, BufWriter};
use std::fs::File;

use boxrs::css::Color;

fn main() {
    let mut args = env::args().skip(1);
    let html_filename = args.next().expect("HTML file provided as first argument");
    let css_filename = args.next().expect("CSS file provided as second argument");

    let html = read_source(&html_filename);
    let css = read_source(&css_filename);

    // Since we don't have an actual window, hard-code the "viewport" size.
    let width = 800;
    let height = 600;

    let mut viewport: boxrs::layout::Dimensions = Default::default();
    viewport.content.width  = width as f32;
    viewport.content.height = height as f32;

    // Parsing and rendering:
    let root_node = boxrs::parse_html(&html);
    let stylesheet = boxrs::parse_css(&css);
    let style_root = boxrs::build_style_tree(&root_node, &stylesheet);
    let layout_root = boxrs::build_layout_tree(&style_root, viewport);
    let display_list = boxrs::build_display_list(&layout_root);

    // Create the output file:
    let filename = "output.png";
    let mut file = BufWriter::new(File::create(&filename).unwrap());

    // Rasterize:
    let background = Color { r: 255, g: 255, b: 255, a: 255 };
    let mut canvas = vec![background; width * height];

    for item in display_list {
        match item {
            boxrs::painting::DisplayCommand::SolidColor(color, rect) => {
                // Clip the rectangle to the canvas boundaries.
                let x0 = rect.x.clamp(0.0, width as f32) as usize;
                let y0 = rect.y.clamp(0.0, height as f32) as usize;
                let x1 = (rect.x + rect.width).clamp(0.0, width as f32) as usize;
                let y1 = (rect.y + rect.height).clamp(0.0, height as f32) as usize;

                for y in y0 .. y1 {
                    for x in x0 .. x1 {
                        // TODO: alpha compositing with existing pixel
                        canvas[y * width + x] = color.clone();
                    }
                }
            }
        }
    }

    let img = image::ImageBuffer::from_fn(width as u32, height as u32, move |x, y| {
        let color = &canvas[(y * width as u32 + x) as usize];
        image::Pixel::from_channels(color.r, color.g, color.b, color.a)
    });
    let result = image::DynamicImage::ImageRgba8(img).save(&mut file, image::PNG);

    match result {
        Ok(_) => println!("Saved output as {}", filename),
        Err(_) => println!("Error saving output as {}", filename),
    }
}

fn read_source(filename: &str) -> String {
    let mut s = String::new();
    File::open(filename).unwrap().read_to_string(&mut s).unwrap();
    s
}

trait Clamp {
    fn clamp(self, lower: Self, upper: Self) -> Self;
}
impl Clamp for f32 {
    fn clamp(self, lower: f32, upper: f32) -> f32 {
        self.max(lower).min(upper)
    }
}
