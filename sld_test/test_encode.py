import os.path
import shutil

from aoe2_tex_com import decode, BcFormat, encode

from sld import SldFile
from sld.sections.sld_frame.layer_header import SldLayerHeader

from utils import timed

from sld_test.m_utils import bc1_to_bytes, bc4_to_bytes

def main():
    sld = SldFile.from_file(r"./files/b_indi_university_age3_x2.sld")

    with timed("encode"):
        prev, p_width, p_height = None, None, None
        p_x1, p_y1 = None, None
        for i, frame in enumerate(os.listdir(dir := "./frames")):
            frame = os.path.join(dir, frame)
            with open(frame, "rb") as file:
                img = file.read()
            (_bytes, (draw_cmds, bytes_)) = encode(img, BcFormat.Bc1, gen_commands = True)

            img, _prev = decode(_bytes, 748, 572, BcFormat.Bc1)

            with open(r"./frames/test.png", "wb") as file:
                file.write(img)

    sld_cmds = sld.frames[0].main_layer.draw_commands
    sld_blocks = bc1_to_bytes(sld.frames[0].main_layer.pixels)
    print(len(draw_cmds), draw_cmds)
    print(len(sld_cmds), [(cmd.num_blocks_skipped, cmd.num_blocks_drawn) for cmd in sld_cmds])

    print(len(bytes_))
    print(len(sld_blocks))



if __name__ == "__main__":
    main()
