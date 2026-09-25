use super::*;

#[test]
fn uniform_promotes_on_write() {
    let mut c = Chunk::uniform(0);
    c.set(1, 2, 3, 0);
    assert!(c.as_uniform().is_some(), "writing the same value must not allocate");
    c.set(1, 2, 3, 7);
    assert!(c.as_uniform().is_none());
    assert_eq!(c.get(1, 2, 3), 7);
    assert_eq!(c.get(0, 0, 0), 0);
}

#[test]
fn from_blocks_collapses() {
    assert_eq!(Chunk::from_blocks(vec![4; CHUNK_VOLUME]).as_uniform(), Some(4));
    let mut v = vec![4; CHUNK_VOLUME];
    v[100] = 1;
    assert!(Chunk::from_blocks(v).dense().is_some());
}
