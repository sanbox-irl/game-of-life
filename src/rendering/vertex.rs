use gfx_hal::{
    format::Format,
    pso::{AttributeDesc, ElemOffset, Element},
};
use std::mem;

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Vertex {
    xyz: [f32; 3],
    uv: [f32; 2],
}

impl Vertex {
    pub fn attributes() -> Vec<AttributeDesc> {
        let position_attribute = AttributeDesc {
            location: 0,
            binding: 0,
            element: Element {
                format: Format::Rgb32Sfloat,
                offset: 0,
            },
        };
        let uv_attribute = AttributeDesc {
            location: 1,
            binding: 0,
            element: Element {
                format: Format::Rg32Sfloat,
                offset: mem::size_of::<[f32; 3]>() as ElemOffset,
            },
        };
        let color_attribute = AttributeDesc {
            location: 2,
            binding: 0,
            element: Element {
                format: Format::Rgb32Sfloat,
                offset: mem::size_of::<[f32; 5]>() as ElemOffset,
            },
        };
        vec![position_attribute, uv_attribute, color_attribute]
    }
}

#[cfg_attr(rustfmt, rustfmt_skip)]
pub const QUAD_VERTICES: [Vertex; 4] = [
    Vertex { xyz: [0.0, 0.0, 0.0], uv: [0.0, 1.0] }, /* bottom left */
    Vertex { xyz: [0.0, 1.0, 0.0], uv: [0.0, 0.0] }, /* top left */
    Vertex { xyz: [1.0, 0.0, 0.0], uv: [1.0, 1.0] }, /* bottom right */
    Vertex { xyz: [1.0, 1.0, 0.0], uv: [1.0, 0.0] }, /* top right */
];

#[cfg_attr(rustfmt, rustfmt_skip)]
pub const QUAD_INDICES: [u16; 6] = [
    0,  1,  2,  2,  1,  3,
];
