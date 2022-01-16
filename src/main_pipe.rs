use miniquad::*;
use glam::{Vec4, Mat3, Mat4};
use crate::objects::{Object, ColouredObject};

pub struct MainPipe {
    pass:RenderPass,
    pipe:Pipeline,
    coloured_pipe:Pipeline,
    output:Texture
}

fn normal_matrix(model:Mat4) -> Mat4 {
    // normal matrix calculation from
    // https://www.lighthouse3d.com/tutorials/glsl-12-tutorial/the-normal-matrix/ 
    Mat4::from_mat3(
        Mat3::from_mat4(model)
            .inverse().transpose()
    )
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
                stride: 48,
                ..Default::default()
            }],
            &[
                VertexAttribute::new("pos", VertexFormat::Float3),
                VertexAttribute::new("color0", VertexFormat::Float4),
                VertexAttribute::new("normal", VertexFormat::Float3),
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
                stride: 48,
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
        scene_model: &Mat4, light_pos: Vec4, view_proj: &Mat4, light_view: &Mat4,
        light_proj: &Mat4) {
        ctx.begin_pass(
            self.pass,
            PassAction::clear_color(0.0, 0.0, 0.0, 0.0),
        );
        ctx.apply_pipeline(&self.pipe);
        ctx.apply_bindings(bind);
        for obj in objects.iter() {
            let model = *scene_model * obj.model;
            let normal_matrix = normal_matrix(model);
            ctx.apply_uniforms(&Uniforms {
                model,
                proj: *view_proj,
                normal_matrix,
                light_pos,
                light_mv: *light_view * model,
                light_proj: *light_proj,
            });
            ctx.draw(obj.start, obj.end, 1);
        }
        ctx.apply_pipeline(&self.coloured_pipe);
        ctx.apply_bindings(bind);
        for cobj in coloured_objects.iter() {
            ctx.apply_uniforms(&ColouredUniforms {
                mvp: *view_proj * *scene_model * cobj.object.model,
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
attribute vec3 normal;
attribute vec4 color0;

varying vec3 vlight_dir;
varying vec3 vnormal_view;
varying vec4 vpos_from_light;
varying vec4 vshadow_coord;

uniform mat4 model;
uniform mat4 proj;
uniform mat4 normal_matrix;
uniform vec4 light_pos;
uniform mat4 light_proj;
uniform mat4 light_mv;

void main() {
    vec4 position = model * pos;
    gl_Position = proj * position;
    vpos_from_light = light_mv * pos;
    vshadow_coord = light_proj * vpos_from_light;
    vnormal_view = (normal_matrix * vec4(normal, 0.0)).xyz;
    vlight_dir = (light_pos - position).xyz;
}
"#;

const FRAGMENT: &str = r#"#version 100

precision mediump float;

varying vec3 vlight_dir;
varying vec3 vnormal_view;
varying vec4 vpos_from_light;
varying vec4 vshadow_coord;

uniform sampler2D shadow_map;

void main() {
    float ambient = 0.0;

    float c = 40.0;
    vec2 shadow_uv = (vshadow_coord.xy / vshadow_coord.w) * 0.5 + 0.5;
    float map_depth = texture2D(shadow_map, shadow_uv).x;
    float light_depth = vshadow_coord.z / vshadow_coord.w;
    float shadow = clamp(exp(-c * (light_depth - map_depth)), 0.0, 1.0);

    float lambert = max(0.0, dot(normalize(vlight_dir), normalize(vnormal_view)));

    gl_FragColor = vec4(1.0) * clamp(ambient + lambert * shadow, 0.0, 1.0);
}
"#;

fn meta() -> ShaderMeta {
    ShaderMeta {
        images: vec!["shadow_map".to_string()],
        uniforms: UniformBlockLayout {
            uniforms: vec![
                UniformDesc::new("model", UniformType::Mat4),
                UniformDesc::new("proj", UniformType::Mat4),
                UniformDesc::new("normal_matrix", UniformType::Mat4),
                UniformDesc::new("light_pos", UniformType::Float4),
                UniformDesc::new("light_mv", UniformType::Mat4),
                UniformDesc::new("light_proj", UniformType::Mat4),
            ]
        },
    }
}

#[repr(C)]
pub struct Uniforms {
    pub model: glam::Mat4,
    pub proj: glam::Mat4,
    pub normal_matrix: glam::Mat4,
    pub light_pos: glam::Vec4,
    pub light_mv: glam::Mat4,
    pub light_proj: glam::Mat4,
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