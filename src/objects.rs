use miniquad::*;

use glam::{vec3, vec4, Vec3, Vec4, Mat4, EulerRot};
use xorshift::{Rng, RngJump, Xoroshiro128, SeedableRng};

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
        /* pos               color                   normal         uvs */
        -1.0, -1.0, -1.0,    1.0, 0.5, 0.5, 1.0, 0.0, 0.0, -1.0,    0.0, 0.0,
        1.0, -1.0, -1.0,    1.0, 0.5, 0.5, 1.0,  0.0, 0.0, -1.0,   1.0, 0.0,
        1.0,  1.0, -1.0,    1.0, 0.5, 0.5, 1.0,  0.0, 0.0, -1.0,   1.0, 1.0,
        -1.0,  1.0, -1.0,    1.0, 0.5, 0.5, 1.0, 0.0, 0.0, -1.0,    0.0, 1.0,

        -1.0, -1.0,  1.0,    0.5, 1.0, 0.5, 1.0, 0.0, 0.0, 1.0,    0.0, 0.0,
        1.0, -1.0,  1.0,    0.5, 1.0, 0.5, 1.0,  0.0, 0.0, 1.0,   1.0, 0.0,
        1.0,  1.0,  1.0,    0.5, 1.0, 0.5, 1.0,  0.0, 0.0, 1.0,   1.0, 1.0,
        -1.0,  1.0,  1.0,    0.5, 1.0, 0.5, 1.0, 0.0, 0.0, 1.0,    0.0, 1.0,

        -1.0, -1.0, -1.0,    0.5, 0.5, 1.0, 1.0, -1.0, 0.0, 0.0,    0.0, 0.0,
        -1.0,  1.0, -1.0,    0.5, 0.5, 1.0, 1.0, -1.0, 0.0, 0.0,    1.0, 0.0,
        -1.0,  1.0,  1.0,    0.5, 0.5, 1.0, 1.0, -1.0, 0.0, 0.0,    1.0, 1.0,
        -1.0, -1.0,  1.0,    0.5, 0.5, 1.0, 1.0, -1.0, 0.0, 0.0,    0.0, 1.0,

        1.0, -1.0, -1.0,    1.0, 0.5, 0.0, 1.0,  1.0, 0.0, 0.0,   0.0, 0.0,
        1.0,  1.0, -1.0,    1.0, 0.5, 0.0, 1.0,  1.0, 0.0, 0.0,   1.0, 0.0,
        1.0,  1.0,  1.0,    1.0, 0.5, 0.0, 1.0,  1.0, 0.0, 0.0,   1.0, 1.0,
        1.0, -1.0,  1.0,    1.0, 0.5, 0.0, 1.0,  1.0, 0.0, 0.0,   0.0, 1.0,

        -1.0, -1.0, -1.0,    0.0, 0.5, 1.0, 1.0, 0.0, -1.0, 0.0,    0.0, 0.0,
        -1.0, -1.0,  1.0,    0.0, 0.5, 1.0, 1.0, 0.0, -1.0, 0.0,    1.0, 0.0,
        1.0, -1.0,  1.0,    0.0, 0.5, 1.0, 1.0,  0.0, -1.0, 0.0,   1.0, 1.0,
        1.0, -1.0, -1.0,    0.0, 0.5, 1.0, 1.0,  0.0, -1.0, 0.0,   0.0, 1.0,

        -1.0,  1.0, -1.0,    1.0, 0.0, 0.5, 1.0, 0.0, 1.0, 0.0,    0.0, 0.0,
        -1.0,  1.0,  1.0,    1.0, 0.0, 0.5, 1.0, 0.0, 1.0, 0.0,    1.0, 0.0,
        1.0,  1.0,  1.0,    1.0, 0.0, 0.5, 1.0,  0.0, 1.0, 0.0,   1.0, 1.0,
        1.0,  1.0, -1.0,    1.0, 0.0, 0.5, 1.0,  0.0, 1.0, 0.0,   0.0, 1.0
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

