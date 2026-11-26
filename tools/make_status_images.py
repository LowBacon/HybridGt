"""Draw the README's status badges and progress chart (docs/img/*.svg).

python -I tools/make_status_images.py

Edit BADGES and AREAS below with measured numbers only, then run it and commit the SVGs.
"""
import os

OUT = os.path.join(os.path.dirname(os.path.abspath(__file__)), '..', 'docs', 'img')

# (file, label, value, colour)
BADGES = [
    ('badge-shaders.svg', 'shaders translated', '26,084 / 26,084', '#2e9e4f'),
    ('badge-fps.svg', 'world fps', '6.9 / 30', '#d9822b'),
    ('badge-fsr.svg', 'AMD FSR', '3.1.6 on RTX 4070 Ti', '#2e9e4f'),
    ('badge-raytracing.svg', 'ray tracing', 'builder runs, crashes', '#c0392b'),
    ('badge-tested.svg', 'tested on', 'RTX 4070 Ti · 32 GB', '#555'),
]

# (area, percent done, note). Percentages are measured ratios where one exists
# (programs, fps); otherwise the five-step scale of docs/STATUS.md (20 % a step).
AREAS = [
    ('Shader programs translate', 100.0, '26,084 of 26,084 (with the opt-in switches)'),
    ('Pass Microsoft\'s DXIL validator', 98.4, '25,479 of 25,889 emitted'),
    ('Vertex/pixel pipelines on the card', 98.6, '14,247 of 14,442'),
    ('Compute pipelines on the card', 100.0, '2,682 of 2,682'),
    ('Mesh pipelines on the card', 100.0, '8,765 of 8,765'),
    ('Ray-tracing builder translates', 100.0, '52 of 52 programs'),
    ('Menus at 30 fps', 100.0, '30 fps'),
    ('World frame rate (goal 30)', 23.0, '6.9 fps median, p10 3.7'),
    ('AMD FSR 3.1 upscaling', 80.0, 'runs, 2 bad frames seen'),
    ('Characters and bodies', 100.0, 'whole in the prologue'),
    ('Textures on time', 80.0, 'some still late'),
    ('Interior light (stand-in)', 80.0, 'lit, costs ~1 fps'),
    ('Boat mission', 60.0, 'fix in, not checked'),
    ('Real ray tracing', 60.0, 'runs, title crashes'),
    ('Walls and surfaces textured', 60.0, '15 % fewer textures waiting'),
    ('Clothes changes (bakes)', 60.0, 'bake sources first, not seen yet'),
    ('Mouth animation', 80.0, 'moves; choppy at low fps'),
]


def text_width(s, size=11):
    # Verdana-ish average advance; good enough for badge boxes.
    return int(sum(0.68 if c not in 'il.,:;|1 ' else 0.36 for c in s) * size) + 14


def badge(label, value, colour):
    lw, vw = text_width(label), text_width(value)
    w = lw + vw
    return f'''<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="20" role="img" aria-label="{label}: {value}">
<title>{label}: {value}</title>
<linearGradient id="s" x2="0" y2="100%"><stop offset="0" stop-color="#bbb" stop-opacity=".1"/><stop offset="1" stop-opacity=".1"/></linearGradient>
<clipPath id="r"><rect width="{w}" height="20" rx="3" fill="#fff"/></clipPath>
<g clip-path="url(#r)"><rect width="{lw}" height="20" fill="#555"/><rect x="{lw}" width="{vw}" height="20" fill="{colour}"/><rect width="{w}" height="20" fill="url(#s)"/></g>
<g fill="#fff" text-anchor="middle" font-family="Verdana,Geneva,DejaVu Sans,sans-serif" font-size="11">
<text x="{lw / 2}" y="15" fill="#010101" fill-opacity=".3">{label}</text><text x="{lw / 2}" y="14">{label}</text>
<text x="{lw + vw / 2}" y="15" fill="#010101" fill-opacity=".3">{value}</text><text x="{lw + vw / 2}" y="14">{value}</text>
</g></svg>
'''


def colour_for(p):
    if p >= 99.5:
        return '#2e9e4f'
    if p >= 60:
        return '#7cb342'
    if p >= 40:
        return '#d9822b'
    return '#c0392b'


