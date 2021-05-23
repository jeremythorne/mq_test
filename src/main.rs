use miniquad::*;

use glam::{vec3, Mat4, EulerRot};
use quad_rand as qrand;

struct PipeBind {
    pipe: Pipeline,
    bind: Bindings,
}

struct Stage {
    blur: PipeBind,
    copy: PipeBind,
    depth_view: PipeBind,
    depth_write: PipeBind,
    main: PipeBind,
    offscreen_pass: RenderPass,
    depth_write_pass: RenderPass,
    cubes: Vec<Mat4>,
    rx: f32,
    ry: f32,
}

fn cube_verts() -> (&'static[f32], &'static[u16]) {
    #[rustfmt::skip]
    let vertices: &[f32] = &[
        /* pos               color                   uvs */
        -1.0, -1.0, -1.0,    1.0, 0.5, 0.5, 1.0,     0.0, 0.0,
        1.0, -1.0, -1.0,    1.0, 0.5, 0.5, 1.0,     1.0, 0.0,
        1.0,  1.0, -1.0,    1.0, 0.5, 0.5, 1.0,     1.0, 1.0,
        -1.0,  1.0, -1.0,    1.0, 0.5, 0.5, 1.0,     0.0, 1.0,

        -1.0, -1.0,  1.0,    0.5, 1.0, 0.5, 1.0,     0.0, 0.0,
        1.0, -1.0,  1.0,    0.5, 1.0, 0.5, 1.0,     1.0, 0.0,
        1.0,  1.0,  1.0,    0.5, 1.0, 0.5, 1.0,     1.0, 1.0,
        -1.0,  1.0,  1.0,    0.5, 1.0, 0.5, 1.0,     0.0, 1.0,

        -1.0, -1.0, -1.0,    0.5, 0.5, 1.0, 1.0,     0.0, 0.0,
        -1.0,  1.0, -1.0,    0.5, 0.5, 1.0, 1.0,     1.0, 0.0,
        -1.0,  1.0,  1.0,    0.5, 0.5, 1.0, 1.0,     1.0, 1.0,
        -1.0, -1.0,  1.0,    0.5, 0.5, 1.0, 1.0,     0.0, 1.0,

        1.0, -1.0, -1.0,    1.0, 0.5, 0.0, 1.0,     0.0, 0.0,
        1.0,  1.0, -1.0,    1.0, 0.5, 0.0, 1.0,     1.0, 0.0,
        1.0,  1.0,  1.0,    1.0, 0.5, 0.0, 1.0,     1.0, 1.0,
        1.0, -1.0,  1.0,    1.0, 0.5, 0.0, 1.0,     0.0, 1.0,

        -1.0, -1.0, -1.0,    0.0, 0.5, 1.0, 1.0,     0.0, 0.0,
        -1.0, -1.0,  1.0,    0.0, 0.5, 1.0, 1.0,     1.0, 0.0,
        1.0, -1.0,  1.0,    0.0, 0.5, 1.0, 1.0,     1.0, 1.0,
        1.0, -1.0, -1.0,    0.0, 0.5, 1.0, 1.0,     0.0, 1.0,

        -1.0,  1.0, -1.0,    1.0, 0.0, 0.5, 1.0,     0.0, 0.0,
        -1.0,  1.0,  1.0,    1.0, 0.0, 0.5, 1.0,     1.0, 0.0,
        1.0,  1.0,  1.0,    1.0, 0.0, 0.5, 1.0,     1.0, 1.0,
        1.0,  1.0, -1.0,    1.0, 0.0, 0.5, 1.0,     0.0, 1.0
    ];

    #[rustfmt::skip]
    let indices: &[u16] = &[
        0, 1, 2,  0, 2, 3,
        6, 5, 4,  7, 6, 4,
        8, 9, 10,  8, 10, 11,
        14, 13, 12,  15, 14, 12,
        16, 17, 18,  16, 18, 19,
        22, 21, 20,  23, 22, 20
    ];

    (vertices, indices)
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

fn depth_write_pipe(ctx: &mut Context) -> PipeBind {
    let (vertices, indices) = cube_verts();
    let vertex_buffer = Buffer::immutable(ctx, BufferType::VertexBuffer, &vertices);
    let index_buffer = Buffer::immutable(ctx, BufferType::IndexBuffer, &indices);

    let bind = Bindings {
        vertex_buffers: vec![vertex_buffer],
        index_buffer: index_buffer,
        images: vec![],
    };

    let shader = Shader::new(
        ctx,
        depth_write_shader::VERTEX,
        depth_write_shader::FRAGMENT,
        depth_write_shader::meta(),
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
            depth_test: Comparison::Equal,
            depth_write: true, // TODO we don't want writes on here only reads
            ..Default::default()
        },
    );

    PipeBind {
        pipe,
        bind
    }
}

