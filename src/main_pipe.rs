use miniquad::*;
use crate::*;

use glam;

pub fn pipe(ctx: &mut Context, shadow_map:Texture) -> PipeBind {
    let (vertices, indices) = cube_verts();
    let vertex_buffer = Buffer::immutable(ctx, BufferType::VertexBuffer, &vertices);
    let index_buffer = Buffer::immutable(ctx, BufferType::IndexBuffer, &indices);

    let bind = Bindings {
        vertex_buffers: vec![vertex_buffer],
        index_buffer: index_buffer,
        images: vec![shadow_map],
    };

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

    PipeBind {
        pipe,
        bind
    }
}

const VERTEX: &str = r#"#version 100
attribute vec4 pos;
attribute vec4 color0;

varying vec4 color;
varying vec2 light_uv;
varying float light_depth;


uniform mat4 mvp;
uniform mat4 light_mvp;

void main() {
    gl_Position = mvp * pos;
    vec4 light_pos = light_mvp * pos;
    light_uv = (light_pos.xy / light_pos.w) * 0.5 + 0.5;
    light_depth = light_pos.z / 100.0;
    color = color0;
}
"#;

const FRAGMENT: &str = r#"#version 100

precision mediump float;

varying vec4 color;
varying vec2 light_uv;
varying float light_depth;

uniform sampler2D shadow_map;

void main() {
    float ambient = 0.2;
    float light = 0.8;
    if (texture2D(shadow_map, light_uv).r < light_depth) {
        light = 0.0;
    }
    //light = texture2D(shadow_map, light_uv).r;
    gl_FragColor = color * (ambient + light);
    //gl_FragColor = vec4(vec3(light), 1.0);
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
