use crate::ident::Ident;
use std::fmt;
use std::rc::Rc;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Ty {
    Void,
    Bool,
    Int,
    Float,
    Bvec2,
    Bvec3,
    Bvec4,
    Ivec2,
    Ivec3,
    Ivec4,
    Vec2,
    Vec3,
    Vec4,
    Mat2,
    Mat3,
    Mat4,
    Array { elem_ty: Rc<Ty>, len: usize },
    Struct { ident: Ident },
}

impl Ty {
    pub fn elem_ty(&self) -> Option<Ty> {
        match *self {
            Ty::Bvec2 | Ty::Bvec3 | Ty::Bvec4 => Some(Ty::Bool),
            Ty::Ivec2 | Ty::Ivec3 | Ty::Ivec4 => Some(Ty::Int),
            Ty::Vec2 | Ty::Vec3 | Ty::Vec4 => Some(Ty::Float),
            Ty::Mat2 => Some(Ty::Vec2),
            Ty::Mat3 => Some(Ty::Vec3),
            Ty::Mat4 => Some(Ty::Vec4),
            Ty::Array { ref elem_ty, .. } => Some((**elem_ty).clone()),
            _ => None,
        }
    }

    pub fn len(&self) -> Option<usize> {
        match *self {
            Ty::Bool | Ty::Int | Ty::Float => Some(1),
            Ty::Bvec2 | Ty::Ivec2 | Ty::Vec2 | Ty::Mat2 => Some(2),
            Ty::Bvec3 | Ty::Ivec3 | Ty::Vec3 | Ty::Mat3 => Some(3),
            Ty::Bvec4 | Ty::Ivec4 | Ty::Vec4 | Ty::Mat4 => Some(4),
            Ty::Array { len, .. } => Some(len),
            _ => None,
        }
    }

    pub fn size(&self) -> Option<usize> {
        match self {
            Ty::Bool | Ty::Int | Ty::Float => Some(1),
            Ty::Bvec2 | Ty::Ivec2 | Ty::Vec2 => Some(2),
            Ty::Bvec3 | Ty::Ivec3 | Ty::Vec3 => Some(3),
            Ty::Bvec4 | Ty::Ivec4 | Ty::Vec4 | Ty::Mat2 => Some(4),
            Ty::Mat3 => Some(9),
            Ty::Mat4 => Some(16),
            Ty::Array { elem_ty, len } => elem_ty.size().map(|size| size * len),
            _ => None,
        }
    }
}

impl<'a> fmt::Display for Ty {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Ty::Void => write!(f, "void"),
            Ty::Bool => write!(f, "bool"),
            Ty::Int => write!(f, "int"),
            Ty::Float => write!(f, "float"),
            Ty::Bvec2 => write!(f, "bvec2"),
            Ty::Bvec3 => write!(f, "bvec3"),
            Ty::Bvec4 => write!(f, "bvec4"),
            Ty::Ivec2 => write!(f, "ivec2"),
            Ty::Ivec3 => write!(f, "ivec3"),
            Ty::Ivec4 => write!(f, "ivec4"),
            Ty::Vec2 => write!(f, "vec2"),
            Ty::Vec3 => write!(f, "vec3"),
            Ty::Vec4 => write!(f, "vec4"),
            Ty::Mat2 => write!(f, "mat2"),
            Ty::Mat3 => write!(f, "mat3"),
            Ty::Mat4 => write!(f, "mat4"),
            Ty::Array { elem_ty, len } => write!(f, "{}[{}]", elem_ty, len,),
            Ty::Struct { ident } => write!(f, "{}", ident),
        }
    }
}