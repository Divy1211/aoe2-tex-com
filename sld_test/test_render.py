import os.path
import shutil

from aoe2_tex_com import encode, decode, BcFormat, BcQuality, DrawCall, render

from sld import SldFile
from sld.sections.sld_frame.layer_header import SldLayerHeader

from utils import timed

from sld_test.m_utils import bc1_to_bytes, bc4_to_bytes

def main():
    sld = SldFile.from_file(r"./files/u_vil_male_villager_attackA_x2.sld")

    if os.path.exists(r"./frames"):
        shutil.rmtree(r"./frames")
    os.mkdir(r"./frames")

    with (timed("render")):
        p_info = {}
        for i, frame in enumerate(sld.frames):
            for layer_name, suffix in [("main_layer", "m"), ("shadow_layer", "s"), ("player_color_mask_layer", "p"), ("damage_mask_layer", "d")]:
                layer = getattr(frame, layer_name)
                if layer is None:
                    continue

                layer_p_info = p_info.setdefault(layer_name, {})

                prev = layer_p_info.get("prev", None)
                p_width = layer_p_info.get("p_width", None)
                p_height = layer_p_info.get("p_height", None)
                p_x1 = layer_p_info.get("p_x1", None)
                p_y1 = layer_p_info.get("p_y1", None)

                if layer.header.storage_scheme & 1:
                    bytes_ = bc4_to_bytes(layer.pixels)
                    format = BcFormat.Bc4
                else:
                    bytes_ = bc1_to_bytes(layer.pixels)
                    format = BcFormat.Bc1

                if isinstance(layer.header, SldLayerHeader):
                    header = layer.header
                else:
                    header = frame.main_layer.header

                width = header.offset_x2 - header.offset_x1
                height = header.offset_y2 - header.offset_y1
                x1, y1 = header.offset_x1, header.offset_y1

                commands = [DrawCall(cmd.num_blocks_skipped, cmd.num_blocks_drawn) for cmd in layer.draw_commands]

                extras = None
                if layer.header.storage_scheme & 128:
                    extras = (prev, p_width, p_height, p_x1 - x1, p_y1 - y1)
                    # print(i, extras[1:], (x1, y1), (p_x1, p_y1))

                img, layer_p_info["prev"] = decode(bytes_, width, height, format, commands, extras)
                layer_p_info["p_width"] = width
                layer_p_info["p_height"] = height
                layer_p_info["p_x1"] = x1
                layer_p_info["p_y1"] = y1
                layer_p_info["img"] = img

            img = render(
                p_info["main_layer"]["img"], p_info["shadow_layer"]["img"],
                (p_info["shadow_layer"]["p_x1"] - p_info["main_layer"]["p_x1"], p_info["shadow_layer"]["p_y1"] - p_info["main_layer"]["p_y1"]),
                p_info["player_color_mask_layer"]["img"]
            )
            with open(f"./frames/render{i}.png", "wb") as file:
                file.write(img)

if __name__ == "__main__":
    main()
