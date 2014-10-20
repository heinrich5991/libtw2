map_item!(0, Version,
    1: ,
)
map_item!(1, Info,
    1: author map_version credits license,
)
map_item!(2, Image,
    1: width height external name data,
    2: format,
)
map_item!(3, Envelope,
    1: channels start_points num_points name[8],
    2: synchronized,
)
map_item!(4, Group,
    1: offset_x offset_y parallax_x parallax_y start_layer num_layers,
    2: use_clipping clip_x clip_y clip_w clip_h,
)
map_item!(5, Layer,
    1: type_ flags,
)
map_item!(6, EnvPoints,
    1: time curvetype values[4],
)
