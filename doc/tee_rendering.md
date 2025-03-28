This document describes how to render a tee like Teeworlds 0.6 or DDNet, given
a tee skin image.

A skin image must have an aspect ratio of 2:1 (width:height). We get the
following body parts by splitting the tee skin image (3/8 means at 3/8 of the
image's width, 1/4 means at 1/4 of the image's height, etc.):

    body:         (0/8, 0/4) to (3/8, 3/4)
    body_outline: (3/8, 0/4) to (6/8, 3/4)
    hand:         (6/8, 0/4) to (7/8, 1/4)
    hand_outline: (7/8, 0/4) to (8/8, 1/4)
    foot:         (6/8, 1/4) to (8/8, 2/4)
    foot_outline: (6/8, 2/4) to (8/8, 3/4)
    eye_normal:   (2/8, 3/4) to (3/8, 4/4)
    eye_angry:    (3/8, 3/4) to (4/8, 4/4)
    eye_pain:     (4/8, 3/4) to (5/8, 4/4)
    eye_happy:    (5/8, 3/4) to (6/8, 4/4)
    eye_dead:     (6/8, 3/4) to (7/8, 4/4) -- entirely unused
    eye_surprise: (7/8, 3/4) to (8/8, 4/4)

This leaves an unused rectangle at `(0/8, 3/4) to (2/8, 4/4)`.

For rendering, the segments need to be scaled like this (relative to body being
100%):

    body: 100%
    feet: 150%
    eyes: 120%
    hand: 93.75%

The right eye needs to be mirrored horizontally (<->).
The last eye shape `eye_blink` is achieved by scaling `eye_normal` 120%
horizontally, but 45% vertically.

Then, the images must be positioned like the following (hands or moving feet
not handled), relative to 64/64 or 1 being the edge length of the body
segment).

    body:
        x: 4/64 up
    feet:
        x: 7/64 left/right
        y: 10/64 down
    eyes:
        dir = angle of eyes (view angle), right = 0
        eyes_center:
            x: cos(dir) * 0.125 right
            y: body.y and then sin(dir) * 0.1 - 0.05 down
        eyes_offset:
            x: 0.075 - abs(cos(dir)) * 0.01 left/right
