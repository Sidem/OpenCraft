use super::*;
use crate::block::{BLOCK_COUNT, STONE};

#[test]
fn block_ids_take_a_byte_below_255_and_three_bytes_from_255() {
    let mut w = ByteWriter::default();
    w.block(0);
    w.block(STONE);
    w.block(254);
    assert_eq!(w.bytes.len(), 3, "the one-byte form every older save used");
    w.block(255);
    w.block(300);
    w.block(u16::MAX);
    assert_eq!(&w.bytes[3..], &[255, 255, 0, 255, 44, 1, 255, 255, 255]);
}

#[test]
fn block_ids_read_back_and_unknown_ones_fail() {
    let mut w = ByteWriter::default();
    (0..BLOCK_COUNT as u16).for_each(|b| w.block(b));
    w.block(BLOCK_COUNT as u16);
    let mut r = ByteReader::new(&w.bytes);
    for b in 0..BLOCK_COUNT as u16 {
        assert_eq!(r.block(), Some(b));
    }
    assert_eq!(r.block(), None, "an id past the table is damage");
    assert_eq!(ByteReader::new(&[255, 1]).block(), None, "a cut-off wide id is damage");
}
