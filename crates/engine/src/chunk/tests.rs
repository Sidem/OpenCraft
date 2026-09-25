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
fn same_blocks_write_the_same_bytes() {
    let bytes = |c: &Chunk| {
        let mut w = ByteWriter::default();
        c.write_state(&mut w);
        w.bytes
    };
    // Dense but all stone again, versus uniform stone: one run either way.
    let mut dense = Chunk::uniform(1);
    dense.set(5, 6, 7, 2);
    dense.set(5, 6, 7, 1);
    assert!(dense.dense().is_some());
    assert_eq!(bytes(&dense), bytes(&Chunk::uniform(1)));
    assert_eq!(bytes(&dense), [0x00, 0x80, 1]);

    dense.set(1, 0, 0, 2);
    assert_eq!(bytes(&dense), [1, 0, 1, 1, 0, 2, 0xfe, 0x7f, 1], "runs of 1, 1 and 32766");
}

#[test]
fn from_blocks_collapses() {
    assert_eq!(Chunk::from_blocks(vec![4; CHUNK_VOLUME]).as_uniform(), Some(4));
    let mut v = vec![4; CHUNK_VOLUME];
    v[100] = 1;
    assert!(Chunk::from_blocks(v).dense().is_some());
}
