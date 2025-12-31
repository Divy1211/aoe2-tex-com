import os.path
import shutil

from aoe2_tex_com import decode, BcFormat, encode

from sld import SldFile
from sld.sections.sld_frame.layer_header import SldLayerHeader

from utils import timed

from sld_test.m_utils import bc1_to_bytes, bc4_to_bytes

def main():
    sld = SldFile.from_file(r"./files/u_vil_male_villager_attackA_x2.sld")

    with timed("encode"):
        prev, p_width, p_height = None, None, None
        p_x1, p_y1 = None, None
        extra = None

        for i, frame in enumerate(os.listdir(dir_name := "./frames")):
            frame = os.path.join(dir_name, frame)
            with open(frame, "rb") as file:
                print(frame)
                img = file.read()
            if prev is not None:
                extra = (prev, 64, 84, 0, 0)

            (prev, (draw_cmds, bytes_)) = encode(img, BcFormat.Bc1, gen_commands = True, prev_info = extra)

            print(f"======== {i} ========")

            sld_cmds = sld.frames[i].main_layer.draw_commands
            sld_blocks = bc1_to_bytes(sld.frames[i].main_layer.pixels)
            print(len(draw_cmds), draw_cmds)
            print(len(sld_cmds), [(cmd.num_blocks_skipped, cmd.num_blocks_drawn) for cmd in sld_cmds])

            print(len(bytes_))
            print(len(sld_blocks))

            if i == 1:
                break


if __name__ == "__main__":
    main()
