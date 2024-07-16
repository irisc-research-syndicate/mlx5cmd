lbl entry
    set64 r5, {{ r5 }}
    set64 r6, {{ r6 }}
    set64 r7, 0

lbl test
    subs r0, r5, r6
    csr.r r7, zero, {{ csrreg }}

lbl result
    st.q r0, r4, r5, 0x08
    st.q r0, r4, r6, 0x10
    st.q r0, r4, r7, 0x18

lbl exit
    ret.d