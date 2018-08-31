; two bytes:
; byte 1 specifies the command
; byte 2 specifies an argument for the command
;
; set audio registers:
; 0x10 - set FF10 ; channel 1
; 0x11 - set FF11
; 0x12 - set FF12
; 0x13 - set FF13
; 0x14 - set FF14
; 0x15 - invalid
; 0x16 - set FF16 ; channel 2
; 0x17 - set FF17
; 0x18 - set FF18
; 0x19 - set FF19
; 0x1A - set FF1A ; channel 3
; 0x1B - set FF1B
; 0x1C - set FF1C
; 0x1D - set FF1D
; 0x1E - set FF1E
; 0x1F - invalid
; 0x20 - set FF20 ; channel 4
; 0x21 - set FF21
; 0x22 - set FF22
; 0x23-0x7F invalid
; 0x80-0xFE - put cool commands here
;
; disable audio:
; 0xFC - set GGBASMAudioEnable to 0
;
; pointer management:
; Use this to loop the song, chain the song across banks or lead into another song.
; 0xFD - set GGBASMAudioBank
; 0xFE - set GGBASMAudioPointerHi and GGBASMAudioPointerLo
;
; stop processing commands, rest for $argument game loops:
; 0xFF - set GGBASMAudioRest

; the commands are arranged so that only set 0xFFXX commands have the first bit 0
; this means we can quickly check the first bit, then use the byte as the address to write to.
; then the remaining commands can be manually checked.

InitSound:
    ; registers
    ld a 0x80 ; 0b10000000
	ld [0xFF00+0x26] a

	ld a 0x77 ; 0b01110111
	ld [0xFF00+0x24] a

    ld a 0xFF ; 0b00000010
    ld [0xFF00+0x25] a

    ; set sound variables
    xor a; ld a 0
    ld [GGBASMAudioEnable] a

    ret

StepSound:
    ; enable
    ld hl GGBASMAudioEnable
    cp [hl]
    ret z

    ; handle rests
    ld hl GGBASMAudioRest
    ld a [hl]
    and a ; cp 0
    jp z doStepSound
    dec [hl]

    ret

doStepSound:
    ; TODO set bank to GGBASMAudioBank

    ; get audio pointer
    ld de GGBASMAudioPointerHi
    ld a [de]
    ld h a
    ld de GGBASMAudioPointerLo
    ld a [de]
    ld l a

processCommand:
    ; load command
    ldi a [hl]
    ld c a
    ; load argument
    ldi a [hl]

audioCommandWriteIO:
    bit 7 c
    jp nz audioCommands
    ld [0xFF00+c] a
    jp processCommand

audioCommands:
    ; the remaining branches use the command so swap a and c
    ld b a
    ld a c ; the command is now a
    ld c b ; the argument is now c

audioCommandRest:
    cp 0xFF
    jp nz audioCommandSetPointer
    ld a c
    ld [GGBASMAudioRest] a
    jp saveProgress

audioCommandSetPointer:
    cp 0xFE
    jp nz audioCommandSetBank
    ld h [hl]
    ld l c
    jp processCommand

audioCommandSetBank:
    cp 0xFD
    jp nz audioCommandDisable
    ld a c
    ld [GGBASMAudioBank] a
    jp processCommand

audioCommandDisable:
    cp 0xFC
    jp nz processCommand
    xor a; ld a 0
    ld [GGBASMAudioEnable] a
    jp processCommand

saveProgress:
    ; save audio pointer
    ld de GGBASMAudioPointerHi
    ld a h
    ld [de] a
    ld de GGBASMAudioPointerLo
    ld a l
    ld [de] a

    ret
