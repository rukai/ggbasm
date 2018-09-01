GGBASMAudioEnable    EQU 0xC020 ; dont process music when 0 otherwise process it
GGBASMAudioBank      EQU 0xC021 ; the bank the currently playing song is stored on
GGBASMAudioPointerLo EQU 0xC022 ; pointer to the currently playing song
GGBASMAudioPointerHi EQU 0xC023
GGBASMAudioRest      EQU 0xC024 ; rest for this many steps
