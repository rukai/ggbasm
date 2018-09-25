;*****************************************
;* cartridge header
;*****************************************

RST_00: 
    jp 0x100

    advance_address 0x8
RST_08: 
    jp 0x100

    advance_address 0x10
RST_10:
    jp 0x100

    advance_address 0x18
RST_18:
    jp 0x100

    advance_address 0x20
RST_20:
    jp 0x100

    advance_address 0x28
RST_28:
    jp 0x100

    advance_address 0x30
RST_30:
    jp 0x100

    advance_address 0x38
RST_38:
    jp 0x100

    advance_address 0x40
VBL_VECT:
    reti
    
    advance_address 0x48
LCD_VECT:
    reti

    advance_address 0x50
TIMER_VECT:
    reti

    advance_address 0x58
SERIAL_VECT:
    reti

    advance_address 0x60
JOYPAD_VECT:
    reti
    
    advance_address 0x100
    nop
    jp Start

    ; 0x0104-0x0133 (Nintendo logo - do _not_ modify the logo data here or the GB will not run the program)
    DB 0xCE, 0xED, 0x66, 0x66, 0xCC, 0x0D, 0x00, 0x0B, 0x03, 0x73, 0x00, 0x83, 0x00, 0x0C, 0x00, 0x0D
    DB 0x00, 0x08, 0x11, 0x1F, 0x88, 0x89, 0x00, 0x0E, 0xDC, 0xCC, 0x6E, 0xE6, 0xDD, 0xDD, 0xD9, 0x99
    DB 0xBB, 0xBB, 0x67, 0x63, 0x6E, 0x0E, 0xEC, 0xCC, 0xDD, 0xDC, 0x99, 0x9F, 0xBB, 0xB9, 0x33, 0x3E

    ; 0x0134-0x013E (Game title - up to 11 upper case ASCII characters; pad with 0)
    DB "ASM Header", 0
       ;0123456789A

    ; 0x013F-0x0142 (Product code - 4 ASCII characters, assigned by Nintendo, just leave blank)
    DB "    "
     ;0123

    ; 0x0143 (Color GameBoy compatibility code)
    DB 0x00 ; 0x00 - DMG 
            ; 0x80 - DMG/GBC
            ; 0xC0 - GBC Only cartridge

    ; 0x0144 (High-nibble of license code - normally 0x00 if [0x014B] != 0x33)
    DB 0x00

    ; 0x0145 (Low-nibble of license code - normally 0x00 if [0x014B] != 0x33)
    DB 0x00

    ; 0x0146 (GameBoy/Super GameBoy indicator)
    DB 0x00 ; 0x00 - GameBoy

    ; 0x0147 (Cartridge type - all Color GameBoy cartridges are at least 0x19)
    DB 0x19 ; 0x19 - ROM + MBC5

    ; 0x0148 (ROM size)
    DB 0x01 ; 0x01 - 512Kbit = 64Kbyte = 4 banks

    ; 0x0149 (RAM size)
    DB 0x00 ; 0x00 - None

    ; 0x014A (Destination code)
    DB 0x00 ; 0x01 - All others
            ; 0x00 - Japan

    ; 0x014B (Licensee code - this _must_ be 0x33)
    DB 0x33 ; 0x33 - Check 0x0144/0x0145 for Licensee code.

    ; 0x014C (Mask ROM version)
    DB 0x00

    ; 0x014D (Complement check)
    DB 0x00 ; TODO: You will have to do this manually.

    ; 0x014E-0x014F (Cartridge checksum)
    DW 0x00

;*****************************************
;* Program Start
;*****************************************

    advance_address 0x150
Start:
    ; TODO: Put your init code here

Loop:
    ; TODO: Put your game loop here
    jp Loop
