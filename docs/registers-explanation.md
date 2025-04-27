i wanna start by saying that custom commands are never supposed to crash the game, even if you made a "mistake" with using them. so please let me know if they do.

so for 66_10 - 66_18, i gotta explain what a register is. a register is like, a custom variable basically, it's a place where you can store whatever you want. there's 128 registers for storing integers (whole numbers) and 128 registers for storing floating point numbers (numbers with decimals)

these are numbered
00-7f: integer registers
80-ff: floating point registers.
so basically if it starts with an 8, 9, a,b,c,d,e,f it's a floating point register.

registers all default to storing 0 at the start of a game.

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
e0 is the distination

so let's say EE had both 1.7 in them, then register E0 gets set to 1.0 (true)
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

AH HELL I JUST NOTICED A BUG where the register type on unaries is always the destination type so you cant do a float input for an integer output or vice versa. i'll fix that at some point

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


## 66_16 is storing a register back into a game variable
```
66000000
16000000
000000FF
30000000
```
same format, 00 00 00 there doesn't mean anything, FF is the register, 30000000 is the variable (the special air action counter).

so if register FF has 1.7 in it, then that gets converted to an integer (since the counter is an integer), then it gets set to 1. this means you can do 2 more specials in the air.

## 66_17 is loading an *opponent's* variable into a register
```
66000000
17000000
000000A4
21000000
```
same format again
so now register A4 has the opponent's Y position (21000000)
this always uses their point character.

## 66_18 is storing a register into an *opponent's* variable
```
66000000
18000000
00000033
10000000
```
same format again
so now the opponent's yellow health (10000000) has been set to whatever number is in register 33.
this always uses their point character.

## list of operations/variables
this is all the binary operations. if you want any not listed here feel free to ask, no promises though
```
pub enum BinaryOp {
    Add = 0x00,
    Sub = 0x01,
    Mul = 0x02,
    Div = 0x03,
    Mod = 0x04,
    Min = 0x10,
    Max = 0x11,
    AbsDiff = 0x20,
    CopySign = 0x21,
    BitwiseAnd = 0xB0,
    BitwiseOr = 0xB1,
    BitwiseXor = 0xB2,
    ShiftLeft = 0xB3,
    ShiftRight = 0xb4,
    RotateLeft = 0xb5,
    RotateRight = 0xb6,
    EqualityApproximate = 0xC0,
    LessThan = 0xC1,
    LessThanEqual = 0xC2,
    GreaterThan = 0xC3,
    GreaterThanEqual = 0xC4,
}
```

this is all the unary operations. if you want any not listed here feel free to ask, no promises though
```
pub enum UnaryOp {
    Floor = 0x00,
    Ceil = 0x01,
    Round = 0x02,
    Fract = 0x03,
    SqrtWithNegative = 0x10,
    Sin = 0x11,
    Cos = 0x12,
    Abs = 0x20,
    Signum = 0x21,
    IsPositive = 0x30,
    BitwiseNot = 0xB0,
    LogicalNot = 0xC0,
}
```
all the game variables. if you want any not listed here feel free to ask, no promises though
```
pub enum MatchState {
    Timer = 0x00,
    FrameTimerReadOnly = 0x01,
    MatchStateReadOnly = 0x02,
    Health = 0x10,
    XPosition = 0x20,
    YPosition = 0x21,
    SpecialAirActionCounter = 0x30,
    NormalAirActionCounter = 0x31,
    CharacterComboCounter = 0x32,
    Meter = 0x40,
    TeamComboCounterReadOnly = 0x41,
    ConditionRegister = 0xC0,
}
