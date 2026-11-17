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

#[test]
fn a_chunk_holds_and_saves_block_ids_past_a_byte() {
    let mut c = Chunk::uniform(0);
    c.set(1, 2, 3, 300);
    c.set(4, 5, 6, 70);
    assert_eq!((c.get(1, 2, 3), c.get(4, 5, 6), c.get(0, 0, 0)), (300, 70, 0));
    let mut w = ByteWriter::default();
    c.write_state(&mut w);
    // 300 is past the block table, so a load refuses the chunk rather than index past a table.
    assert!(Chunk::read_state(&mut ByteReader::new(&w.bytes)).is_none());
    // The same chunk with ids in the table loads back whole.
    c.set(1, 2, 3, 200);
    let mut w = ByteWriter::default();
    c.write_state(&mut w);
    let back = Chunk::read_state(&mut ByteReader::new(&w.bytes)).expect("loads");
    assert_eq!((back.get(1, 2, 3), back.get(4, 5, 6)), (200, 70));
}
