use crate::pack::BlockWord;

#[inline]
pub fn block_is_skip<T: Copy + Eq>(
    idx: usize,
    blocks: &[T],
    block_width: u32,
    transparent: T,
    prev_info: &Option<(Vec<T>, u32, u32, i32, i32)>,
) -> bool {
    let cur = blocks[idx];

    let Some((ref prev, prev_width, prev_height, off_x, off_y)) = *prev_info else {
        return cur == transparent;
    };
    
    let row = (idx / block_width as usize) as i32 - off_y;
    let col = (idx % block_width as usize) as i32 - off_x;

    if row < 0 || col < 0 || row >= prev_height as i32 || col >= prev_width as i32 {
        return cur == transparent
    }
    let pidx = (row * prev_width as i32 + col) as usize;
    cur == prev[pidx]
}

pub fn encode_blocks<W: BlockWord>(
    blocks: &[u8],
    block_width: u32,
    total_blocks: usize,
    transparent_block: &[u8],
    prev_info: Option<(Vec<u8>, u32, u32, i32, i32)>,
) -> (Vec<(u8, u8)>, Vec<u8>) {
    let blocks = W::pack(blocks);
    let prev_info = prev_info.map(|(blocks, pw, ph, ox, oy)| {
        (W::pack(&blocks), (pw + 3)/4, (ph + 3)/4, ox/4, oy/4)
    });
    let transparent_block = W::from_bytes(transparent_block);

    let mut commands = Vec::new();
    let mut drawn_blocks = Vec::new();

    let mut idx = 0_usize;
    while idx < total_blocks {
        let mut skip = 0_u8;
        let mut draw = 0_u8;

        while idx < total_blocks && skip < u8::MAX
            && block_is_skip(idx, &blocks, block_width, transparent_block, &prev_info) {
            skip += 1;
            idx += 1;
        }

        let draw_start = idx;
        while idx < total_blocks && draw < u8::MAX
            && !block_is_skip(idx, &blocks, block_width, transparent_block, &prev_info) {
            draw += 1;
            idx += 1;
        }

        if skip != 0 || draw != 0 {
            commands.push((skip, draw));
            drawn_blocks.extend_from_slice(&blocks[draw_start..idx])
        }
    }

    (commands, W::unpack(&drawn_blocks))
}