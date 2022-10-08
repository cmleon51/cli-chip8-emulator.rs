# cli-chip8-emulator.rs
chip8 emulator built in rust that runs on the terminal. The emulator doesn't produce any sound.

# Keyboard
| Original      | Emulator      |  
| ------------- | ------------- | 
| 1             | 1             | 
| 2             | 2             |
| 3             | 3             | 
| C             | 4             |
| 4             | Q             | 
| 5             | W             | 
| 6             | E             |
| D             | R             | 
| 7             | A             | 
| 8             | S             |
| 9             | D             |
| E             | F             | 
| A             | Z             |
| 0             | X             |
| B             | C             | 
| F             | V             |
| /             | ↓             | 
| /             | ↑             |
| /             | ←             | 
| /             | →             | 
| /             | l             | 

The `l`,`↓` and `↑` are new keys only for the emulator:
 - :The `l` switches between having the emulator fill the whole terminal or being confined to the resolution of the original chip8 screen
 - :The `↓` decrements the number of opcodes per cpu cycle
 - :The `↑` increments the number of opcodes per cpu cycle
 - :The `↓` decrements the number of opcodes per cpu cycle
 - :The `←` decrements the number of hz per cpu cycle
 - :The `→` increments the number of hz per cpu cycle

# To play
run `./chip_8_emulator.exe "file/to/rom"` on the terminal
