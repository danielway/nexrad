/// The position of this chunk within the volume.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum ChunkType {
    Start,
    Intermediate,
    End,
}
