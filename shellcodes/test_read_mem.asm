lbl entry
    set32 r5, {{ address }}
    ld.q r6, r5, 0
    st.q r0, r4, r6, 0x18
    ret.d