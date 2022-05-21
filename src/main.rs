use miniquad::*;

use glam::{vec3, Mat4, EulerRot};

mod blur_pipe;
mod blur_shadow_pipe;
mod main_pipe;
mod shadow_pipe;
mod glow_pipe;
mod objects;

use main_pipe::MainPipe;
use shadow_pipe::ShadowPipe;
use glow_pipe::GlowPipe;
use objects::{Object, ColouredObject};
use mq_test::quad_verts;

struct PipeBind {
    pipe: Pipeline,
    bind: Bindings
}

struct Stage {
    shadow_map: ShadowPipe,
    shadow_map_bind: Bindings,
    main: MainPipe,
    main_bind: Bindings,
    glow: GlowPipe,
    glow_bind: Bindings,
    copy: PipeBind,
    depth_view: PipeBind,
    glow_blend: PipeBind,
    objects: Vec<Object>,
    coloured_objects: Vec<ColouredObject>,
    rx: f32,
    ry: f32,
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

fn glow_blend_pipe(ctx: &mut Context,
        main:Texture, glow:Texture) -> PipeBind {
    let (vertices, indices) = quad_verts();
    let vertex_buffer = Buffer::immutable(ctx, BufferType::VertexBuffer, &vertices);
    let index_buffer = Buffer::immutable(ctx, BufferType::IndexBuffer, &indices);

    let bind = Bindings {
        vertex_buffers: vec![vertex_buffer],
        index_buffer: index_buffer,
        images: vec![main, glow],
    };

    let shader = Shader::new(
        ctx,
        glow_blend_shader::VERTEX,
        glow_blend_shader::FRAGMENT,
        glow_blend_shader::meta(),
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
        let objects = objects::cubes();
        let coloured_objects = objects::coloured_cubes();
        let bind = objects::cube_bindings(ctx);

        let shadow_map = ShadowPipe::new(ctx);
        let shadow_map_bind = bind.clone();

        let main = MainPipe::new(ctx);
        let mut main_bind = bind.clone();
        main_bind.images.push(shadow_map.get_output());

        let glow = GlowPipe::new(ctx);
        let glow_bind = bind.clone();

        let copy = copy_pipe(ctx, main.get_output());
        let glow_blend = glow_blend_pipe(ctx,
            main.get_output(), glow.get_output());
        let depth_view = depth_view_pipe(ctx, shadow_map.get_output());
 
        Stage {
            shadow_map,
            shadow_map_bind,
            main,
            main_bind,
            glow,
            glow_bind,
            copy,
            depth_view,
            glow_blend,
            objects,
            coloured_objects,
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
        self.glow_blend.bind.images[0] = self.main.get_output();
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

        let light_pos = vec3(-100.0, 100.0, 100.0);
        let light_range = (136.0, 200.0);
        let light_proj = Mat4::perspective_rh_gl(10.0f32.to_radians(), 1.0,
            light_range.0, light_range.1);
        let light_view = Mat4::look_at_rh(
            light_pos,
            vec3(0.0, 0.0, 0.0),
            vec3(0.0, 1.0, 0.0),
        );
        let light_pos_view = view * light_pos.extend(1.0);
        //self.rx += 0.01;
        self.ry += 0.01;
        let model = Mat4::from_euler(EulerRot::YXZ, self.ry, self.rx, 0.);

        //let (w, h) = ctx.screen_size();
        
        self.shadow_map.draw(ctx, &self.shadow_map_bind,
            &self.objects, 
            &model, &light_view, &light_proj);

        self.main.draw(ctx, &self.main_bind,
            &self.objects, 
            &self.coloured_objects, 
            &model, light_pos_view, &view_proj, &light_view,
            &light_proj);

        self.glow.draw(ctx, &self.glow_bind,
            &self.objects, 
            &self.coloured_objects, 
            &model, &view_proj);

        let output = &self.glow_blend;
        //let output = &self.depth_view;
        //let output = &mut self.copy;
        // output.bind.images[0] = self.glow.get_output();
        // and the post-processing-pass, rendering a quad, using the
        // previously rendered offscreen render-target as texture
        ctx.begin_default_pass(PassAction::Nothing);
        ctx.apply_pipeline(&output.pipe);
        ctx.apply_bindings(&output.bind);
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

    vec4 gamma_correct( in vec4 colour)
    {
        return vec4(pow(colour.xyz, vec3(1.0/2.2)), colour.w);
    }

    void main() {
        gl_FragColor = gamma_correct(texture2D(tex, texcoord));
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

mod glow_blend_shader {
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

    uniform sampler2D scene;
    uniform sampler2D glow;

    vec4 gamma_correct( in vec4 colour)
    {
        return vec4(pow(colour.xyz, vec3(1.0/2.2)), colour.w);
    }

    void main() {
        vec3 src = texture2D(scene, texcoord).rgb;
        vec3 dst = texture2D(glow, texcoord).rgb;
        vec4 colour = vec4(clamp((src + dst) - (src * dst), 0.0, 1.0), 1.0);
        gl_FragColor = gamma_correct(colour);
    }
    "#;

    pub fn meta() -> ShaderMeta {
        ShaderMeta {
            images: vec!["scene".to_string(),"glow".to_string() ],
            uniforms: UniformBlockLayout {
                uniforms: vec![],
            },
        }
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
    precision mediump float;

    varying vec2 texcoord;

    uniform sampler2D tex;
    
    float unpack_depth_simple(vec4 value) {
        return value.x;
    }

    float unpack_depth(const in vec4 rgba_depth)
    {
        const vec4 bit_shift = vec4(1.0/(256.0*256.0*256.0), 1.0/(256.0*256.0), 1.0/256.0, 1.0);
        float depth = dot(rgba_depth, bit_shift);
        return depth;
    }

    void main() {
        float depth = unpack_depth(texture2D(tex, texcoord));
        gl_FragColor = vec4(vec3(1.0 - depth), 1.0);
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

