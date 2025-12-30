`mag_patch` adds new anmchr commands.

Commands 66_10 through 66_1b are in the [Registers explanation document](registers-explanation.md). That document also covers replacing floating point numbers with the content of registers.

## 66_00 is relative teleport in the X axis

The last 4 bytes are a floating point position. This example puts you 150 units in front of the opponent.

```
66000000
00000000
00001643
```

To go behind the opponent, make it negative, so here's 150 units behind hte opponent.
```
66000000
00000000
000016C3
```

If you want to have a teleport that appears in the air above the opponent like many base game teleports, then combine this with 66_01.

## 66_01 is relative teleport in the Y axis

The last 4 bytes are a floating point position. This example puts you 150 units above the opponent.

```
66000000
01000000
00001643
```

To go below the opponent, make it negative, so here's 150 units below the opponent.
```
66000000
01000000
000016C3
```

## 66_50 is attraction/repulsion on the X axis.

This command causes the opponent to be pulled or pushed away from your character. The first floating point parameter controls starting velocity, and the second one controls deceleration. Negative decelerations are not recommended.

Negative starting speed is attraction, positive starting speed is repulsion.

So here's an example of attraction with starting velocity -19 and deceleration of -1.
```
66000000
50000000
000098C1
0000803F
```

And here's an example of repulsion with starting velocity 19 and deceleration of -1.
```
66000000
50000000
00009841
0000803F
```
