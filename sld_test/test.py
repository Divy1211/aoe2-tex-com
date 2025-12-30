import os.path

from aoe2_tex_com import encode, decode, BcFormat, BcQuality

from bfp_rs.types.le import Array

from sld import SldFile
from sld.sections.sld_frame.bc1_block import Bc1Block
from sld.sections.sld_frame.bc4_block import Bc4Block
from sld.sections.sld_frame.layer_header import SldLayerHeader

from utils import timed

def bc1_to_bytes(pixels: list[Bc1Block]) -> bytes:
    return Array[len(pixels)][Bc1Block].to_bytes(pixels)

def bc4_to_bytes(pixels: list[Bc4Block]) -> bytes:
    return Array[len(pixels)][Bc4Block].to_bytes(pixels)

def main():
    sld = SldFile.from_file(r"./files/u_vil_male_villager_attackA_x2.sld")

    with timed("decode"):
        prev, p_width, p_height = None, None, None
        p_x1, p_y1 = None, None
        for i, frame in enumerate(sld.frames):
            layer = frame.main_layer
            bytes_ = bc1_to_bytes(layer.pixels)
            # print(i, layer.header.storage_scheme)

            if isinstance(layer.header, SldLayerHeader):
                header = layer.header
            else:
                header = frame.main_layer.header

            width = header.offset_x2 - header.offset_x1
            height = header.offset_y2 - header.offset_y1
            x1, y1 = header.offset_x1, header.offset_y1

            commands = [(cmd.num_blocks_skipped, cmd.num_blocks_drawn) for cmd in layer.draw_commands]

            extras = ()
            if layer.header.storage_scheme & 128:
                extras = (prev, p_width, p_height, p_x1 - x1, p_y1 - y1)
                # print(i, extras[1:], (x1, y1), (p_x1, p_y1))

            img, prev = decode(bytes_, width, height, BcFormat.Bc1, commands, *extras)
            p_width, p_height = width, height
            p_x1, p_y1 = x1, y1
            if not os.path.exists(r"./frames"):
                os.mkdir(r"./frames")

            with open(rf"./frames/frame{i}.png", "wb") as file:
                file.write(img)

if __name__ == "__main__":
    main()
