#!/usr/bin/env python3
"""Draws the five module icons (256x256 PNG, as LGX packaging requires) in the Logos design
system's colours: Theme.palette.background (#171717), primary/accentOrange (#ED7B58),
textSecondary (#A4A4A4). Drawn at 4x and downscaled for smooth edges.

    python3 tools/make-icons.py
"""
import math
import os

from PIL import Image, ImageDraw

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
S = 1024                      # drawing size; saved at 256
BG = (23, 23, 23, 255)        # gray900
RAISED = (35, 35, 35, 255)    # gray360 (surfaceRaised)
LINE = (52, 52, 52, 255)      # gray320 (surface)
ORANGE = (237, 123, 88, 255)  # orange300
DEEP = (191, 81, 4, 255)      # orange700
MUTED = (164, 164, 164, 255)  # gray400


def canvas():
    im = Image.new("RGBA", (S, S), (0, 0, 0, 0))
    d = ImageDraw.Draw(im)
    d.rounded_rectangle([0, 0, S - 1, S - 1], radius=220, fill=BG)
    return im, d


def map_lines(d, box, step=96, width=10):
    """A faint street grid with one diagonal avenue, clipped to box."""
    x0, y0, x1, y1 = box
    for x in range(x0 + step // 2, x1, step):
        d.line([(x, y0), (x, y1)], fill=LINE, width=width)
    for y in range(y0 + step // 2, y1, step):
        d.line([(x0, y), (x1, y)], fill=LINE, width=width)
    d.line([(x0, y1 - 80), (x1 - 80, y0)], fill=LINE, width=width * 3)


def pin(d, cx, top, r, fill=ORANGE, hole=BG, outline=False):
    """A map pin: a circle of radius r whose sides run tangent into a point 2.1 r below."""
    cy = top + r
    tip = (cx, cy + 2.1 * r)
    dist = tip[1] - cy
    a = math.acos(r / dist)
    left = (cx - r * math.sin(a), cy + r * math.cos(a))
    right = (cx + r * math.sin(a), cy + r * math.cos(a))
    if outline:
        w = int(r * 0.22)
        d.ellipse([cx - r, cy - r, cx + r, cy + r], outline=fill, width=w)
        d.line([left, tip, right], fill=fill, width=w, joint="curve")
        d.ellipse([cx - r * 0.36, cy - r * 0.36, cx + r * 0.36, cy + r * 0.36], fill=fill)
        return
    d.ellipse([cx - r, cy - r, cx + r, cy + r], fill=fill)
    d.polygon([left, tip, right], fill=fill)
    d.ellipse([cx - r * 0.42, cy - r * 0.42, cx + r * 0.42, cy + r * 0.42], fill=hole)


def save(im, path):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    im.resize((256, 256), Image.LANCZOS).save(path)
    print("wrote", os.path.relpath(path, ROOT))


def app():
    im, d = canvas()
    box = [150, 150, S - 150, S - 150]
    d.rounded_rectangle(box, radius=80, fill=RAISED)
    # The street grid lives on its own layer, clipped to the card's rounded shape.
    grid = Image.new("RGBA", (S, S), (0, 0, 0, 0))
    map_lines(ImageDraw.Draw(grid), tuple(box))
    mask = Image.new("L", (S, S), 0)
    ImageDraw.Draw(mask).rounded_rectangle(box, radius=80, fill=255)
    im.paste(grid, (0, 0), Image.composite(grid, Image.new("RGBA", (S, S)), mask).split()[3])
    d = ImageDraw.Draw(im)
    pin(d, S // 2, 230, 180, hole=RAISED)
    return im


def distribution():
    im, d = canvas()
    nodes = [(250, 760), (512, 840), (774, 760)]
    for n in nodes:
        d.line([(512, 640), n], fill=MUTED, width=22)
    for x, y in nodes:
        d.ellipse([x - 62, y - 62, x + 62, y + 62], fill=RAISED, outline=MUTED, width=20)
    pin(d, 512, 150, 175)
    return im


def sdk():
    im, d = canvas()
    w = 46
    # { }  around a small pin
    for side in (-1, 1):
        x = 512 + side * 300
        xi = 512 + side * 220
        d.line([(xi, 250), (x, 300), (x, 460), (x + side * 60, 512), (x, 564), (x, 724), (xi, 774)],
               fill=MUTED, width=w, joint="curve")
    pin(d, 512, 330, 130)
    return im


def consumer_module():
    im, d = canvas()
    d.rounded_rectangle([210, 210, S - 210, S - 210], radius=70, outline=MUTED, width=30)
    for cx, cy in [(512, 210), (512, S - 210), (210, 512), (S - 210, 512)]:  # puzzle tabs
        d.ellipse([cx - 60, cy - 60, cx + 60, cy + 60], fill=BG, outline=MUTED, width=30)
    pin(d, 512, 300, 125)
    return im


def consumer_app():
    im, d = canvas()
    d.rounded_rectangle([170, 230, S - 170, S - 200], radius=60, fill=RAISED, outline=MUTED, width=24)
    d.line([(170, 330), (S - 170, 330)], fill=MUTED, width=24)
    for i, x in enumerate((240, 300, 360)):
        d.ellipse([x - 18, 262, x + 18, 298], fill=ORANGE if i == 0 else MUTED)
    pin(d, 512, 400, 115, hole=RAISED)
    return im


if __name__ == "__main__":
    save(app(), f"{ROOT}/basecamp/osm-app/src/icons/pramadani_osm_app.png")
    save(distribution(), f"{ROOT}/basecamp/osm-distribution/icons/pramadani_osm_distribution.png")
    save(sdk(), f"{ROOT}/basecamp/osm-sdk/icons/pramadani_osm_sdk.png")
    save(consumer_module(), f"{ROOT}/examples/consumer-module/icons/pramadani_osm_consumer_module.png")
    save(consumer_app(), f"{ROOT}/examples/distribution-consumer-app/src/icons/pramadani_osm_consumer_app.png")
