use miniquad::*;

use glam::{vec3, Mat4, EulerRot};

mod main_pipe;
mod shadow_pipe;
mod objects;

use main_pipe::MainPipe;
use shadow_pipe::ShadowPipe;
use objects::Object;

struct PipeBind {
    pipe: Pipeline,
    bind: Bindings
}

struct Stage {
    shadow_map: ShadowPipe,
    shadow_map_bind: Bindings,
    main: MainPipe,
    main_bind: Bindings,
    copy: PipeBind,
    objects: Vec<Object>,
    rx: f32,
    ry: f32,
}

fn quad_verts() -> (&'static[f32], &'static[u16]) {
    #[rustfmt::skip]
    let vertices: &[f32] = &[
        /* pos         uvs */
        -1.0, -1.0,    0.0, 0.0,
        1.0, -1.0,    1.0, 0.0,
        1.0,  1.0,    1.0, 1.0,
        -1.0,  1.0,    0.0, 1.0,
    ];
    let indices: &[u16] = &[0, 1, 2, 0, 2, 3];
    (vertices, indices)
}

fn copy_pipe(ctx: &mut Context, tex:Texture) -> PipeBind {
    let (vertices, indices) = quad_verts();
    let vertex_buffer = Buffer::immutable(ctx, BufferType::VertexBuffer, &vertices);
    let index_buffer = Buffer::immutable(ctx, BufferType::IndexBuffer, &indices);

    let bind = Bindings {
        vertex_buffers: vec![vertex_buffer],
        index_buffer: index_buffer,
        images: vec![tex],
    };

    let shader = Shader::new(
        ctx,
        copy_to_screen_shader::VERTEX,
        copy_to_screen_shader::FRAGMENT,
        copy_to_screen_shader::meta(),
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

    PipeBind {
        pipe,
        bind
    }
}

fn blur_pipe(ctx: &mut Context, tex:Texture) -> PipeBind {
    let (vertices, indices) = quad_verts();
    let vertex_buffer = Buffer::immutable(ctx, BufferType::VertexBuffer, &vertices);
    let index_buffer = Buffer::immutable(ctx, BufferType::IndexBuffer, &indices);

    let bind = Bindings {
        vertex_buffers: vec![vertex_buffer],
        index_buffer: index_buffer,
        images: vec![tex],
    };

    let shader = Shader::new(
        ctx,
        blur_shader::VERTEX,
        blur_shader::FRAGMENT,
        blur_shader::meta(),
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

    PipeBind {
        pipe,
        bind
    }
}

fn depth_view_pipe(ctx: &mut Context, tex:Texture) -> PipeBind {
    let (vertices, indices) = quad_verts();
    let vertex_buffer = Buffer::immutable(ctx, BufferType::VertexBuffer, &vertices);
    let index_buffer = Buffer::immutable(ctx, BufferType::IndexBuffer, &indices);

    let bind = Bindings {
        vertex_buffers: vec![vertex_buffer],
        index_buffer: index_buffer,
        images: vec![tex],
    };

    let shader = Shader::new(
        ctx,
        depth_view_shader::VERTEX,
        depth_view_shader::FRAGMENT,
        depth_view_shader::meta(),
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

    PipeBind {
        pipe,
        bind
    }
}

impl Stage {
    pub fn new(ctx: &mut Context) -> Stage {
        let (bind, objects) = objects::cubes(ctx);

        let shadow_map = ShadowPipe::new(ctx);
        let shadow_map_bind = bind.clone();

        let main = MainPipe::new(ctx);
        let mut main_bind = bind.clone();
        main_bind.images.push(shadow_map.get_output());

        let copy = copy_pipe(ctx, main.get_output());
 
        Stage {
            shadow_map,
            shadow_map_bind,
            main,
            main_bind,
            copy,
            objects,
            rx: 0.,
            ry: 0.,
        }
    }
}

impl EventHandler for Stage {
    fn update(&mut self, _ctx: &mut Context) {}

    fn resize_event(&mut self, ctx: &mut Context, width: f32, height: f32) {
        self.main.resize(ctx, width, height);
        self.copy.bind.images[0] = self.main.get_output();
    }

    fn draw(&mut self, ctx: &mut Context) {
        let (width, height) = ctx.screen_size();
        let proj = Mat4::perspective_rh_gl(60.0f32.to_radians(), width / height, 0.01, 20.0);
        let view = Mat4::look_at_rh(
            vec3(0.0, 1.5, 10.0),
            vec3(0.0, 0.0, 0.0),
            vec3(0.0, 1.0, 0.0),
        );
        let view_proj = proj * view;

        let proj = Mat4::perspective_rh_gl(60.0f32.to_radians(), 1.0, 10.0, 20.0);
        let light_view = Mat4::look_at_rh(
            vec3(10.0, 10.0, 10.0),
            vec3(0.0, 0.0, 0.0),
            vec3(0.0, 1.0, 0.0),
        );
        let light_view_proj = proj * light_view;
 
        //self.rx += 0.01;
        self.ry += 0.01;
        let model = Mat4::from_euler(EulerRot::YXZ, self.ry, self.rx, 0.);

        let (w, h) = ctx.screen_size();
        
        self.shadow_map.draw(ctx, &self.shadow_map_bind, &self.objects, &model, &light_view_proj);
        self.main.draw(ctx, &self.main_bind, &self.objects, &model, &view_proj, &light_view_proj);

        // and the post-processing-pass, rendering a quad, using the
        // previously rendered offscreen render-target as texture
        ctx.begin_default_pass(PassAction::Nothing);
        ctx.apply_pipeline(&self.copy.pipe);
        ctx.apply_bindings(&self.copy.bind);
        //ctx.apply_uniforms(&copy_to_screen_shader::Uniforms {
        //    resolution: glam::vec2(w, h),
        //});
        ctx.draw(0, 6, 1);
        ctx.end_render_pass();
        ctx.commit_frame();
    }
}

fn main() {
    miniquad::start(conf::Conf::default(), |mut ctx| {
        UserData::owning(Stage::new(&mut ctx), ctx)
    });
}

mod copy_to_screen_shader {
    use miniquad::*;

    pub const VERTEX: &str = r#"#version 100
    attribute vec2 pos;
    attribute vec2 uv;

    varying lowp vec2 texcoord;

    void main() {
        gl_Position = vec4(pos, 0, 1);
        texcoord = uv;
    }
    "#;

    pub const FRAGMENT: &str = r#"#version 100
    precision lowp float;

    varying vec2 texcoord;

    uniform sampler2D tex;

    void main() {
        gl_FragColor = texture2D(tex, texcoord);
    }
    "#;

    pub fn meta() -> ShaderMeta {
        ShaderMeta {
            images: vec!["tex".to_string()],
            uniforms: UniformBlockLayout {
                uniforms: vec![],
            },
        }
    }
}

mod blur_shader {
    use miniquad::*;

    pub const VERTEX: &str = r#"#version 100
    attribute vec2 pos;
    attribute vec2 uv;

    varying lowp vec2 texcoord;

    void main() {
        gl_Position = vec4(pos, 0, 1);
        texcoord = uv;
    }
    "#;

    pub const FRAGMENT: &str = r#"#version 100
    precision lowp float;

    varying vec2 texcoord;

    uniform sampler2D tex;
    uniform vec2 resolution;



    // Source: https://github.com/Jam3/glsl-fast-gaussian-blur/blob/master/5.glsl
    vec4 blur5(sampler2D image, vec2 uv, vec2 resolution, vec2 direction) {
        vec4 color = vec4(0.0);
        vec2 off1 = vec2(1.3333333333333333) * direction;
        color += texture2D(image, uv) * 0.29411764705882354;
        color += texture2D(image, uv + (off1 / resolution)) * 0.35294117647058826;
        color += texture2D(image, uv - (off1 / resolution)) * 0.35294117647058826;
        return color;
    }

    void main() {
        gl_FragColor = blur5(tex, texcoord, resolution, vec2(3.0));
    }
    "#;

    pub fn meta() -> ShaderMeta {
        ShaderMeta {
            images: vec!["tex".to_string()],
            uniforms: UniformBlockLayout {
                uniforms: vec![UniformDesc::new("resolution", UniformType::Float2)],
            },
        }
    }

    #[repr(C)]
    pub struct Uniforms {
        pub resolution: glam::Vec2,
    }
}

mod depth_view_shader {
    use miniquad::*;

    pub const VERTEX: &str = r#"#version 100
    attribute vec2 pos;
    attribute vec2 uv;

    varying lowp vec2 texcoord;

    void main() {
        gl_Position = vec4(pos, 0, 1);
        texcoord = uv;
    }
    "#;

    pub const FRAGMENT: &str = r#"#version 100
    precision lowp float;

    varying vec2 texcoord;

    uniform sampler2D tex;

    void main() {
        vec3 depth = vec3(texture2D(tex, texcoord).r);
        gl_FragColor = vec4(1.0 - depth, 1.0);
    }
    "#;

    pub fn meta() -> ShaderMeta {
        ShaderMeta {
            images: vec!["tex".to_string()],
            uniforms: UniformBlockLayout {
                uniforms: vec![],
            },
        }
    }
}

