i wanna start by saying that custom commands are never supposed to crash the game, even if you made a "mistake" with using them. so please let me know if they do.

so for 66_10 - 66_18, i gotta explain what a register is. a register is like, a custom variable basically, it's a place where you can store whatever you want. there's 128 registers for storing integers (whole numbers) and 128 registers for storing floating point numbers (numbers with decimals)

these are numbered
00-7f: integer registers
80-ff: floating point registers.
so basically if it starts with an 8, 9, a,b,c,d,e,f it's a floating point register.

registers all default to storing 0 at the start of a game. also note that any registers will never get set to an invalid number like infinity or "[not a number](https://en.wikipedia.org/wiki/NaN)" if you do something like division by 0, instead they'll be set back to 0. if you notice something being set to NaN or infinity, please report it as a bug.

also, for all of these, any time you use one of these, the "condition register" gets set to the result, which is the thing used for the game's existing conditional commands like 0_02. i only tested 0_02 and 0_08 but the other should probably work too. so this is gonna be big way to get more complicated logics

## 66_10, load immediate into register.
immediate is just the term for like a number that you typed in yourself, and not like a variable
```
66000000
10000000
000000FF
9999d93F
```

so the format here is, that 3rd line is 00 00 00 are ignored, and then that FF is the register. so we're using a floating point register.
the 9999d93F is 1.7 in floating point of course

so this sets register FF to 1.7 !

okay so let's say you wanna do some math with this.

## 66_11 is binary operation with register and immediate
```
66000000
11000000
02000000
FF000033
04000000
```

