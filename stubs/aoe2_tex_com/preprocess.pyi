from aoe2_tex_com import BcFormat


class ProcessedFrame:
    image_bytes: bytes
    offset_x1: int
    offset_y1: int
    offset_x2: int
    offset_y2: int
    anchor_x: int
    anchor_y: int

def preprocess_frames(
    frame_info: list[tuple[str, tuple[int, int]]],
    format: BcFormat = BcFormat.Bc1,
) -> tuple[list[ProcessedFrame], tuple[int, int], tuple[int, int]]:
    """
    Preprocesses frames to remove transparency and

    :param frame_info:
    :param format:
    :return:
    """