"""HybridGt: decode saved device-method 0x1c8 descriptions (VICTORY_SAVE_PIPELINES).

decode_1c8.py <dump dir> <native-index.txt>

Per description: whether the VS and PS code headers are in the native index, and the
graphics state read from the 0x184 register block (word 1):
  +0xa0 PA_SU_SC_MODE_CNTL (cull), +0xb8 DB_DEPTH_CONTROL, +0xc0/+0xc4 DB_STENCILREFMASK(_BF),
  +0xcc CB_TARGET_MASK, +0xd0..+0xec CB_BLEND0-7_CONTROL.
Prints counts of distinct states, so the pipeline builder covers what the title asks for.
"""
import collections
import glob
import os
import struct
import sys

dump, index_path = sys.argv[1], sys.argv[2]
index = set()
for line in open(index_path, encoding='utf8'):
    f = line.split()
    if len(f) >= 5 and len(f[0]) == 24:
        index.add(f[0])

ZFUNC = ['never', 'less', 'equal', 'lequal', 'greater', 'notequal', 'gequal', 'always']
BLEND = {0: 'zero', 1: 'one', 2: 'src_c', 3: 'inv_src_c', 4: 'src_a', 5: 'inv_src_a', 6: 'dst_a', 7: 'inv_dst_a',
         8: 'dst_c', 9: 'inv_dst_c', 10: 'src_a_sat', 13: 'const_c', 14: 'inv_const_c', 15: 'src1_c', 16: 'inv_src1_c',
         17: 'src1_a', 18: 'inv_src1_a', 19: 'const_a', 20: 'inv_const_a'}
COMB = ['add', 'sub', 'min', 'max', 'rsub']


def blend(v):
    if not v & (1 << 30):
        return 'off'
    c = f"{BLEND.get(v & 31, v & 31)} {COMB[(v >> 5) & 7] if (v >> 5) & 7 < 5 else '?'} {BLEND.get((v >> 8) & 31, (v >> 8) & 31)}"
    if v & (1 << 29):
        c += f" | a {BLEND.get((v >> 16) & 31, '?')} {COMB[(v >> 21) & 7] if (v >> 21) & 7 < 5 else '?'} {BLEND.get((v >> 24) & 31, '?')}"
    return c


descs = sorted(glob.glob(os.path.join(dump, '1c8-*-desc.bin')))
states = collections.Counter()
found = collections.Counter()
layouts = collections.Counter()
for d in descs:
    call = os.path.basename(d).split('-')[1]
    b = open(d, 'rb').read()
    w = struct.unpack_from('<64Q', b)
    files = {os.path.basename(f).split('-')[2]: f for f in glob.glob(os.path.join(dump, f'1c8-{call}-w*-*'))}
    def block(k):
        f = files.get(f'w{k:02}')
        return open(f, 'rb').read() if f else None
    regs, vs, ps, inputs = block(1), block(5), block(9), block(3)
    vs_in = vs is not None and vs[:12].hex() in index
    ps_in = ps is not None and ps[:12].hex() in index
    found[(vs_in, ps_in)] += 1
    if regs and len(regs) >= 0xf0:
        r = lambda off: struct.unpack_from('<I', regs, off)[0]
        cull = r(0xa0) & 3
        dc = r(0xb8)
        depth = 'off' if not dc & 2 else f"{ZFUNC[(dc >> 4) & 7]}{' write' if dc & 4 else ''}"
        sten = 'stencil' if dc & 1 else ''
        key = (['none', 'front', 'back', 'both'][cull], depth, sten, f"mask {r(0xcc):08x}", blend(r(0xd0)))
        states[key] += 1
    if inputs:
        names = []
        for name in (b'POSITION', b'NORMAL', b'TANGENT', b'COLOR', b'TEXCOORD', b'BLENDWEIGHT', b'BLENDINDICES', b'BINORMAL'):
            if name in inputs:
                names.append(name.decode())
        layouts[' '.join(names) or '-'] += 1

print(f'{len(descs)} descriptions of 0x1c8')
print('VS/PS code headers in the native index:', dict(found))
print('\ndistinct states (cull, depth, stencil, target mask, RT0 blend):')
for k, n in states.most_common(40):
    print(f'{n:5}  {k}')
print('\nvertex layouts (semantics present):')
for k, n in layouts.most_common(20):
    print(f'{n:5}  {k}')
