# Main Project Structure

## Input

User will provide a json file through the command line argument, and the program will read it and
get the following information:

-   Who's turn is it (Either X or O)
-   The board

Here is an example json:

```json
{
    "turn": "X",
    "board": [
        ["X", null, null],
        ["O", "X", null],
        ["O", null, null]
    ]
}
```

where the board will be an array of 3 arrays, and each array contains 3 values. The value can either
be "X", "O" or null to indicate what is on that square.

## Analysis and Calculation

First, deserialize the JSON input.

After that, I think it is just pure brute force.

Here is my thought:

First of all, the program needs to check whether it is a valid position or not. If the position
already has one side connecting 3 already, then the program will output a result of the player that
won. If there are 2 sides that wins the game, the program throw an invalid position error. If all
the board is filled up and there is no possible squares to go to, then it will simply output a draw.

At first, the program will try all the move possible, and if any of them results in a win, well
then, count that as a win! That will go into the analysis result as one of the possible moves. If
none of the moves results in an immediate win, then the program continues by picking a random move,
and move to the next turn and try all the possible moves again. Unlike last time, if any of the
moves result in a win, that means the previous move is a blunder, and it saves to the analysis
result as a lose. This keeps repeating until all the squares are filled up which then it is a draw.

This is not a very efficient way of doing this, so I need to think of other ways that might work
better, but I will go with this for now.

Seems like it is working perfectly. Basically I did a recursion of analysis and with some magic I made it work. I am so proud of myself lmao. I am very happy to see the result though.

## Output

The program should output a json file to indicate all the possible squares the player can go and
whether it is winning, draw, or losing (with 100% accurate play, of course).

User can also customize what the output json filename will be by putting `-O <FILE_NAME>` in the
arguments. Default output filename will be `analysis.json`.

Square can be represented similar to how chess annotation works. Assign each row with 1, 2, 3, and
each column with A, B, and C. When you write the square, it should be letter (column) first and then
number (row). Example: C3 will be the bottom right corner square, and C1 will be the bottom left
corner square.

It will also output the evaluation, which is with perfect play what will the result of the game be.

Example json:

```json
{
    "moves": [
        {
            "square": "C3",
            "analysis": "win"
        }
        // and all the possible move and its analysis list after...
    ],
    "eval": "win"
}
```

it should also be important that the moves are sorted from win to draw to lose.