def chart(areas):
    row, top, left, bar = 30, 46, 250, 380
    h = top + row * len(areas) + 16
    w = left + bar + 230
    out = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}" role="img" aria-label="HybridGt progress">',
           '<style>text{font-family:Segoe UI,Helvetica,Arial,sans-serif;font-size:13px;fill:#24292f}'
           '.t{font-size:16px;font-weight:600}.n{fill:#57606a;font-size:12px}.p{font-weight:600}'
           '@media (prefers-color-scheme:dark){text{fill:#e6edf3}.n{fill:#9198a1}.bg{fill:#30363d}}</style>',
           f'<rect width="{w}" height="{h}" rx="8" fill="none" stroke="#d0d7de"/>',
           '<text x="16" y="28" class="t">HybridGt progress</text>']
    for i, (name, p, note) in enumerate(areas):
        y = top + i * row
        filled = bar * p / 100
        out.append(f'<text x="16" y="{y + 14}">{name}</text>')
        out.append(f'<rect class="bg" x="{left}" y="{y + 2}" width="{bar}" height="16" rx="4" fill="#eaeef2"/>')
        if filled > 0:
            out.append(f'<rect x="{left}" y="{y + 2}" width="{filled:.1f}" height="16" rx="4" fill="{colour_for(p)}"/>')
        shown = f'{p:.0f}' if p == int(p) else f'{p:.1f}'
        out.append(f'<text x="{left + bar + 10}" y="{y + 15}" class="p">{shown} %</text>')
        out.append(f'<text x="{left + bar + 56}" y="{y + 15}" class="n">{note}</text>')
    out.append('</svg>\n')
    return '\n'.join(out)


# World frame rate, median of a run's world frames (3,000+ draws), RTX 4070 Ti, in the order measured.
# (label under the point, fps). Measured runs only: see docs/PERFORMANCE_OPTIMIZATION.md.
FPS_HISTORY = [
    ('start', 3.0),
    ('GPU copies', 3.7),
    ('remap locks', 4.8),
    ('lock-free', 5.8),
    ('fxceless', 6.7),
    ('0x1c8 off', 7.7),
    ('wave-wide', 6.8),
    ('asset room', 6.7),
    ('CPU switches', 6.9),
]


def fps_chart(points, goals=(10, 30)):
    w, h, left, bottom, top = 860, 300, 56, 40, 30
    plot_w, plot_h = w - left - 30, h - top - bottom
    most = max(goals[-1], max(p[1] for p in points))
    x = lambda i: left + plot_w * i / max(1, len(points) - 1)
    y = lambda v: top + plot_h * (1 - v / most)
    out = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}" role="img" aria-label="World frame rate over time">',
           '<style>text{font-family:Segoe UI,Helvetica,Arial,sans-serif;font-size:12px;fill:#24292f}.t{font-size:16px;font-weight:600}'
           '.n{fill:#57606a;font-size:11px}@media (prefers-color-scheme:dark){text{fill:#e6edf3}.n{fill:#9198a1}}</style>',
           f'<rect width="{w}" height="{h}" rx="8" fill="none" stroke="#d0d7de"/>',
           '<text x="16" y="22" class="t">World frame rate (median fps, RTX 4070 Ti)</text>']
    for g in goals:
        out.append(f'<line x1="{left}" x2="{left + plot_w}" y1="{y(g):.1f}" y2="{y(g):.1f}" stroke="#d9822b" stroke-dasharray="6 4"/>')
        out.append(f'<text x="{left + plot_w - 70}" y="{y(g) - 5:.1f}" class="n">goal {g} fps</text>')
    for v in range(0, int(most) + 1, 5):
        out.append(f'<text x="{left - 28}" y="{y(v) + 4:.1f}" class="n">{v}</text>')
        out.append(f'<line x1="{left}" x2="{left + plot_w}" y1="{y(v):.1f}" y2="{y(v):.1f}" stroke="#d0d7de" stroke-opacity=".5"/>')
    path = ' '.join(f'{"M" if i == 0 else "L"}{x(i):.1f},{y(v):.1f}' for i, (_, v) in enumerate(points))
    out.append(f'<path d="{path}" fill="none" stroke="#2e9e4f" stroke-width="2.5"/>')
    for i, (label, v) in enumerate(points):
        out.append(f'<circle cx="{x(i):.1f}" cy="{y(v):.1f}" r="4" fill="#2e9e4f"/>')
        out.append(f'<text x="{x(i) - 10:.1f}" y="{y(v) - 9:.1f}">{v}</text>')
        out.append(f'<text x="{x(i) - 22:.1f}" y="{h - 14}" class="n">{label}</text>')
    out.append('</svg>\n')
    return '\n'.join(out)


def main():
    os.makedirs(OUT, exist_ok=True)
    for name, label, value, colour in BADGES:
        with open(os.path.join(OUT, name), 'w', encoding='utf8') as f:
            f.write(badge(label, value, colour))
    with open(os.path.join(OUT, 'progress.svg'), 'w', encoding='utf8') as f:
        f.write(chart(AREAS))
    with open(os.path.join(OUT, 'fps-history.svg'), 'w', encoding='utf8') as f:
        f.write(fps_chart(FPS_HISTORY))
    print('wrote', len(BADGES), 'badges, progress.svg and fps-history.svg to', os.path.normpath(OUT))


main()
