# Smart-road

## Description

This program have the objective to solves the trafic problems by simulating with visualization of the trafic control strategies.
```txt

               |   |   |   |   |   |   |
               |   |   |   |   |   |   |
               |   |   |   |   |   |   |
               |r  | s | l |   |   |   |
_______________| ← | ↓ | → |   |   |   |________________
                           |            ↑ r
_______________            |            ________________
                           |            ← s
_______________            |            ________________
                           |            ↓ l
___________________________|____________________________
           l ↑             |
_______________            |            ________________
           s →             |
_______________            |            ________________
           r ↓             |
_______________            |            ________________
               |   |   |   | ← | ↑ | → |
               |   |   |   | l | s | r |
               |   |   |   |   |   |   |
               |   |   |   |   |   |   |
               |   |   |   |   |   |   |
               |   |   |   |   |   |   |
```

## Installation

Clone the repository and you can use directly if all rust setup is done:
`https://zone01normandie.org/git/lchouvil/smart-road`

Install sdl2 and sdl2:image with the following:
```cmd
sudo apt-get update
sudo apt-get install libsdl2-dev libsdl2-image-dev libsdl2-ttf-dev
```

## Usage

Launch this program with the following command on app source repository:
```rs
cargo run
```

Input and they effects
- **Left arrow**    : Spawn Vehicle from the **West**
- **Right arrow**   : Spawn Vehicle from the **East**
- **Up arrow**      : Spawn Vehicle from the **North**
- **Down arrow**    : Spawn Vehicle from the **South**
- **R**             : Spawn Vehicle from the **Random Direction**
- **A**             : Spawn Vehicle from the **Random Direction** one minute duration

- **Escape**    : Show stats and Close the Simulation 
- **Space**     : Accelerate the simulation
- **T**         : Slowdown the simulation
- **P**         : Pause the simulation
- **O**         : Show vehicle hitbox
- **I**         : Show maps sector


## Authors

- aferrand [[Gitea]](https://zone01normandie.org/git/aferrand)
- lchouvil [[Gitea]](https://zone01normandie.org/git/lchouvil) [[Git]](https://github.com/lchouville)