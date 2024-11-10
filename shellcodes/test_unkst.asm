lbl entry
    addi r5, r4, {{ offset }}
    set64 r6, {{ r6 }}
    st.q r0, r4, r5, 0x008

lbl test
    unk.st {{ opcode }}, r6, r5, {{ stoff14 }}, {{ width }}

lbl result
    st.q r0, r4, r5, 0x010

lbl exit
    ret.d