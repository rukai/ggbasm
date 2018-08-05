start:
    halt

    ; because all *.asm files are imported with a bank specified by rust code,
    ; the advance_address instruction refers to a position within the bank of the *.asm file.
    advance_address 0x200
the_real_start:
    stop
    nop
    jp the_real_start
