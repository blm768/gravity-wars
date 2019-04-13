# Gravity Wars

This project is an attempt at a modern re-implementation of the classic game [Gravity Wars](http://www.physics.rutgers.edu/~doroshen/downloads/gwars.html) using WebAssembly and WebGL.
It's not fully playable yet, but the core logic is working well enough that it's at least recognizable as a game.

## Building and running

Run either `./build.sh` (UNIX-like platforms) or `.\build.ps1` (Windows) to compile the code.

To start the game, open `index.html` in a Web browser.

## Gameplay

Players take turns firing missiles at each other.
Enter the angle (in degrees) and power (from 0 to 10) for your shot, and press the Fire button.

Your missile is affected by the planets' gravity, sometimes in unexpected ways.
It's possible for missiles to enter a (rather unstable) orbit and keep flying for some time without hitting anything.
To prevent significant delays between turns, missiles will self-destruct after a few seconds.