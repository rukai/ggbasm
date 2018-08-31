start:
    ; setup interrupts
	ei
	ld a 0x1
	ld [0xFF00+0xFF] a

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

mainLoop:
    call GGBASMAudioStep
    halt
    jp mainLoop

GGBASMAudioEnable    EQU 0xC020 ; dont process music when 0 otherwise process it
GGBASMAudioBank      EQU 0xC021 ; the bank the currently playing song is stored on
GGBASMAudioPointerLo EQU 0xC022 ; pointer to the currently playing song
GGBASMAudioPointerHi EQU 0xC023
GGBASMAudioRest      EQU 0xC024 ; rest for this many steps
