use miniquad::*;
use glam::Mat4;
use crate::objects::{Object, ColouredObject};

pub struct MainPipe {
    pass:RenderPass,
    pipe:Pipeline,
    coloured_pipe:Pipeline,
    output:Texture
}

impl MainPipe {
    pub fn new(ctx: &mut Context) -> MainPipe {
        let (w, h) = ctx.screen_size();
        let color_img = Texture::new_render_texture(
            ctx,
            TextureParams {
                width: w as _,
                height: h as _,
                format: TextureFormat::RGBA8,
                ..Default::default()
            },
        );
        let depth_img = Texture::new_render_texture(
            ctx,
            TextureParams {
                width: w as _,
                height: h as _,
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
                VertexAttribute::new("color0", VertexFormat::Float4),
            ],
            shader,
            PipelineParams {
                depth_test: Comparison::LessOrEqual,
                depth_write: true,
                ..Default::default()
            },
        );

        let shader = Shader::new(
            ctx,
            COLOURED_VERTEX,
            COLOURED_FRAGMENT,
            coloured_meta(),
        )
        .unwrap();

        let coloured_pipe = Pipeline::with_params(
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
        MainPipe {
            pass,
            pipe,
            coloured_pipe,
            output: color_img
        }
    }

    pub fn resize(&mut self, ctx: &mut Context, width: f32, height: f32) {
        let color_img = Texture::new_render_texture(
            ctx,
            TextureParams {
                width: width as _,
                height: height as _,
                format: TextureFormat::RGBA8,
                ..Default::default()
            },
        );
        
        let depth_img = Texture::new_render_texture(
            ctx,
            TextureParams {
                width: width as _,
                height: height as _,
                format: TextureFormat::Depth,
                ..Default::default()
            },
        );

        let pass = RenderPass::new(ctx, color_img, depth_img);

        self.pass.delete(ctx);
        self.pass = pass;
        self.output = color_img;
    }

    pub fn draw(&self, ctx: &mut Context,
        bind: &Bindings,
        objects: &Vec<Object>,
        coloured_objects: &Vec<ColouredObject>,
        model: &Mat4, view_proj: &Mat4, light_view_proj: &Mat4) {
        ctx.begin_pass(
            self.pass,
            PassAction::clear_color(0.0, 0.0, 0.0, 0.0),
        );
        ctx.apply_pipeline(&self.pipe);
        ctx.apply_bindings(bind);
        for obj in objects.iter() {
            ctx.apply_uniforms(&Uniforms {
                mvp: *view_proj * *model * obj.model,
                light_mvp: *light_view_proj * *model * obj.model,
            });
            ctx.draw(obj.start, obj.end, 1);
        }
        ctx.apply_pipeline(&self.coloured_pipe);
        ctx.apply_bindings(bind);
        for cobj in coloured_objects.iter() {
            ctx.apply_uniforms(&ColouredUniforms {
                mvp: *view_proj * *model * cobj.object.model,
                colour: cobj.colour,
            });
            ctx.draw(cobj.object.start, cobj.object.end, 1);
        }
        ctx.end_render_pass();
    }

    pub fn get_output(&self) -> Texture {
        self.output
    }
}

const VERTEX: &str = r#"#version 100
attribute vec4 pos;
attribute vec4 color0;

varying vec4 color;
varying vec4 light_pos;

uniform mat4 mvp;
uniform mat4 light_mvp;

void main() {
    gl_Position = mvp * pos;
    light_pos = light_mvp * pos;
    color = color0;
}
"#;

const FRAGMENT: &str = r#"#version 100

precision mediump float;

varying vec4 color;
varying vec4 light_pos;

uniform sampler2D shadow_map;

void main() {
    float ambient = 0.0;
    float c = 4.0;
    vec2 light_uv = (light_pos.xy / light_pos.w) * 0.5 + 0.5;
    vec4 texel = texture2D(shadow_map, light_uv);
    float light_depth = light_pos.z / 100.0;
    float shadow = clamp(exp(-c * (light_depth - texel.r)), 0.0, 1.0);
    gl_FragColor = vec4(1.0) * clamp(ambient + shadow, 0.0, 1.0);
}
"#;

fn meta() -> ShaderMeta {
    ShaderMeta {
        images: vec!["shadow_map".to_string()],
        uniforms: UniformBlockLayout {
            uniforms: vec![
                UniformDesc::new("mvp", UniformType::Mat4),
                UniformDesc::new("light_mvp", UniformType::Mat4)
            ]
        },
    }
}

#[repr(C)]
pub struct Uniforms {
    pub mvp: glam::Mat4,
    pub light_mvp: glam::Mat4,
}

const COLOURED_VERTEX: &str = r#"#version 100
attribute vec4 pos;

uniform mat4 mvp;

void main() {
    gl_Position = mvp * pos;
}
"#;

const COLOURED_FRAGMENT: &str = r#"#version 100

precision mediump float;

uniform vec4 colour;

void main() {
    gl_FragColor = colour;
}
"#;

fn coloured_meta() -> ShaderMeta {
    ShaderMeta {
        images: vec![],
        uniforms: UniformBlockLayout {
            uniforms: vec![
                UniformDesc::new("mvp", UniformType::Mat4),
                UniformDesc::new("colour", UniformType::Float4),
            ]
        },
    }
}

#[repr(C)]
pub struct ColouredUniforms {
    pub mvp: glam::Mat4,
    pub colour: glam::Vec4,
}