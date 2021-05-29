use miniquad::*;
use glam::Mat4;
use crate::objects::{Object, ColouredObject};
use crate::blur_pipe::BlurPipe;

pub struct GlowPipe {
    pass:RenderPass,
    pipe:Pipeline,
    blur_pipe:BlurPipe,
    output:Texture
}

impl GlowPipe {
    pub fn new(ctx: &mut Context) -> GlowPipe {
        let color_img = Texture::new_render_texture(
            ctx,
            TextureParams {
                width: 256,
                height: 256,
                format: TextureFormat::RGBA8,
                ..Default::default()
            },
        );
        let depth_img = Texture::new_render_texture(
            ctx,
            TextureParams {
                width: 256,
                height: 256,
                format: TextureFormat::Depth,
                ..Default::default()
            },
        );

        let pass = RenderPass::new(ctx, color_img, depth_img);

        let shader = Shader::new(
            ctx,
            VERTEX,
            FRAGMENT,
            meta(),
        )
        .unwrap();

        let pipe = Pipeline::with_params(
            ctx,
            &[BufferLayout {
                stride: 36,
                ..Default::default()
            }],
            &[
                VertexAttribute::new("pos", VertexFormat::Float3),
            ],
            shader,
            PipelineParams {
                depth_test: Comparison::LessOrEqual,
                depth_write: true,
                ..Default::default()
            },
        );

        let blur_pipe = BlurPipe::new(ctx, color_img);
        let output = blur_pipe.get_output();

        GlowPipe {
            pass,
            pipe,
            blur_pipe,
            output
        }
    }

    pub fn draw(&self, ctx: &mut Context,
        bind: &Bindings,
        objects: &Vec<Object>, 
        coloured_objects: &Vec<ColouredObject>,
        model: &Mat4, view_proj: &Mat4) {
        ctx.begin_pass(
            self.pass,
            PassAction::clear_color(0.0, 0.0, 0.0, 1.0),
        );
        ctx.apply_pipeline(&self.pipe);
        ctx.apply_bindings(bind);
        for obj in objects.iter() {
            ctx.apply_uniforms(&Uniforms {
                mvp: *view_proj * *model * obj.model,
                colour: glam::vec4(0., 0., 0., 1.0)
            });
            ctx.draw(obj.start, obj.end, 1);
        }
        for obj in coloured_objects.iter() {
            ctx.apply_uniforms(&Uniforms {
                mvp: *view_proj * *model * obj.object.model,
                colour: obj.colour
            });
            ctx.draw(obj.object.start, obj.object.end, 1);
        }
        ctx.end_render_pass();
        self.blur_pipe.draw(ctx);
    }

    pub fn get_output(&self) -> Texture {
        self.output
    }
}

pub const VERTEX: &str = r#"#version 100
attribute vec4 pos;

uniform mat4 mvp;

void main() {
    gl_Position = mvp * pos;
}
"#;

pub const FRAGMENT: &str = r#"#version 100

precision mediump float;
uniform vec4 colour;

void main() {
    gl_FragColor = colour;
}
"#;

pub fn meta() -> ShaderMeta {
    ShaderMeta {
        images: vec![],
        uniforms: UniformBlockLayout {
            uniforms: vec![UniformDesc::new("mvp", UniformType::Mat4),
                           UniformDesc::new("colour", UniformType::Float4)
            ],
        },
    }
}

#[repr(C)]
pub struct Uniforms {
    pub mvp: glam::Mat4,
    pub colour: glam::Vec4,
}