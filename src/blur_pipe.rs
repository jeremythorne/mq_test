use miniquad::*;
use mq_test::quad_verts;
use glam::{vec2, Vec2};

struct Node {
    pass:RenderPass,
    pipe:Pipeline,
    bind:Bindings,
    output:Texture    
}

impl Node {
    pub fn new(ctx: &mut Context,
        vertex_shader: &str,
        fragment_shader: &str,
        input:Texture) -> Node {

        let color_img = Texture::new_render_texture(
            ctx,
            TextureParams {
                width: 256,
                height: 256,
                format: TextureFormat::RGBA8,
                ..Default::default()
            },
        );

        let pass = RenderPass::new(ctx, color_img, None);

        let shader = Shader::new(
            ctx,
            vertex_shader,
            fragment_shader,
            meta(),
        )
        .unwrap();

        let pipe = Pipeline::new(
            ctx,
            &[BufferLayout::default()],
            &[
                VertexAttribute::new("pos", VertexFormat::Float2),
                VertexAttribute::new("uv", VertexFormat::Float2),
            ],
            shader,
        );

        let (vertices, indices) = quad_verts();
        let vertex_buffer = Buffer::immutable(ctx, BufferType::VertexBuffer, &vertices);
        let index_buffer = Buffer::immutable(ctx, BufferType::IndexBuffer, &indices);

        let bind = Bindings {
            vertex_buffers: vec![vertex_buffer],
            index_buffer: index_buffer,
            images: vec![input],
        };

        let output = color_img;

        Node {
            pass,
            pipe,
            bind,
            output
        }
    }

    pub fn draw(&self, ctx: &mut Context) {
        ctx.begin_pass(
            self.pass,
            PassAction::clear_color(0.0, 0.0, 0.0, 1.0),
        );
        ctx.apply_pipeline(&self.pipe);
        ctx.apply_bindings(&self.bind);
        let (w, h) = (256.0, 256.0);
        ctx.apply_uniforms(&Uniforms {
            resolution: vec2(1.0 / w, 1.0 / h),
        });
        ctx.draw(0, 6, 1);
        ctx.end_render_pass();
    }

    pub fn get_output(&self) -> Texture {
        self.output
    }}

pub struct BlurPipe {
    horiz:Node,
    vert:Node,
    output:Texture
}

impl BlurPipe {
    pub fn new(ctx: &mut Context, input:Texture) -> BlurPipe {
        let horiz = Node::new(ctx, VERTEX, HORIZ_FRAGMENT, input);
        let vert = Node::new(ctx, VERTEX, VERT_FRAGMENT, horiz.get_output());
        let output = vert.get_output();
        BlurPipe {
            horiz,
            vert,
            output
        }
    }

    pub fn draw(&self, ctx: &mut Context) {
        self.horiz.draw(ctx);
        self.vert.draw(ctx);
    }

    pub fn get_output(&self) -> Texture {
        self.output
    }
}

const VERTEX: &str = r#"#version 100
attribute vec2 pos;
attribute vec2 uv;

varying lowp vec2 texcoord;

void main() {
    gl_Position = vec4(pos, 0, 1);
    texcoord = uv;
}
"#;

pub const HORIZ_FRAGMENT: &str = r#"#version 100
precision lowp float;

varying vec2 texcoord;

uniform sampler2D tex;
uniform vec2 resolution;

void main() {
    float width = 3.0;
    vec4 acc = vec4(0.0);

    for (int i = 0; i <= 6; i++) {
        acc += texture2D(tex, texcoord + resolution * vec2(float(i) - width, 0.0));
    }
    gl_FragColor = acc / (2.0 * width);
}
"#;

pub const VERT_FRAGMENT: &str = r#"#version 100
precision lowp float;

varying vec2 texcoord;

uniform sampler2D tex;
uniform vec2 resolution;

void main() {
    float width = 3.0;
    vec4 acc = vec4(0.0);

    for (int i = 0; i <= 6; i++) {
        acc += texture2D(tex, texcoord + resolution * vec2(0.0, float(i) - width));
    }
    gl_FragColor = acc / (2.0 * width);
}
"#;

pub fn meta() -> ShaderMeta {
    ShaderMeta {
        images: vec!["tex".to_string()],
        uniforms: UniformBlockLayout {
            uniforms: vec![UniformDesc::new("resolution", UniformType::Float2),
            ]},
    }
}

#[repr(C)]
pub struct Uniforms {
    pub resolution: glam::Vec2,
}