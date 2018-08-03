;*****************************************
;* cartridge header
;*****************************************
RST_00: 
    jp $100

    advance_address 08h
RST_08: 
    jp $100

    advance_address 10h
RST_10:
    jp $100

    advance_address 18h
RST_18:
    jp $100

    advance_address 20h
RST_20:
    jp $100

    advance_address 28h
RST_28:
    jp $100

    advance_address 30h
RST_30:
    jp $100

    advance_address 38h
RST_38:
    jp $100

    advance_address 40h
VBL_VECT:
    reti
    
    advance_address 48h
LCD_VECT:
    reti

    advance_address 50h
TIMER_VECT:
    reti

    advance_address 58h
SERIAL_VECT:
    reti

    advance_address 60h
JOYPAD_VECT:
    reti
    
    advance_address 100h
    nop
    jp Start

    ; $0104-$0133 (Nintendo logo - do _not_ modify the logo data here or the GB will not run the program)
    DB CEh,EDh,66h,66h,CCh,0Dh,00h,0Bh,03h,73h,00h,83h,00h,0Ch,00h,0Dh
    DB 00h,08h,11h,1Fh,88h,89h,00h,0Eh,DCh,CCh,6Eh,E6h,DDh,DDh,D9h,99h
    DB BBh,BBh,67h,63h,6Eh,0Eh,ECh,CCh,DDh,DCh,99h,9Fh,BBh,B9h,33h,3Eh

    ; $0134-$013E (Game title - up to 11 upper case ASCII characters; pad with $00)
    DB "ASM Header",0
       ;0123456789A

    ; $013F-$0142 (Product code - 4 ASCII characters, assigned by Nintendo, just leave blank)
    DB "    "
     ;0123

    ; $0143 (Color GameBoy compatibility code)
    DB 00h ; $00 - DMG 
      ; $80 - DMG/GBC
      ; $C0 - GBC Only cartridge

    ; $0144 (High-nibble of license code - normally $00 if $014B != $33)
    DB 00h

    ; $0145 (Low-nibble of license code - normally $00 if $014B != $33)
    DB 00h

    ; $0146 (GameBoy/Super GameBoy indicator)
    DB 00h ; $00 - GameBoy

    ; $0147 (Cartridge type - all Color GameBoy cartridges are at least $19)
    DB 19h ; $19 - ROM + MBC5

    ; $0148 (ROM size)
    DB 01h ; $01 - 512Kbit = 64Kbyte = 4 banks

    ; $0149 (RAM size)
    DB 00h ; $00 - None

    ; $014A (Destination code)
    DB 00h ; $01 - All others
      ; $00 - Japan

    ; $014B (Licensee code - this _must_ be $33)
    DB 33h ; $33 - Check $0144/$0145 for Licensee code.

    ; $014C (Mask ROM version)
    DB 00h

    ; $014D (Complement check)
    DB 00h // TODO

    ; $014E-$014F (Cartridge checksum)
    DW 00h

;*****************************************
;* Program Start
;*****************************************

    advance_address 150h
Start:
    ; TODO: Init

Loop:
    ; TODO: Game loop
    halt
    jp Loop
