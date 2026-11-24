# Screen

## Modes

0: ASCII 8
1: ASCII 32
2: no
3: pixel 32

### ASCII 8 mode

Each byte is interpreted as a character on a 96 by 40 character screen.

Settings:
0 Change mode
1 Set data offset
2 Set foreground color
4 Set background color


### ASCII 32 mode

Each set of 4 bytes is interpreted as a character, a foreground color and a background color
like so:
CCRGBRGB

this gives one byte to the character, and 4 bits for each color of the foreground and background



### Pixel 32 mode

Every 4 bytes are interpreted as a pixel color, with one byte each for red, green and blue, and a final alpha byte:
RRGGBBAA

Pixel mode resolutions are 4:3 aspect ratio, from 4x3 all the way to 1024x768 (vram size limitations might make some resolutions unavailable).
The resolution is calculated based on the setting as follows:
4 x (setting + 1) : 3 x (setting + 1)

Settings:
0 Change mode
1 Set data offset
2 Set resolution
