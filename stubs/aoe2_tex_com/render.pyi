def render(
    main_layer: bytes,
    shadow_layer: bytes,
    shadow_layer_offset: tuple[int, int],
    player_color_mask: bytes,
    damage_mask: tuple[bytes, int] = None,
    color: tuple[int, int, int] = (255, 0, 0),
) -> bytes:
    """
    Renders an SLD frame given the different layers after decoding and a player color
    """
