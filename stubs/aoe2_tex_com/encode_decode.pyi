from aoe2_tex_com.format import BcFormat, BcQuality


def encode(bytes_: bytes, format: BcFormat, quality: BcQuality) -> bytes:
    """
    Encode the given image bytes using the format and the provided quality

    :param bytes_: The bytes to encode (common image formats are supported)
    :param format: The compression format
    :param quality: Compression quality

    :return: The compressed texture blocks as bytes
    """

def decode(
    bytes_: bytes,
    width: int,
    height: int,
    format: BcFormat,
    commands: list[tuple[int, int]] = None,

    prev_blocks: bytes = None,
    prev_width: int = None,
    prev_height: int = None,
    off_x: int = None,
    off_y: int = None,
) -> (bytes, bytes):
    """
    Decode compressed bytes into raw image bytes

    :param bytes_: The bytes to decompress
    :param width: The width of the decompressed image
    :param height: The height of the decompressed image
    :param format: The compression format
    :param commands: Draw commands of the format [(num_blocks_skipped, num_blocks_drawn)]

    :param prev_blocks: For animations that use keyframes/interpolation, pass the compressed blocks of the previous
                        frame *after* processing the previous frame's draw commands. This is the second return value of
                        this function, and it should be passed to the next decode call in this parameter when required.
                        Note: The remaining four params must be supplied when this param is supplied.

    :param prev_width: The previous frame's width
    :param prev_height: The previous frame's height
    :param off_x: This frame's delta offset_x from the previous frame (previous.offset_x - this.offset_x)
    :param off_y: This frame's delta offset_y from the previous frame (previous.offset_y - this.offset_y)

    :return: A tuple containing the raw image bytes and the compressed blocks for this frame after the draw commands
             have been processed.
    """