"binary operation" means any operation where there's two inputs to it.
the 02000000 means multiply (there'll be a full list at the bottom of this of what operations there are)

FF is the left hand side register. the 00 00 is ignored. then 33 is the destination register. notice how 33 is an integer register, not a floating point register. this is how the command decides what format the immediate should be.

then that 04000000 is the immediate. again, integer, because register 33 is integer.
so this command means:

register[33] = register[FF] * 4

so if we do it after that command i listed first, then it's going to do 1.8 * 4, which the answer is 6.8. to fit that into an integer register, it'll chop that .8 off, and become just 6 stored in register 33.

## 66_12 is binary operation with register and register
```
66000000
12000000
c0000000
FFEE00e0
```
so, c0 is an equality check. it has some fuzziness (within 0.000001) when comparing two floating points for complicated reasons because floating point math is imprecise, but you should basically treat it as an equality check.

the FF is the left hand side register
the EE is the right hand side register
00 here is ignored
e0 is the destination

so let's say FF and EE had both 1.7 in them, then register E0 gets set to 1.0 (true)
otherwise register E0 gets set to 0.0 (false)

## 66_13 is a unary operation with a register
```
66000000
13000000
11000000
FF0000AA
```
a unary operation is just one number involved
operation 11 is sine. a full list of em should be below.

FF is the input register, AA is the destination
so
register[AA] = sin(register[FF])

## 66_14 is a unary operation with an immediate
```
66000000
14000000
10000000
00000099
40000000
```
i think at this point you know the deal
operation 10 is square root
register 99 is the destination
40 is hex for 64
so
register[99] = sqrt(64)
so register[99] = 8

## 66_15 is where we get into some more juicy stuff again
load variable into register
```
66000000
15000000
00000023
40000000
```
so. ignore those first three 00s. then register 23 is our destination

that 40000000 is the variable. in this case 40 means your super meter. there's a list of variables at the end of this document

so let's say you have 5 bars, then register 23 will get set to 50000.


to get an opponent's variable you use:
```
66000000
15000000
008000A4
21000000
```
same format again, except on the third line that "80" controls it being the opponent.

so now register A4 has the opponent's Y position (21000000)
this always uses their point character.

you can control your teammates' stuff too with these values:
```rs
    /// Current executing character. if on opponent's team this will also be the point. For child characters, some variables like health will be on actual playable character that summoned them
    Me = 0x00,
    /// My team's point character. This can be the current character.
    Point = 0x01,
    /// Assist 1. if character is dead then get 0s. This can be the current character.
    Assist1NoFallBack = 0x02,
    /// Assist 2. if character is dead then get 0s. This can be the current character.
    Assist2NoFallBack = 0x03,
    /// Assist 1. if character is dead then get point. This can be the current character.
    Assist1WithFallback = 0x04,
    /// Assist 2. if character is dead then get assist 1. if assist 1 is dead get point. This can be the current character.
    Assist2WithFallback = 0x05,
    /// If current (or parent) is point character, then assist 1. If current (or parent) is assist 1, then point character. If only 1 char left, then just get 0s.
    Char1NotMe = 0x06,
    /// If this character is a child, get the parent-most parent (or parent's parent, etc), otherwise just get the current character.
    TrueAncestor = 0x07,
    /// If this character is a child, get the parent, otherwise just get the current character.
    Parent = 0x08,
```
to get the opponent's stuff, just add 80, so your opponent's Assist 1 would be 82.

So to load your assist 1's health into register 44, you'd do:
```
66000000
15000000
00020044
10000000
```

## 66_16 is storing a register back into a game variable
```
66000000
16000000
000000FF
30000000
```
same format, 00 00 00 there doesn't mean anything, FF is the register, 30000000 is the variable (the special air action counter).

so if register FF has 1.7 in it, then that gets converted to an integer (since the counter is an integer), then it gets set to 1. this means you can do 2 more specials in the air.

and to store into your opponent's stuff:
```
66000000
16000000
00800033
10000000
```
same format again
so now the opponent's yellow health (10000000) has been set to whatever number is in register 33.
this always uses their point character.

## 66_17 is storing a variable from an immediate value
this works pretty much the same as 66_16 except you provide a value

```
66000000
17000000
00820000
20000000
0000803f
```
This one sets the opponent's assist 1 (the 82, see the description of above commands)'s x position (the 20000000, see down at the bottom XPosition), to 1.0 (the 0000803f).

the type (floating point or integer) of the immediate is determined by the type of the variable.

## 66_18 is a binary operation with a variable and a register
again, similar to previous commands.

```
66000000
18000000
11000000
FF010000
12000000
```
This example sets your point character (the 01)'s maximum health (the 12), to the current maximum health minus register FF (the FF).

in short:
point character's new maximum health = point character's old maximum health - register[FF]

## 66_19 is a binary operation with a variable and an immediate value
again, similar to the previous command, this time with a value you provide instead of a register

```
66000000
19000000
b0000000
00810000
F2000000
FBFFFFFF
```
This example sets your opponent's point character (the 80)'s invincibility flags (the F2) to the existing flags bitwise-and (the B0) 0xFFFFFFFB (the FBFFFFFF). 

in short:
opponent character's new flags = opponent character's old flags & 0xFFFFFFFB

(this turns off an opponent character's invincibility, if you wanted to do that for some reason. bit cursed).

## 66_1a is a unary operation with a variable

```
66000000
1A000000
10000000
00800000
40000000
```
this example takes sets the opponent's (80) meter (40) to the square root (10) of itself

in short:
opponent's new meter = square root(opponent's old meter)

## 66_1b is detecting characters by id name

66_1B will detect what character is your opponent or on your team by id name (the `CharacterID` from your `Characters.ini`).

The first byte should be 00.
The second byte tells mag_patch which character in the current game to look at (see 66_15 for a list of characters)
The third byte should be 00.
The fourth byte is which register.
Then there is the name as a string, up to 64 bytes, and should end in 00s

```
66000000
1B000000
00800017
47756900
00000000
00000000
00000000
00000000
00000000
00000000
00000000
00000000
00000000
00000000
00000000
00000000
00000000
00000000
00000000
```

The above example will check if the opponent's character (80) is named `Gui` and load it into register 17. The result will be 1 if it is `Gui`, and 0 if it isn't. This works with any modded character.


## Float replacement
You should be able to replace any floating point value in another command with a register by just putting XXFFFFFF instead of the float. This doesn't work with integers unfortunately.

so for example, you can use the 01_B1 dash physics command and set the character's x velociy to the contents of register 01 like so:
```
01000000
B1000000
03000000
00000000
06000000
06000000
06000000
01FFFFFF
000080BF
00000000
```

## Operation replacement
Just like the float replacement above, you can use XXFFFFFF for operations. For fancy-ish math, so for operations 66_11 through 66_14 you can replace the operation with the contents of a register
```
66000000
12000000
18FFFFFF
15160017
```
So this will load what operation to do from register 18. So if register 18 = 0, then this will do
`register[17] = register[15] + register[16]`
But if register 18 is, say, 3, then it would do
`register[17] = register[15] / register[16]`

## list of operations/variables
this is all the binary operations. if you want any not listed here feel free to ask, no promises though
```rs
pub enum BinaryOp {
    /// addition
    Add = 0x00,
    /// subtraction
    Sub = 0x01,
    /// multiplication
    Mul = 0x02,
    /// division
    /// note: if the rhs is 0, the result will be 0
    Div = 0x03,
    /// takes the remainder of division
    Remainder = 0x04,
    /// Minimum, the smaller of the two numbers
    Min = 0x10,
    /// Maximum, the larger of the two numbers
    Max = 0x11,
    /// absolute difference. absolute value of (left hand side - right hand side)
    AbsDiff = 0x20,
    /// copies the sign from the right hand side. if right hand side is positive, then result is the left hand side. if the right hand side is negative, then the result is -left hand side.
    CopySign = 0x21,
    /// [bitwise and](https://en.wikipedia.org/wiki/Bitwise_operation#AND)
    /// note that doing this on floating point numbers won't be very useful as mag_patch resets any NaN/infinity floats to 0.0
    BitwiseAnd = 0xB0,
    /// [bitwise or](https://en.wikipedia.org/wiki/Bitwise_operation#OR)
    /// note that doing this on floating point numbers won't be very useful as mag_patch resets any NaN/infinity floats to 0.0
    BitwiseOr = 0xB1,
    /// [bitwise or](https://en.wikipedia.org/wiki/Bitwise_operation#XOR)
    /// note that doing this on floating point numbers won't be very useful as mag_patch resets any NaN/infinity floats to 0.0
    BitwiseXor = 0xB2,
    /// [sign-extending arithmetic shift left](https://en.wikipedia.org/wiki/Arithmetic_shift)
    /// note that doing this on floating point numbers won't be very useful as mag_patch resets any NaN/infinity floats to 0.0
    ShiftLeftSignExtend = 0xB3,
    /// [sign-extending arithmetic shift right](https://en.wikipedia.org/wiki/Arithmetic_shift)
    /// note that doing this on floating point numbers won't be very useful as mag_patch resets any NaN/infinity floats to 0.0
    ShiftRightSignExtend = 0xb4,
    /// [rotate left](https://en.wikipedia.org/wiki/Circular_shift)
    /// note that doing this on floating point numbers won't be very useful as mag_patch resets any NaN/infinity floats to 0.0
    RotateLeft = 0xb5,
    /// [rotate right](https://en.wikipedia.org/wiki/Circular_shift)
    /// note that doing this on floating point numbers won't be very useful as mag_patch resets any NaN/infinity floats to 0.0
    RotateRight = 0xb6,
    /// [zero-extending logical shift left](https://en.wikipedia.org/wiki/Logical_shift)
    /// note that doing this on floating point numbers won't be very useful as mag_patch resets any NaN/infinity floats to 0.0
    ShiftLeftZeroExtend = 0xB7,
    /// [zero-extending logical shift right](https://en.wikipedia.org/wiki/Logical_shift)
    /// note that doing this on floating point numbers won't be very useful as mag_patch resets any NaN/infinity floats to 0.0
    ShiftRightZeroExtend = 0xb8,
    /// equality operator. approximate for floating point numbers within 0.000001 due to [floating point imprecision](https://en.wikipedia.org/wiki/Floating-point_arithmetic#Accuracy_problems)
    /// true results are 1 and false are 0
    EqualityApproximate = 0xC0,
    /// left hand side < right hand side
    /// true results are 1 and false are 0
    LessThan = 0xC1,
    /// left hand side <= right hand side
    /// true results are 1 and false are 0
    LessThanEqual = 0xC2,
    /// left hand side > right hand side
    /// true results are 1 and false are 0
    GreaterThan = 0xC3,
    /// left hand side >= right hand side
    /// true results are 1 and false are 0
    GreaterThanEqual = 0xC4,
}
```

this is all the unary operations. if you want any not listed here feel free to ask, no promises though
```rs
pub enum UnaryOp {
    /// chop the fraction part off a number, so  4.0 or 4.1 or 4.5 or 4.9 will all become 4
    /// (does nothing for integers)
    Floor = 0x00,
    /// if there's a fraction part on a number, chop that off and get the next highest number so 5.0 or 4.1 or 4.5 or 4.9 will all become 5
    /// (does nothing for integers)
    Ceil = 0x01,
    /// rounds to the nearest integer so 4.0 and 4.1 will become 4.0. and 4.5 or 4.9 will become 5
    /// (does nothing for integers)
    Round = 0x02,
    /// keeps only the fraction part of a number. 0.1, 1.1, 1234.1 will all become 0.1
    /// (returns 0 for integers)
    Fract = 0x03,
    /// takes the square root of the number, so 64 will give 8.
    /// if the number is negative, then takes the absolute value and multiplies it by -1. so -64 will give -8
    SqrtWithNegative = 0x10,
    /// approximate sine of the number. approximation is used to prevent this from working differently on different cpus / compiler versions.
    /// the approximation is here https://github.com/anotak/mag_patch/blob/main/src/math.rs#L58
    Sin = 0x11,
    /// approximate cosine of the number. approximation is used to prevent this from working differently on different cpus / compiler versions.
    /// the approximation is here https://github.com/anotak/mag_patch/blob/main/src/math.rs#L58
    Cos = 0x12,
    /// the number times itself
    Square = 0x13,
    /// absolute value
    Abs = 0x20,
    /// the sign of the number.
    /// if it's negative, it gives -1.
    /// positive gives 1.
    /// 0 gives 0.
    Signum = 0x21,
    /// the number multiplied by -1
    Negate = 0x22,
    /// 1 if the value is positive
    /// 0 if 0 or negative
    IsPositive = 0x30,
    /// 1 if the value is positive
    /// 0 if 0 or negative
    IsNegative = 0x31,
    /// flips all the bits 0 to 1 and 1 to 0. see also https://en.wikipedia.org/wiki/Bitwise_operation#NOT
    /// note that doing this on floating point numbers won't be very useful as mag_patch resets any NaN/infinity floats to 0.0
    BitwiseNot = 0xB0,
    /// if nonzero then returns 0.0
    /// if 0 then returns 1.0
    LogicalNot = 0xC0,
}
```
all the game variables. if you want any not listed here feel free to ask, no promises though
```rs
 pub enum MatchState {
    /// The game timer, as in 0-99. -1 if the timer is infinite.
    Timer = 0x00,
    /// Counts up from the start of the current match state once per frame
    FrameTimerReadOnly = 0x01,
    MatchStateReadOnly = 0x02,
    Health = 0x10,
    RedHealth = 0x11,
    MaxHealth = 0x12,
    /// Currently executing anmchr index
    AnmChrIdReadonly = 0x13,
    /// 0.0 is the middle of the stage
    XPosition = 0x20,
    /// Floor is 0.0, upward is positive
    YPosition = 0x21,
    /// counts up from 0 every time you do a special in the air. is used to limit specials to 3 normally
    SpecialAirActionCounter = 0x30,
    /// counts up from 0 for every time you switch button strength in an air chain. is used to limit how much you can chain in normal jump mode
    NormalAirActionCounter = 0x31,
    /// combo counter for just this current character
    CharacterComboCounter = 0x32,
    /// the extra cooldown off assist this specific character has
    AssistCooldown = 0x33,
    /// super meter. 50000.0 is the max. 10000.0 is one bar
    Meter = 0x40,
    /// the combo counter for the whole team. read only because this is derived from the character specific combo counters.
    TeamComboCounterReadOnly = 0x41,
    /// Inputs as flags:
    /// 1 = fwd, 2 = bwd, 4 = up, 8 = down,
    /// 0x10 = l, 0x20 = m, 0x40 = h, 0x80 = s,
    /// 0x100 = a1, 0x200 = a2,
    /// 0x100000 = taunt
    InputsReadOnly = 0x42,
    /// controller forward / backward as a number, +1 is holding forward, -1 is holding backward, 0 is neutral
    InputsForwardBackwardReadOnly = 0x43,
    /// controller up / down as a number, +1 is holding up, -1 is holding down, 0 is neutral
    InputsUpDownReadOnly = 0x44,
    /// 1 if facing left, 0 if facing right
    FacingReadOnly = 0xB0,
    /// current position on the team. 0 = point character. 1 = assist 1. 2 = assist 2.
    CharOrderReadOnly = 0xB1,
    /// current assist type. 0 = alpha, 1 = beta, 2 = gamma
    AssistType = 0xB2,
    /// the condition register used by commands like 0_46 (rng) or 1_92 (check dhc)
    ConditionRegister = 0xC0,
    /// the flags that are affected by the 1_2f - 1_34 commands
    AirGroundStateFlags = 0xF0,
    /// the flags that are affected by the 1_35 - 1_3a commands
    LeftRightFlags = 0xF1,
    /// the flags that are affected by the 1_3b - 1_40 commands
    InvincibilityFlags = 0xF2,
    /// the flags that are affected by the 1_41 - 1_46 commands
    SpecialFlags = 0xF3,
    /// the flags that are affected by the 1_4D-1_52 commands
    PartnerFlags = 0xF5,
    /// note: this is set on the opponent. this the flying screen install flag, which is normally set by launching the opponent. if it's 1 then a jump S will cause flying screen and a hard kd
    FlyingScreenInstallFlag = 0x1000,
}
```