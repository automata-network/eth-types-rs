use crate::SH256;

pub trait Hasher {
    fn hash(&self) -> SH256;
}
