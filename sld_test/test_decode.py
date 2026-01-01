import os.path
import shutil

from aoe2_tex_com import encode, decode, BcFormat, BcQuality

from sld import SldFile
from sld.sections.sld_frame.layer_header import SldLayerHeader

from utils import timed

from sld_test.m_utils import bc1_to_bytes, bc4_to_bytes

def main():
    sld = SldFile.from_file(r"./files/b_west_blacksmith_age2_x2.sld")

    if os.path.exists(r"./frames"):
        shutil.rmtree(r"./frames")
    os.mkdir(r"./frames")

    with timed("decode"):
        prev, p_width, p_height = None, None, None
        p_x1, p_y1 = None, None
        for i, frame in enumerate(sld.frames):
            layer = frame.main_layer
            if layer.header.storage_scheme & 1:
                bytes_ = bc4_to_bytes(layer.pixels)
                format = BcFormat.Bc4
            else:
                bytes_ = bc1_to_bytes(layer.pixels)
                format = BcFormat.Bc1

            # print(i, layer.header.storage_scheme)

            if isinstance(layer.header, SldLayerHeader):
                header = layer.header
            else:
                header = frame.main_layer.header

            width = header.offset_x2 - header.offset_x1
            height = header.offset_y2 - header.offset_y1
            x1, y1 = header.offset_x1, header.offset_y1

            commands = [(cmd.num_blocks_skipped, cmd.num_blocks_drawn) for cmd in layer.draw_commands]

            extras = None
            if layer.header.storage_scheme & 128:
                extras = (prev, p_width, p_height, p_x1 - x1, p_y1 - y1)
                # print(i, extras[1:], (x1, y1), (p_x1, p_y1))

            img, prev = decode(bytes_, width, height, format, commands, extras)
            p_width, p_height = width, height
            p_x1, p_y1 = x1, y1

            with open(rf"./frames/frame_m{i}.png", "wb") as file:
                file.write(img)

if __name__ == "__main__":
    main()
