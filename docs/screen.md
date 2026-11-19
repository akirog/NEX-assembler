# Screen

## Modes

0: ascii 8
1: n0
2: no
3: pixel 32

### Ascii 8 mode

Each byte is interpreted as a character on a 96 by 40 character screen.

Settings:
0 Change mode
1 Set data offset
2 Set foreground color
4 Set font


### Pixel 32 mode

Every 4 bytes are interpreted as a pixel color, with one byte each for red, green and blue, and a final alpha byte:
RRRRRRRRGGGGGGGGBBBBBBBBAAAAAAAA

Pixel mode resolutions are 4:3 aspect ratio, from 4x3 all the way to 1024x768 (vram size limitations might make some resolutions unavailable).
The resolution is calculated based on the setting as follows:
4 x (setting + 1) : 3 x (setting + 1)

Settings:
0 Change mode
1 Set data offset
2 Set resolution

## Ascii fonts

The screen has 4 ascii fonts

0 Art   (cp437)
1 Text  (cp850)
2 APL   (cp910)
3 Cyrillic (cp866)
