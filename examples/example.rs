extern crate bmfont;
#[macro_use]
extern crate glium;
extern crate image;

use bmfont::{BMFont, OrdinateOrientation};
use glium::{Display, Program, VertexBuffer};
use std::io::Cursor;

fn create_program(display: &Display) -> Program {
    let vertex_shader_src = r#"
        #version 140

        in vec2 position;
        in vec2 tex_coords;
        out vec2 v_tex_coords;

        void main() {
            v_tex_coords = tex_coords;
            gl_Position = vec4(position, 0.0, 1.0);
        }
    "#;
    let fragment_shader_src = r#"
        #version 140

        in vec2 v_tex_coords;
        out vec4 color;

uniform sampler2D tex;

        void main() {
            color = texture(tex, v_tex_coords);
        }
    "#;
    Program::from_source(display, vertex_shader_src, fragment_shader_src, None).unwrap()
}

fn main() {
    use glium::{DisplayBuild, DrawParameters, Surface};
    let display = glium::glutin::WindowBuilder::new().build_glium().unwrap();

    let image = image::load(Cursor::new(&include_bytes!("../font.png")[..]), image::PNG)
        .unwrap()
        .to_rgba();
    let image_dimensions = image.dimensions();
    println!("{:?}", image_dimensions);
    let image = glium::texture::RawImage2d::from_raw_rgba(image.into_raw(), image_dimensions);
    let texture = glium::texture::Texture2d::new(&display, image).unwrap();

    #[derive(Copy, Clone, Debug)]
    struct Vertex {
        position: [f32; 2],
        tex_coords: [f32; 2],
    }

    implement_vertex!(Vertex, position, tex_coords);
    let design_size = (1024.0, 768.0);
    let bmfont = BMFont::new(
        Cursor::new(&include_bytes!("../font.fnt")[..]),
        OrdinateOrientation::BottomToTop,
    )
    .unwrap();
    let char_positions = bmfont.parse("Hello\nmy\nfriend").unwrap();
    let shapes = char_positions.into_iter().map(|char_position| {
        let left_page_x = char_position.page_rect.x as f32 / image_dimensions.0 as f32;
        let right_page_x = char_position.page_rect.max_x() as f32 / image_dimensions.0 as f32;
        let top_page_y = char_position.page_rect.y as f32 / image_dimensions.1 as f32;
        let bottom_page_y = char_position.page_rect.max_y() as f32 / image_dimensions.1 as f32;

        let left_screen_x = char_position.screen_rect.x as f32 / design_size.0 as f32;
        let right_screen_x = char_position.screen_rect.max_x() as f32 / design_size.0 as f32;
        let bottom_screen_y = char_position.screen_rect.y as f32 / design_size.1 as f32;
        let top_screen_y = char_position.screen_rect.max_y() as f32 / design_size.1 as f32;
        vec![
            Vertex {
                position: [left_screen_x, bottom_screen_y],
                tex_coords: [left_page_x, bottom_page_y],
            },
            Vertex {
                position: [left_screen_x, top_screen_y],
                tex_coords: [left_page_x, top_page_y],
            },
            Vertex {
                position: [right_screen_x, top_screen_y],
                tex_coords: [right_page_x, top_page_y],
            },
            Vertex {
                position: [left_screen_x, bottom_screen_y],
                tex_coords: [left_page_x, bottom_page_y],
            },
            Vertex {
                position: [right_screen_x, top_screen_y],
                tex_coords: [right_page_x, top_page_y],
            },
            Vertex {
                position: [right_screen_x, bottom_screen_y],
                tex_coords: [right_page_x, bottom_page_y],
            },
        ]
    });
    let mut vertex_buffers = Vec::new();
    for shape in shapes {
        vertex_buffers.push(VertexBuffer::new(&display, &shape).unwrap());
    }
    let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);
    let program = create_program(&display);
    let uniforms = uniform! { tex: &texture };
    let draw_parameters = DrawParameters {
        blend: glium::draw_parameters::Blend::alpha_blending(),
        ..DrawParameters::default()
    };

    loop {
        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 1.0, 1.0);

        for vertex_buffer in &vertex_buffers {
            target
                .draw(
                    vertex_buffer,
                    &indices,
                    &program,
                    &uniforms,
                    &draw_parameters,
                )
                .unwrap();
        }
        target.finish().unwrap();

        for ev in display.poll_events() {
            match ev {
                glium::glutin::Event::Closed => return,
                _ => (),
            }
        }
    }
}
