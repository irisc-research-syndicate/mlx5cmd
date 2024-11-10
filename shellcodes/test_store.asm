lbl entry
    addi r5, r4, 0x10
    set64 r6, {{ r6 }}

lbl test
    unk.r {{ opcode }}, r{{ rd }}, r5, r6, {{ uimm11 }}

lbl result

lbl exit
    ret.d