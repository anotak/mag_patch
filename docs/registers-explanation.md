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


to get an opponent's register you use:
```
66000000
17000000
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
18000000
00800033
10000000
```
same format again
so now the opponent's yellow health (10000000) has been set to whatever number is in register 33.
this always uses their point character.

## 66_17 is deleted for now (was opponent load but see 66_15 now)


## 66_18 is deleted for now (was opponent store but see 66_16 now)


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
```rs
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
```rs
pub enum MatchState {
    Timer = 0x00,
    FrameTimerReadOnly = 0x01,
    MatchStateReadOnly = 0x02,
    Health = 0x10,
    RedHealth = 0x11,
    MaxHealth = 0x12,
    XPosition = 0x20,
    YPosition = 0x21,
    SpecialAirActionCounter = 0x30,
    NormalAirActionCounter = 0x31,
    CharacterComboCounter = 0x32,
    Meter = 0x40,
    TeamComboCounterReadOnly = 0x41,
    FacingReadOnly = 0xB0,
    CharOrderReadOnly = 0xB1,
    ConditionRegister = 0xC0,
    FlyingScreenInstallFlag = 0x1000,
}
```