/// Reads a block of data from some memory, wrapping around if the length is larger than the source.
/// The provided callback is invoked with the available data at each wrap-around.
/// The idea is to expose a wrapped memory view to the caller, without copying the data.
pub fn read_block(src: &[u8], src_offset: usize, length: usize, mut callback: impl FnMut(&[u8])) {
    // No source data or zero length: just invoke the callback with no data

    if src.is_empty() || length == 0 {
        callback(&[]);
        return;
    }

    // Read the request length, invoke the callback at each wrap-around

    let mut src_offset = src_offset % src.len();
    let mut remaining = length;

    while remaining > 0 {
        let to_read = src.len().saturating_sub(src_offset).min(remaining);

        callback(&src[src_offset..src_offset + to_read]);

        src_offset = src_offset.wrapping_add(to_read) % src.len();

        remaining = remaining.saturating_sub(to_read);
    }
}

/// Writes a block of data to some memory, wrapping around if the length is larger than the memory.
pub fn write_block(src: &[u8], dst: &mut [u8], dst_offset: usize) {
    // No source or destination data: do nothing

    if src.is_empty() || dst.is_empty() {
        return;
    }

    // Write the request length, wrapping around if the length is larger than the destination

    let mut src_offset = 0;
    let mut dst_offset = dst_offset % dst.len();
    let mut remaining = src.len();

    while remaining > 0 {
        let to_write = (dst.len().saturating_sub(dst_offset)).min(remaining);

        dst[dst_offset..dst_offset + to_write]
            .copy_from_slice(&src[src_offset..src_offset + to_write]);

        src_offset = src_offset.wrapping_add(to_write) % src.len();

        dst_offset = dst_offset.wrapping_add(to_write) % dst.len();

        remaining = remaining.saturating_sub(to_write);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper to test read_block.
    fn read(src: &[u8], offset: usize, length: usize, expected_slices: &[&[u8]]) {
        let mut callbacks = 0;

        read_block(src, offset, length, |data| {
            assert_eq!(data, expected_slices[callbacks]);

            callbacks += 1;
        });

        assert_eq!(callbacks, expected_slices.len());
    }

    const EMPTY: &[u8] = &[];

    #[test]
    fn read_block_empty() {
        let src = [];

        read(&src, 0, 0, &[EMPTY]);

        read(&src, 0, 5, &[EMPTY]);

        read(&src, 5, 0, &[EMPTY]);

        read(&src, 100, 200, &[EMPTY]);
    }

    #[test]
    fn read_block_all() {
        let src = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

        read(&src, 0, 0, &[EMPTY]);

        read(&src, 5, 0, &[EMPTY]);

        read(&src, 0, 3, &[&[1, 2, 3]]);

        read(&src, 3, 2, &[&[4, 5]]);

        read(
            &src,
            0,
            15,
            &[&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10], &[1, 2, 3, 4, 5]],
        );

        read(
            &src,
            5,
            47,
            &[
                &[6, 7, 8, 9, 10],
                &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
                &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
                &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
                &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
                &[1, 2],
            ],
        );
    }

    #[test]
    fn read_block_slice() {
        let src = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let src_slice = &src[4..8]; // [5, 6, 7, 8]

        read(&src_slice, 0, 0, &[EMPTY]);

        read(&src_slice, 5, 0, &[EMPTY]);

        read(&src_slice, 0, 3, &[&[5, 6, 7]]);

        read(&src_slice, 1, 2, &[&[6, 7]]);

        read(&src_slice, 2, 10, &[&[7, 8], &[5, 6, 7, 8], &[5, 6, 7, 8]]);

        read(&src_slice, 13, 8, &[&[6, 7, 8], &[5, 6, 7, 8], &[5]]);
    }

    #[test]
    fn write_block_empty_src() {
        let src = [];
        let mut dst = [0; 10];

        write_block(&src, &mut dst, 0);
        assert_eq!(dst, [0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);

        write_block(&src, &mut dst, 5);
        assert_eq!(dst, [0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);

        write_block(&src, &mut dst, 999);
        assert_eq!(dst, [0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn write_block_empty_dst() {
        let src = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let mut dst = [0; 0];

        write_block(&src, &mut dst, 0);
        assert_eq!(dst, EMPTY);

        write_block(&src, &mut dst, 5);
        assert_eq!(dst, EMPTY);

        write_block(&src, &mut dst, 999);
        assert_eq!(dst, EMPTY);
    }

    #[test]
    fn write_block_larger_dst() {
        let src = [1, 2, 3];
        let mut dst = [0; 10];

        write_block(&src, &mut dst, 0);
        assert_eq!(dst, [1, 2, 3, 0, 0, 0, 0, 0, 0, 0]);

        write_block(&src, &mut dst, 5);
        assert_eq!(dst, [1, 2, 3, 0, 0, 1, 2, 3, 0, 0]);

        write_block(&src, &mut dst, 9);
        assert_eq!(dst, [2, 3, 3, 0, 0, 1, 2, 3, 0, 1]);
    }

    #[test]
    fn write_block_larger_src() {
        let src = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let mut dst = [0; 3];

        write_block(&src, &mut dst, 0);
        assert_eq!(dst, [10, 8, 9]);

        write_block(&src, &mut dst, 2);
        assert_eq!(dst, [8, 9, 10]);

        write_block(&src, &mut dst, 10);
        assert_eq!(dst, [9, 10, 8]);
    }
}