fn rng_from_pos(pos:Vec3, i:u64) -> Xoroshiro128 {
    let h = i32::MAX as i64;
    let mut rng: Xoroshiro128 = SeedableRng::from_seed(&[1, i][..]);
    // first output is just the seed so jump to somewhere interesting
    rng.jump(1);
    for d in pos.to_array().iter() {
        let v = ((*d as i32) as i64 + h) as u64;
        let seed = [rng.next_u64(), rng.next_u64() + v];
        rng.reseed(&seed[..]);
        rng.jump(1);
    }
    rng
}

fn cube(pos: Vec3) -> Mat4 {
    // a cuboid with random scale, rotation at pos
    let mut rng = rng_from_pos(pos, 1);
    let rot = Mat4::from_euler(EulerRot::YXZ, 
        rng.gen_range(-std::f32::consts::PI, std::f32::consts::PI),
        0.0, 0.);
    let s = 0.125 * rng.gen_range(0.5, 1.1);
    let y = 10.0 * s;
    let scale = Mat4::from_scale(vec3(s, y, s));
    let trans = Mat4::from_translation(vec3(
        pos.x,
        pos.y + y - 1.0,
        pos.z,
    ));
    trans * scale * rot
}

pub fn cubes(pos: Vec3) -> Vec<Object> {
    // a grid of cuboids on integer coords surrounding pos
    // plus a ground plane centred at pos
    let o = vec3(-pos.x.floor(), -pos.y.floor(), -pos.z.floor());
    let mut cubes = Vec::<Mat4>::new();
    for z in -3..6 {
        for x in -3..3 {
            cubes.push(cube(o + vec3(x as f32, 0., z as f32)));
        }
    }
    let trans = Mat4::from_translation(vec3(0., 0., 2.));
    let rot = Mat4::from_euler(EulerRot::YXZ, 
        0., std::f32::consts::PI / 2., 0.);
    let scale = Mat4::from_scale(vec3(10.0, 1.0, 10.0));
    let trans2 = Mat4::from_translation(-pos);
    let ground_plane = trans2 * scale * rot * trans;

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
        start: 0,
        end: 6
    });

    objects
}

fn wrap(a:Vec3, min:Vec3, max:Vec3) -> Vec3 {
    let o = a - min;
    let s = max - min;
    let r = (o / s).fract();
    (r * s) + min
}

pub fn coloured_cubes(pos:Vec3) -> Vec<ColouredObject> {
    // coloured cubes at random scale, rotation, and position wrapped to be
    // within fixed bounds of pos
    let colours = vec![
        vec4(1., 0., 0., 1.),
        vec4(1., 0.5, 0., 1.),
        vec4(1., 1., 0., 1.),
        vec4(0., 1., 0., 1.),
        vec4(0., 0., 1., 1.),
        vec4(0.3, 0., 0.5, 1.),
        vec4(0.5, 0., 0.5, 1.),
    ];
    let mut rng = rng_from_pos(vec3(0., 0., 0., ), 2);

    let mut cubes = Vec::<ColouredObject>::new();
    for colour in colours {
        let x:f32 = rng.gen_range(-10.0, 10.0);
        let z:f32 = rng.gen_range(-10.0, 10.0);
        let rot = Mat4::from_euler(EulerRot::YXZ, 
            rng.gen_range(-std::f32::consts::PI, std::f32::consts::PI),
            0.0, 0.);
        let s = rng.gen_range(0.4, 0.6);
        let scale = Mat4::from_scale(vec3(s, s, s));
        let p = vec3(x, s - 1.0, z);
        let p = wrap(p, -pos - vec3(10.0, 10.0, 10.0),
                        -pos + vec3(10.0, 10.0, 10.0));
        let trans = Mat4::from_translation(p);
        cubes.push(
            ColouredObject {
                object: Object {
                    model: trans * scale * rot,
                    start: 0,
                    end: 36
                },
                colour
            });
    }
    cubes
}