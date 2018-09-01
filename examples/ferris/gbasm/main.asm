start:
    di
    ; TODO: load graphics

    call GGBASMAudioInit

    ; start playing ferris theme
    ld hl GGBASMAudioEnable
    ld [hl] 0x01
    ld hl GGBASMAudioBank
    ld [hl] 0x00
    ld hl GGBASMAudioPointerHi
    ld [hl] MusicFerrisTheme / 0x100
    ld hl GGBASMAudioPointerLo
    ld [hl] MusicFerrisTheme % 0x100
    ld hl GGBASMAudioRest
    ld [hl] 0x20

    ; setup interrupts
	ei
	ld a 0x1
	ld [0xFF00+0xFF] a

mainLoop:
    call GGBASMAudioStep

    halt
    ; quick, draw graphics now
    ; TODO

    jp mainLoop
