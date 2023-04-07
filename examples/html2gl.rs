extern crate boxrs;
#[macro_use]
extern crate glium;

use std::default::Default;
use std::env;
use std::io::Read;
use std::fs::File;

use boxrs::css::Color;
use boxrs::dom::Node;
use boxrs::layout::Rect;
use boxrs::painting::DisplayCommand;
use glium::glutin;
use glium::index::{NoIndices, PrimitiveType};
use glium::{Display, Frame, Program, Surface, VertexBuffer};

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
}

implement_vertex!(Vertex, position);

fn draw_color_rectangle(target: &mut Frame, square_buffer: &VertexBuffer<Vertex>, program: &Program,
        color: &Color, rect: &Rect, layer: f32) {

    let indices = NoIndices(PrimitiveType::TriangleStrip);

    let uniforms = uniform! {
        x: rect.x,
        y: rect.y,
        width: rect.width,
        height: rect.height,
        layer: layer,
        in_color: [color.r as f32, color.g as f32, color.b as f32, color.a as f32]
    };

    target.draw(square_buffer, &indices, program, &uniforms,
            &Default::default()).unwrap();
}

fn main() {
    let mut args = env::args().skip(1);
    let html_filename = args.next().expect("HTML file provided as first argument");

    let html = read_source(&html_filename);

    // Since we don't have an actual window, hard-code the "viewport" size.
    let width = 800;
    let height = 600;

    let mut viewport: boxrs::layout::Dimensions = Default::default();
    viewport.content.width  = width as f32;
    viewport.content.height = height as f32;

    // Parsing and rendering:
    let root_node = boxrs::parse_html(&html);

    // Extract title:
    let title = match root_node.get_elements_by_tag_name("title").iter().next() {
        Some(node) => node.get_text_content(),
        None => "html2gl".to_owned(),
    };

    // TODO: replace with:
    // let css_filename = match root_node.select("html > head > link[rel=stylesheet][href]") {
    //   Some(node) => Some(node.get_attribute("href")),
    //   None => None,
    // }

    // TODO: of course, really replace this with something that keeps track of all sheets

    let mut css_filename = None;

    match root_node.get_elements_by_tag_name("link").iter().next() {
        Some(Node::Element { attrs, .. }) => {
            if attrs.contains(&("rel".to_owned(), "stylesheet".to_owned())) {
                for attr in attrs {
                    if attr.0 == "href" {
                        css_filename = Some(attr.1.clone());
                    }
                }
            }
        },
        _ => (),
    }

    println!("Opening CSS file {}", css_filename.as_ref().unwrap());

    let css = read_source(&css_filename.unwrap());

    // Combine HTML with CSS to create list of draw commands
    let stylesheet = boxrs::parse_css(&css);
    let style_root = boxrs::build_style_tree(&root_node, &stylesheet);
    let layout_root = boxrs::build_layout_tree(&style_root, viewport);
    let display_list = boxrs::build_display_list(&layout_root);

    // Render with OpenGL:
    let mut event_loop = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new()
        .with_title(format!("{title} - html2gl"));
    let cb = glutin::ContextBuilder::new()
        .with_depth_buffer(24);
    let display = Display::new(wb, cb, &event_loop).unwrap();

    let square_shape = vec![
        Vertex { position: [0.0, 0.0] },
        Vertex { position: [1.0, 0.0] },
        Vertex { position: [0.0, 1.0] },
        Vertex { position: [1.0, 1.0] },
    ];
    let square_buffer = VertexBuffer::new(&display, &square_shape).unwrap();

    let vertex_shader_src = r#"
        #version 140

        in vec2 position;

        uniform float x;
        uniform float y;
        uniform float width;
        uniform float height;
        uniform float layer;

        void main() {
            gl_Position = vec4(
                (x + position.x * width) / 800.0 * 2.0 - 1.0,
                (y + position.y * height) / 600.0 * -2.0 + 1.0,
                layer,
                1.0
            );
        }
    "#;

    let fragment_shader_src = r#"
        #version 140

        out vec4 color;

        uniform vec4 in_color;

        void main() {
            color = in_color;
        }
    "#;

    let program = Program::from_source(&display, vertex_shader_src, fragment_shader_src, None)
            .unwrap();

    event_loop.run(move |ev, _, control_flow| {
        let mut target = display.draw();
        target.clear_color_and_depth((1.0, 1.0, 1.0, 1.0), 1.0);

        let mut layer = 0.0;

        for item in &display_list {
            match item {
                DisplayCommand::SolidColor(color, rect) => {
                    draw_color_rectangle(&mut target, &square_buffer, &program, color, rect, layer);
                }
            }

            layer += 0.001;
        }

        target.finish().unwrap();

        let next_frame_time = std::time::Instant::now() +
            std::time::Duration::from_nanos(16_666_667);
        *control_flow = glutin::event_loop::ControlFlow::WaitUntil(next_frame_time);
        match ev {
            glutin::event::Event::WindowEvent { event, .. } => match event {
                glutin::event::WindowEvent::CloseRequested => {
                    *control_flow = glutin::event_loop::ControlFlow::Exit;
                    return;
                },
                _ => return,
            },
            _ => (),
        }
    });
}

fn read_source(filename: &str) -> String {
    let mut s = String::new();
    File::open(filename).unwrap().read_to_string(&mut s).unwrap();
    s
}
