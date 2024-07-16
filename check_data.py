import os
import json

class Continue(Exception): pass

def qwords(*data):
    dwords = []
    for d in data:
        dwords += [(d >> 32) & 0xffff_ffff, d & 0xffff_ffff]
    return dwords

def sx(width, word): return word - (1 << width) if word & (1 << (width - 1)) else word

def alu_skip(r5, r6): raise Continue
def alu_zero(r5, r6): return 0
def alu_one(r5, r6): return 1
def alu_add64(r5, r6): return r6 + r5
def alu_sub64(r5, r6): return r6 - r5
def alu_or64(r5, r6): return r5 | r6
def alu_and64(r5, r6): return r5 & r6
def alu_xor64(r5, r6): return r5 ^ r6
def alu_shl(r5, r6): return r5 << (min(r6, 64))
def alu_shr64(r5, r6): return r5 >> r6
def alu_shr32(r5, r6): return (r5 & 0xffff_ffff) >> r6
def alu_sar64(r5, r6): return sx(64, r5) >> r6 & 0xffff_ffff_ffff_ffff
def alu_sar32(r5, r6): return ((sx(32, r5 & 0xffff_ffff)) >> r6) & 0xffff_ffff
def alu_sar32_i(r5, r6): return alu_sar32(r5, 6)
def alu_0x81(r5, r6): return r5 << 6
def alu_shr32_i(r5, r6): return (r5 & 0xffff_ffff) >> 6
def alu_usar64_i(r5, r6): return (sx(64, r5) >> 32) >> 6
def alu_ushr32_i(r5, r6): return (r5 >> 32) >> 6
def alu_shr64_i(r5, r6): return r5 >> 6
def alu_sar64_i(r5, r6): return sx(64, r5) >> 6
def alu_add24(r5, r6): return (r6 + r5) & 0xff_ffff
def alu_sub24(r5, r6): return (r6 - r5) & 0xff_ffff

aluops = {
    0x00: alu_add64,
    0x01: alu_add64,
    0x04: alu_sub64,
    0x05: alu_sub64,
    0x08: alu_or64,
    0x0b: alu_and64,
    0x0e: alu_xor64,
    0x0f: alu_xor64,

    0x20: alu_skip,

    0x30: alu_add24,
    0x31: alu_sub24,
    0x39: alu_or64,

    0x40: alu_add64,
    0x41: alu_add64,
    0x43: alu_sub64,
    0x44: alu_sub64,
    0x45: alu_sub64,
    0x46: alu_sub64,
    0x47: alu_sub64,

    0x50: alu_add64,
    0x51: alu_add64,

    0x70: alu_skip,
    0x71: alu_skip,
    0x72: alu_skip,
    0x73: alu_skip,
    0x7f: alu_skip,

    0x80: alu_shl,
    0x81: alu_0x81,
    0x82: alu_shr32,
    0x83: alu_shr32_i,
    0x84: alu_sar32,
    0x85: alu_sar32_i,
    0x87: alu_shr64,
    0x88: alu_sar64,
    0x89: alu_usar64_i,
    0x8a: alu_shr64_i,
    0x8b: alu_ushr32_i,
    0x8c: alu_sar64_i,

    0x90: alu_skip,
}

ALU_MEMORY_OPS = [
                0x02, 0x03,             0x06, 0x07,       0x09, 0x0a,       0x0c, 0x0d,
    0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f,
                                        0x86,                                     0x8d, 0x8e, 0x8f,
]

for funct in ALU_MEMORY_OPS: aluops[funct] = alu_skip

def unki_skip(r5, simm16): raise Continue
def unki_zero(r5, simm16): return 0
def unki_addi(r5, simm16): return (simm16 + sx(64, r5))
def unki_addihi(r5, simm16): return ((simm16 << 16) + sx(64, r5))
def unki_subi(r5, simm16): return (simm16 - sx(64, r5))
def unki_seti0(r5, simm16): return r5 | (simm16 << 48)
def unki_seti1(r5, simm16): return r5 | (simm16 << 32)
def unki_seti3(r5, simm16): return r5 | (simm16 << 0)
def unki_seti2(r5, simm16): return r5 | (simm16 << 16)
def unki_andi(r5, simm16): return r5 & (0xffff_ffff_ffff_0000 | simm16)
def unki_andi_zx(r5, simm16): return r5 & simm16
def unki_andihi(r5, simm16): return r5 & ((simm16 << 16) | 0xffff_ffff_0000_ffff)
def unki_andihi_zx(r5, simm16): return r5 & (simm16 << 16)
def unki_xor(r5, simm16): return r5 ^ simm16
def unki_addi3(r5, simm16): return r5 + (simm16 << 48)
def unki_addi2(r5, simm16): return r5 + (simm16 << 32)
def unki_addi24(r5, simm16): return (r5 + simm16) & 0xff_ffff
def unki_subi24(r5, simm16): return (simm16 - r5) & 0xff_ffff

def unki_mask(r5, simm16):
    value = (r5 >> ((simm16 >> 5) & 0x3f))
    mask = (1 << (simm16 & 0x1f)) - 1 if simm16 & 0x1f != 0 else 0xffff_ffff
    return value & mask

def unki_0x33(r5, simm16):
    shift1 = simm16 & 0x1f
    shift2 = (simm16 >> 5) & 0x3f
    mask = ~((1 << shift1) - 1 if shift1 != 0 else 0xffff_ffff)
    mask = (mask << shift2) | ((1 << shift2) - 1)
    mask &= 0xffff_ffff_ffff_ffff
    return (r5 & mask)


