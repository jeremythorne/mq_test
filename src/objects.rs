use miniquad::*;

use glam::{vec3, vec4, Vec4, Mat4, EulerRot};
use quad_rand as qrand;

pub struct Object {
    pub model:Mat4,
    pub start:i32,
    pub end:i32
}

pub struct ColouredObject {
    pub object:Object,
    pub colour:Vec4
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

pub fn cube_bindings(ctx: &mut Context) -> Bindings {
    let (vertices, indices) = cube_verts();
    let vertex_buffer = Buffer::immutable(ctx, BufferType::VertexBuffer, &vertices);
    let index_buffer = Buffer::immutable(ctx, BufferType::IndexBuffer, &indices);

    Bindings {
        vertex_buffers: vec![vertex_buffer],
        index_buffer: index_buffer,
        images: vec![],
    }
}

pub fn cubes() -> Vec<Object> {
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
    let rot = Mat4::from_euler(EulerRot::YXZ, 
        0., -std::f32::consts::PI / 2., 0.);
    let scale = Mat4::from_scale(vec3(10.0, 10.0, 1.0));
    let ground_plane = rot * scale;

    let mut objects = Vec::<Object>::new();
    for cube in cubes.iter() {
        objects.push(Object {
            model: *cube,
            start: 0,
            end: 36
        });
    }
    objects.push(Object{
        model: ground_plane,
        start:0,
        end: 6
    });

    objects
}

pub fn coloured_cubes() -> Vec<ColouredObject> {
    let colours = vec![
        vec4(1., 0., 0., 1.),
        vec4(1., 0.5, 0., 1.),
        vec4(1., 1., 0., 1.),
        vec4(0., 1., 0., 1.),
        vec4(0., 0., 1., 1.),
        vec4(0.3, 0., 0.5, 1.),
        vec4(0.5, 0., 0.5, 1.),
    ];

    let mut cubes = Vec::<ColouredObject>::new();
    for colour in colours {
        let r = qrand::gen_range(0.5, 1.);
        let rot = Mat4::from_euler(EulerRot::YXZ, 
            qrand::gen_range(-std::f32::consts::PI, std::f32::consts::PI),
            0.0, 0.);
        let rot2 = Mat4::from_euler(EulerRot::YXZ, 
            qrand::gen_range(-std::f32::consts::PI, std::f32::consts::PI),
            0.0, 0.);
         let s = (1.4 - r) * qrand::gen_range(0.7, 0.9);
        let scale = Mat4::from_scale(vec3(s, s, s));
        let trans = Mat4::from_translation(vec3(
            6.0 * r,
            s - 1.0,
            0.0,
        ));
        cubes.push(
            ColouredObject {
                object: Object {
                    model: rot * trans * scale * rot2,
                    start: 0,
                    end: 36
                },
                colour
            });
    }
    cubes
}