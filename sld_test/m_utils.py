from bfp_rs.types.le import Array
from sld.sections.sld_frame.bc1_block import Bc1Block
from sld.sections.sld_frame.bc4_block import Bc4Block


def bc1_to_bytes(pixels: list[Bc1Block]) -> bytes:
    return Array[len(pixels)][Bc1Block].to_bytes(pixels)


def bc4_to_bytes(pixels: list[Bc4Block]) -> bytes:
    return Array[len(pixels)][Bc4Block].to_bytes(pixels)