fn main_pipe(ctx: &mut Context) -> PipeBind {
    let (vertices, indices) = cube_verts();
    let vertex_buffer = Buffer::immutable(ctx, BufferType::VertexBuffer, &vertices);
    let index_buffer = Buffer::immutable(ctx, BufferType::IndexBuffer, &indices);

    let bind = Bindings {
        vertex_buffers: vec![vertex_buffer],
        index_buffer: index_buffer,
        images: vec![],
    };

    let shader = Shader::new(
        ctx,
        main_shader::VERTEX,
        main_shader::FRAGMENT,
        main_shader::meta(),
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
impl Stage {
    pub fn new(ctx: &mut Context) -> Stage {
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
        let depth_write_img = Texture::new_render_texture(
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

        let offscreen_pass = RenderPass::new(ctx, color_img, depth_img);
        let depth_write_pass = RenderPass::new(ctx, depth_write_img, depth_img);

        let mut cubes = Vec::<Mat4>::new();
        for _ in 0..40 {
            let r = qrand::gen_range(0., 1.);
            let rot = Mat4::from_euler(EulerRot::YXZ, 
                qrand::gen_range(-std::f32::consts::PI, std::f32::consts::PI),
                0.0, 0.);
            let s = (1.4 - r) * qrand::gen_range(0.7, 0.9);
            let scale = Mat4::from_scale(vec3(s, s, s));
            let trans = Mat4::from_translation(vec3(
                6.0 * r,
                s - 1.0,
                0.0,
            ));
            cubes.push(rot * trans * scale);
        }

        let blur = blur_pipe(ctx, color_img);
        let copy = copy_pipe(ctx, color_img);
        let depth_view = depth_view_pipe(ctx, depth_write_img);
        let depth_write = depth_write_pipe(ctx);
        let main = main_pipe(ctx);

        Stage {
            blur,
            copy,
            depth_view,
            depth_write,
            main,
            offscreen_pass,
            depth_write_pass,
            cubes,
            rx: 0.,
            ry: 0.,
        }
    }
}

impl EventHandler for Stage {
    fn update(&mut self, _ctx: &mut Context) {}

    fn resize_event(&mut self, ctx: &mut Context, width: f32, height: f32) {
        let color_img = Texture::new_render_texture(
            ctx,
            TextureParams {
                width: width as _,
                height: height as _,
                format: TextureFormat::RGBA8,
                ..Default::default()
            },
        );
        let depth_write_img = Texture::new_render_texture(
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

        let offscreen_pass = RenderPass::new(ctx, color_img, depth_img);
        let depth_write_pass = RenderPass::new(ctx, depth_write_img, depth_img);

        self.offscreen_pass.delete(ctx);
        self.offscreen_pass = offscreen_pass;
        self.depth_write_pass.delete(ctx);
        self.depth_write_pass = depth_write_pass;
        self.copy.bind.images[0] = color_img;
        self.blur.bind.images[0] = color_img;
        self.depth_view.bind.images[0] = depth_write_img;
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

        //self.rx += 0.01;
        self.ry += 0.01;
        let model = Mat4::from_euler(EulerRot::YXZ, self.ry, self.rx, 0.);

        let (w, h) = ctx.screen_size();
        // the offscreen pass, rendering rotating, untextured cubes into a render target image
        ctx.begin_pass(
            self.offscreen_pass,
            PassAction::clear_color(1.0, 1.0, 1.0, 1.0),
        );
        ctx.apply_pipeline(&self.main.pipe);
        ctx.apply_bindings(&self.main.bind);
        for &cube in self.cubes.iter() {
            ctx.apply_uniforms(&main_shader::Uniforms {
                mvp: view_proj * model * cube,
            });
            ctx.draw(0, 36, 1);
        }
        ctx.end_render_pass();

        // also write the depth (for future shadow map)
        ctx.begin_pass(
            self.depth_write_pass,
            PassAction::Clear {
                color: Some((0.0, 0.0, 0.0, 0.0)),
                depth: None,
                stencil: None,
            }
        );
        ctx.apply_pipeline(&self.depth_write.pipe);
        ctx.apply_bindings(&self.depth_write.bind);
        for &cube in self.cubes.iter() {
            ctx.apply_uniforms(&depth_write_shader::Uniforms {
                mvp: view_proj * model * cube,
                mv: model * cube,
            });
            ctx.draw(0, 36, 1);
        }
        ctx.end_render_pass();

        // and the post-processing-pass, rendering a quad, using the
        // previously rendered offscreen render-target as texture
        ctx.begin_default_pass(PassAction::Nothing);
        ctx.apply_pipeline(&self.depth_view.pipe);
        ctx.apply_bindings(&self.depth_view.bind);
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

mod depth_write_shader {
    use miniquad::*;

    pub const VERTEX: &str = r#"#version 100
    attribute vec4 pos;

    varying lowp vec4 vpos;

    uniform mat4 mvp;
    uniform mat4 mv;

    void main() {
        gl_Position = mvp * pos;
        vpos = mv * pos;
    }
    "#;

    pub const FRAGMENT: &str = r#"#version 100

    varying lowp vec4 vpos;

    void main() {
        gl_FragColor = vec4(vec3(vpos.z / 10.0), 1.0);
    }
    "#;

    pub fn meta() -> ShaderMeta {
        ShaderMeta {
            images: vec![],
            uniforms: UniformBlockLayout {
                uniforms: vec![UniformDesc::new("mvp", UniformType::Mat4),
                    UniformDesc::new("mv", UniformType::Mat4)
                ],
            },
        }
    }

    #[repr(C)]
    pub struct Uniforms {
        pub mvp: glam::Mat4,
        pub mv: glam::Mat4,
    }
}

mod main_shader {
    use miniquad::*;

    pub const VERTEX: &str = r#"#version 100
    attribute vec4 pos;
    attribute vec4 color0;

    varying lowp vec4 color;

    uniform mat4 mvp;

    void main() {
        gl_Position = mvp * pos;
        color = color0;
    }
    "#;

    pub const FRAGMENT: &str = r#"#version 100

    varying lowp vec4 color;

    void main() {
        gl_FragColor = color;
    }
    "#;

    pub fn meta() -> ShaderMeta {
        ShaderMeta {
            images: vec![],
            uniforms: UniformBlockLayout {
                uniforms: vec![UniformDesc::new("mvp", UniformType::Mat4)],
            },
        }
    }

    #[repr(C)]
    pub struct Uniforms {
        pub mvp: glam::Mat4,
    }
}