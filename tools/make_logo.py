"""Draw HybridGt's logo: an original mark, made entirely here (no font, no outside art).

python -I tools/make_logo.py

A rounded tile with a blue-violet-pink sunset gradient, a soft sun glow, procedural palm
fronds, and a slanted, glowing "H" monogram with two speed lines. Writes:
  crates/hgt-launcher/assets/logo.png      1024x1024, for the README and the launcher
  crates/hgt-launcher/assets/logo-256.png  256x256, the window icon and the brand mark
  crates/hgt-launcher/assets/hybridgt.ico  16-256 px, the executable's icon
"""
import math
import os

import numpy as np
from PIL import Image, ImageChops, ImageDraw, ImageFilter

N = 1024
OUT = os.path.join(os.path.dirname(os.path.abspath(__file__)), '..', 'crates', 'hgt-launcher', 'assets')


def lerp(a, b, t):
    return tuple(int(round(a[i] + (b[i] - a[i]) * t)) for i in range(len(a)))


def gradient():
    """Diagonal sunset: deep blue top-left, violet, hot pink bottom-right."""
    y, x = np.mgrid[0:N, 0:N].astype(np.float32) / (N - 1)
    t = np.clip(0.62 * y + 0.38 * x, 0, 1)
    stops = [(0.0, (22, 36, 214)), (0.45, (112, 44, 236)), (0.78, (214, 38, 190)), (1.0, (255, 54, 150))]
    img = np.zeros((N, N, 3), np.float32)
    for (t0, c0), (t1, c1) in zip(stops, stops[1:]):
        m = (t >= t0) & (t <= t1)
        k = ((t - t0) / (t1 - t0))[..., None]
        img[m] = (np.array(c0) + (np.array(c1) - np.array(c0)) * k)[m]
    # A soft sun glow low on the horizon.
    d = np.hypot(x - 0.62, y - 0.80)
    glow = np.exp(-(d / 0.30) ** 2)[..., None]
    img = img + glow * np.array([90, 40, 70]) * 0.9
    return Image.fromarray(np.clip(img, 0, 255).astype(np.uint8), 'RGB').convert('RGBA')


def frond(draw, base, angle, length, droop, width, fill):
    """One palm frond: a curved spine with leaflets on both sides, tapering to the tip."""
    pts = []
    steps = 60
    for i in range(steps + 1):
        s = i / steps
        a = angle + droop * s * s
        px = base[0] + math.cos(a) * length * s
        py = base[1] + math.sin(a) * length * s
        pts.append((px, py, a, s))
    for i in range(1, steps, 2):
        px, py, a, s = pts[i]
        leaf = width * (1 - s) * (0.55 + 0.45 * math.sin(s * math.pi))
        for side in (-1, 1):
            la = a + side * (1.05 - 0.35 * s)
            tip = (px + math.cos(la) * leaf, py + math.sin(la) * leaf)
            nxt = pts[min(i + 2, steps)]
            draw.polygon([(px, py), tip, (nxt[0], nxt[1])], fill=fill)
    draw.line([(p[0], p[1]) for p in pts], fill=fill, width=max(3, int(width * 0.06)))


def palms():
    layer = Image.new('RGBA', (N, N), (0, 0, 0, 0))
    d = ImageDraw.Draw(layer)
    dark = (38, 10, 92, 215)
    # Top right: fronds hanging in from the corner.
    for k, (ang, ln) in enumerate([(2.35, 560), (2.75, 520), (3.1, 470), (1.95, 500), (2.55, 430)]):
        frond(d, (N + 40, -30), ang, ln, 0.55 + 0.08 * k, 120, dark)
    # Bottom left: fronds rising from the corner.
    for k, (ang, ln) in enumerate([(-0.45, 560), (-0.85, 520), (-0.1, 470), (-1.2, 430)]):
        frond(d, (-40, N + 30), ang, ln, -0.5 - 0.07 * k, 120, (60, 12, 110, 205))
    return layer.filter(ImageFilter.GaussianBlur(1.2))


def monogram():
    """A slanted H of three rounded bars, and two speed lines: white, with a pink glow."""
    shape = Image.new('L', (N, N), 0)
    d = ImageDraw.Draw(shape)
    r = 26
    left, right, top, bottom, bar = 300, 724, 270, 754, 118
    d.rounded_rectangle([left, top, left + bar, bottom], r, fill=255)
    d.rounded_rectangle([right - bar, top, right, bottom], r, fill=255)
    d.rounded_rectangle([left + 40, 455, right - 40, 455 + 104], r, fill=255)
    # Speed lines out of the crossbar, to the left.
    d.rounded_rectangle([150, 470, 262, 494], 12, fill=255)
    d.rounded_rectangle([190, 522, 270, 544], 11, fill=255)
    # Slant it: a shear about the centre.
    shear = 0.18
    shape = shape.transform((N, N), Image.AFFINE, (1, shear, -shear * N / 2, 0, 1, 0), resample=Image.BICUBIC)
    glow_wide = shape.filter(ImageFilter.GaussianBlur(38))
    glow_near = shape.filter(ImageFilter.GaussianBlur(12))
    out = Image.new('RGBA', (N, N), (0, 0, 0, 0))
    for mask, colour, gain in [(glow_wide, (255, 80, 220), 1.6), (glow_near, (255, 150, 245), 1.4)]:
        m = mask.point(lambda v: min(255, int(v * gain)))
        out = Image.alpha_composite(out, Image.merge('RGBA', (*[Image.new('L', (N, N), c) for c in colour], m)))
    white = Image.merge('RGBA', (Image.new('L', (N, N), 255), Image.new('L', (N, N), 250), Image.new('L', (N, N), 255), shape))
    return Image.alpha_composite(out, white)


def rounded_mask(size, radius):
    m = Image.new('L', (size * 4, size * 4), 0)
    ImageDraw.Draw(m).rounded_rectangle([0, 0, size * 4 - 1, size * 4 - 1], radius * 4, fill=255)
    return m.resize((size, size), Image.LANCZOS)


def main():
    os.makedirs(OUT, exist_ok=True)
    img = gradient()
    img = Image.alpha_composite(img, palms())
    img = Image.alpha_composite(img, monogram())
    img.putalpha(ImageChops.multiply(img.getchannel('A'), rounded_mask(N, 200)))
    img.save(os.path.join(OUT, 'logo.png'), optimize=True)
    img.resize((256, 256), Image.LANCZOS).save(os.path.join(OUT, 'logo-256.png'), optimize=True)
    img.save(os.path.join(OUT, 'hybridgt.ico'), sizes=[(16, 16), (24, 24), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)])
    print('wrote logo.png, logo-256.png and hybridgt.ico to', os.path.normpath(OUT))


if __name__ == '__main__':
    main()