unki_ops = {
    0x00: unki_addi,
    0x01: unki_addi,
    0x02: unki_addihi,
    0x03: unki_addihi,
    0x04: unki_subi,
    0x05: unki_subi,
    0x06: unki_seti0,
    0x07: unki_seti1,
    0x08: unki_seti3,
    0x09: unki_seti2,
    0x0a: unki_andi,
    0x0b: unki_andi_zx,
    0x0c: unki_andihi,
    0x0d: unki_andihi_zx,
    0x0e: unki_xor,
    0x20: unki_addi3,
    0x21: unki_addi3,
    0x22: unki_addi2,
    0x23: unki_addi2,
    0x30: unki_addi24,
    0x31: unki_subi24,
    0x32: unki_mask,
    0x33: unki_0x33,
    0x34: unki_skip,
    0x37: unki_skip,
}

def unkr_skip(r5, r6, simm11): raise Continue
def unkr_zero(r5, r6, simm11): return 0
def unkr_from_unki(unki_func): return lambda r5, r6, simm11: unki_func(r5, (6 << 11) | simm11)

def unkr_0x33(r5, r6, simm16):
    shift1 = simm16 & 0x1f
    shift2 = (simm16 >> 5) & 0x3f
    mask = ~((1 << shift1) - 1 if shift1 != 0 else 0xffff_ffff)
    mask = (mask << shift2) | ((1 << shift2) - 1)
    mask &= 0xffff_ffff_ffff_ffff
    return (r5 & mask) | (r6 & ~mask)

def unkr_0x34(r5, r6, simm11):
    #if r5 >> 48 == 0xffff: raise Continue
    #if r6 >> 48 == 0xffff: raise Continue
    shift1 = simm11 & 0x1f
    shift2 = (simm11 >> 5) & 0x3f
    mask = 0xffff_ffff_0000_0000 if shift1 == 0 else 0xffff_ffff_ffff_ffff << shift1
    mask = (mask << shift2) | ((1 << shift2) - 1)
    mask &= 0xffff_ffff_ffff_ffff
    print(f"{hex(mask)=} {hex(r5)=} {hex(r6)=} {shift1=} {shift2=} {simm11=}")
#    if simm11 == 0: return (r5 & mask) | 0x0000_000c
#    if simm11 == 2: return (r5 & mask) | 0xffff_fff4
#    if simm11 == 3: return (r5 & mask) | 0xffff_fff4
    raise Continue
    return 6 << (shift1 + 1)
    return (r5 & mask) | (r6 & ~mask)

unkr_ops = {
    0x00: unkr_from_unki(unki_addi),
    0x01: unkr_from_unki(unki_addi),
    0x02: unkr_from_unki(unki_addihi),
    0x03: unkr_from_unki(unki_addihi),
    0x04: unkr_from_unki(unki_subi),
    0x05: unkr_from_unki(unki_subi),
    0x06: unkr_from_unki(unki_seti0),
    0x07: unkr_from_unki(unki_seti1),
    0x08: unkr_from_unki(unki_seti3),
    0x09: unkr_from_unki(unki_seti2),
    0x0a: unkr_from_unki(unki_andi),
    0x0b: unkr_from_unki(unki_andi_zx),
    0x0c: unkr_from_unki(unki_andihi),
    0x0d: unkr_from_unki(unki_andihi_zx),
    0x0e: unkr_from_unki(unki_xor),
    0x0f: unkr_from_unki(unki_zero),
    0x32: unkr_from_unki(unki_mask),
    0x33: unkr_0x33,
    0x34: unkr_0x34,
}

def test_alu(funct, r5, r6):
    return qwords(r5, r6, aluops.get(funct, alu_zero)(r5, r6))

def test_unki(opcode, r5, simm16):
    return qwords(r5, 0, unki_ops.get(opcode, unki_zero)(r5, simm16))

def test_unkr(opcode, r5, r6, simm11):
    return qwords(r5, r6, unkr_ops.get(opcode, unkr_zero)(r5, r6, simm11))

def read_json_lines(path):
    with open(path, "r") as f:
        return [json.loads(line) for line in f]

def verify_data(func):
    for p in os.listdir("data"):
        if not p.startswith(func.__name__): continue

        lines = read_json_lines(f"data/{p}")
        experiment = lines[0]
        for data in lines[1:]:
            try:
                actual = data["results"]
                actual_r5 = (actual[0] << 32) | actual[1]
                actual_r6 = (actual[2] << 32) | actual[3]
                actual_r7 = (actual[4] << 32) | actual[5]

                expect = func(**data["parameters"])
                expect_r5 = (expect[0] << 32) | expect[1]
                expect_r6 = (expect[2] << 32) | expect[3]
                expect_r7 = (expect[4] << 32) | expect[5]

                if any(e != a and e != None for e, a in zip(expect, actual)):
                    print(f"Counter example to {func.__name__} found in {p}:")
                    print(f"parameters={data['parameters']}")
                    print(f"expect={expect_r5:016x}:{expect_r6:016x}:{expect_r7:016x}")
                    print(f"actual={actual_r5:016x}:{actual_r6:016x}:{actual_r7:016x}")
                    exit(0)
            except Continue:
                if data["parameters"].get("opcode") == 0x34:
                    print(f"actual={actual_r5:016x}:{actual_r6:016x}:{actual_r7:016x}")
                continue

if __name__ == "__main__":
    #verify_data(test_alu)
    verify_data(test_unkr)
    #verify_data(test_unki)
