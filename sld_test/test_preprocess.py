import os.path
import shutil

from aoe2_tex_com import encode, decode, BcFormat, BcQuality, preprocess_frames

from sld import SldFile
from sld.sections.sld_frame.layer_header import SldLayerHeader

from utils import timed

from sld_test.m_utils import bc1_to_bytes, bc4_to_bytes

def main():
    anchor_points = [(262, 176), (262+53, 176+53), (262, 176)]
    frame_info = []
    for i, frame in enumerate(os.listdir("./test_frames")):
        frame = os.path.join("./test_frames", frame)
        frame_info.append((frame, anchor_points[i]))

    (frames, (x, y), (w, h)) = preprocess_frames(frame_info, BcFormat.Bc1)

    print(x, y, w, h)

    for i, frame in enumerate(frames):
        with open(f"./test_frames/pframe{i}.png", "wb") as file:
            file.write(frame.image_bytes)
            print(frame.offset_x1, frame.offset_y1, frame.offset_x2, frame.offset_y2)
            print(frame.anchor_x, frame.anchor_y)

if __name__ == "__main__":
    main()
