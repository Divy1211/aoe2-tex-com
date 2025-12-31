from aoe2_tex_com.format import BcFormat, BcQuality

from typing import Optional

def encode(
    bytes_: bytes,
    format: BcFormat = BcFormat.Bc1,
    quality: BcQuality = BcQuality.Slow,
    gen_commands: bool = False,

    prev_info: tuple[bytes, int, int, int, int] = None,
) -> tuple[bytes, Optional[tuple[list[int, int], bytes]]]:
    """
    Encode the given image bytes using the format and the provided quality

    :param bytes_: The bytes to encode (common image formats are supported)
    :param format: The compression format
    :param quality: Compression quality

    :param gen_commands: If true, generates the skip/draw commands for use in SLD files

    :param prev_info: A tuple containing information from the previous frame.
                      This is the (block_bytes, width, height, delta offset_x1 (prev - this), delta offset_y1 (prev - this)).
                      Note - For animations that use keyframes/interpolation, block_bytes are the compressed blocks of
                      the previous frame *after* processing the previous frame's draw commands. This is the third
                      return value of this function, and it should be passed to the next decode call in this parameter
                      when required.

    :return: A tuple containing the compressed texture blocks as bytes, and if gen_commands is set to True, an optional
             tuple which contains the num skip/draw blocks and the bytes after taking out all the bytes that should be
             skipped according to the commands array, both of these are for direct use in SLD files
    """

def decode(
    bytes_: bytes,
    width: int,
    height: int,
    format: BcFormat,
    commands: list[tuple[int, int]] = None,

    prev_info: tuple[bytes, int, int, int, int] = None,
) -> tuple[bytes, bytes]:
    """
    Decode compressed bytes into raw image bytes

    :param bytes_: The bytes to decompress
    :param width: The width of the decompressed image
    :param height: The height of the decompressed image
    :param format: The compression format
    :param commands: Draw commands of the format [(num_blocks_skipped, num_blocks_drawn)]

    :param prev_info: A tuple containing information from the previous frame.
                      This is the (block_bytes, width, height, delta offset_x1 (prev - this), delta offset_y1 (prev - this)).
                      Note - For animations that use keyframes/interpolation, block_bytes are the compressed blocks of
                      the previous frame *after* processing the previous frame's draw commands. This is the second
                      return value of this function, and it should be passed to the next decode call in this parameter
                      when required. This is for use with SLD files.

    :return: A tuple containing the raw image bytes and the compressed blocks for this frame after the draw commands
             have been processed.
    """