lbl entry
    addi r5, r4, {{ bufoff16 }}
    set64 r6, {{ r6 }}

lbl test
    st.b r6, r5, {{ off16 }}

lbl result

lbl exit
    ret.d