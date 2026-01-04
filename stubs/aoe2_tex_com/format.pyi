from enum import IntEnum

class BcQuality(IntEnum):
    Fast = 0
    Normal = 1
    Slow = 2

class BcFormat(IntEnum):
    Bc1 = 0
    Bc4 = 1
    Bc7 = 2

class DrawCall:
    skip: int
    draw: int

    def __new__(cls, skip: int, draw: int):
        ...
