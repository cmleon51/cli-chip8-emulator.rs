use rand;

const MEM_SIZE: usize = 4096;
const STACK_SIZE: usize = 16;
const VX_REGISTERS: usize = 16;
const START_LOCATION: usize = 0x200;
const KEYBOARD_SIZE: usize = 16;
const FONT_SPRITES: [[u8; 5]; 16] = [
    [0xF0, 0x90, 0x90, 0x90, 0xF0],
    [0x20, 0x60, 0x20, 0x20, 0x70],
    [0xF0, 0x10, 0xF0, 0x80, 0xF0],
    [0xF0, 0x10, 0xF0, 0x10, 0xF0],
    [0x90, 0x90, 0xF0, 0x10, 0x10],
    [0xF0, 0x80, 0xF0, 0x10, 0xF0],
    [0xF0, 0x80, 0xF0, 0x90, 0xF0],
    [0xF0, 0x10, 0x20, 0x40, 0x40],
    [0xF0, 0x90, 0xF0, 0x90, 0xF0],
    [0xF0, 0x90, 0xF0, 0x10, 0xF0],
    [0xF0, 0x90, 0xF0, 0x90, 0x90],
    [0xE0, 0x90, 0xE0, 0x90, 0xE0],
    [0xF0, 0x80, 0x80, 0x80, 0xF0],
    [0xE0, 0x90, 0x90, 0x90, 0xE0],
    [0xF0, 0x80, 0xF0, 0x80, 0xF0],
    [0xF0, 0x80, 0xF0, 0x80, 0x80],
];
pub const SCREEN_HEIGHT: usize = 32;
pub const SCREEN_WIDTH: usize = 64;

pub struct Chip8 {
    memory: [u8; MEM_SIZE],
    Vx: [u8; VX_REGISTERS],
    I: u16,
    ST: u8,
    DT: u8,
    PC: usize,
    SP: usize,
    stack: [usize; STACK_SIZE],
    keyboard: [bool; KEYBOARD_SIZE], //true = pressed, false = not pressed
    monitor: [[u8; SCREEN_WIDTH]; SCREEN_HEIGHT],
}

impl Chip8 {
    //creates a new chip8 instance with everything at 0
    pub fn new() -> Self {
        Self {
            memory: [0u8; MEM_SIZE],
            Vx: [0u8; VX_REGISTERS],
            I: 0,
            PC: START_LOCATION,
            SP: 0,
            stack: [0; STACK_SIZE],
            DT: 0,
            ST: 0,
            keyboard: [false; KEYBOARD_SIZE],
            monitor: [[0u8; SCREEN_WIDTH]; SCREEN_HEIGHT],
        }
    }

    //inserts inside of the chip8 memory (starting from address 0x200) the program to run
    pub fn start(program: &[u8]) -> Self {
        let mut chip_8 = Self::new();
        let mut counter_tmp = 0;

        for i in FONT_SPRITES {
            for j in i {
                chip_8.memory[counter_tmp] = j;
                counter_tmp += 1;
            }
        } //load the fonts in memory

        for i in program {
            chip_8.memory[chip_8.PC] = *i;
            chip_8.PC += 1;
        } //loads the program in memory

        chip_8.PC = START_LOCATION; //resets the PC
        chip_8
    }

    ///removes 1 from the "timer register" and "sound timer"
    pub fn update(&mut self) {
        self.DT -= if self.DT > 0 { 1 } else {0};
        self.ST -= if self.ST > 0 { 1 } else {0};
    }

    ///runs trough one opcode and returns the run opcode
    pub fn execute_next_opcode(&mut self) -> u16 {
        let opcode = u16::from_be_bytes([self.memory[self.PC], self.memory[self.PC + 1]]);

        //we divide the opcode into 4 4 bits values
        let x = ((opcode >> 8) & 0xF) as usize; //lower 4 bits of first byte
        let y = ((opcode >> 4) & 0xF) as usize; //higher 4 bits of second byte
        let n = (opcode & 0xF) as u8; //lower 4 bits of second byte
        let nnn = (opcode & 0xFFF) as u16; //lower 12 bits of opcode
        let kk = (opcode & 0xFF) as u8; //lower bits 8 of opcode
        
        match &opcode & 0xF000 {
            0x0000 => match opcode {
                0x00E0 => self._00E0(),
                0x00EE => self._00EE(),
                _ => self.no_opcode_found(),
            },
            0x1000 => self._1nnn(nnn as usize),
            0x2000 => self._2nnn(nnn as usize),
            0x3000 => self._3xkk(x, kk),
            0x4000 => self._4xkk(x, kk),
            0x5000 => self._5xy0(x, y),
            0x6000 => self._6xkk(x, kk),
            0x7000 => self._7xkk(x, kk),
            0x8000 => match opcode & 0xF {
                0x0000 => self._8xy0(x, y),
                0x0001 => self._8xy1(x, y),
                0x0002 => self._8xy2(x, y),
                0x0003 => self._8xy3(x, y),
                0x0004 => self._8xy4(x, y),
                0x0005 => self._8xy5(x, y),
                0x0006 => self._8xy6(x, y),
                0x0007 => self._8xy7(x, y),
                0x000E => self._8xyE(x, y),
                _ => self.no_opcode_found(),
            },
            0x9000 => self._9xy0(x, y),
            0xA000 => self._Annn(nnn),
            0xB000 => self._Bnnn(nnn),
            0xC000 => self._Cxkk(x, kk),
            0xD000 => self._Dxyn(x, y, n as usize),
            0xE000 => match opcode & 0xFF {
                0x009E => self._Ex9E(x),
                0x00A1 => self._ExA1(x),
                _ => self.no_opcode_found(),
            },
            0xF000 => match opcode & 0xFF {
                0x0007 => self._Fx07(x),
                0x000A => self._Fx0A(x),
                0x0015 => self._Fx15(x),
                0x0018 => self._Fx18(x),
                0x001E => self._Fx1E(x),
                0x0029 => self._Fx29(x),
                0x0033 => self._Fx33(x),
                0x0055 => self._Fx55(x),
                0x0065 => self._Fx65(x),
                _ => self.no_opcode_found(),
            },

            _ => self.no_opcode_found(),
        };
        
        opcode
    }

    //sets to true the desired key and returns ok or error
    pub fn set_key<'a>(&'a mut self,x: usize) -> Result<(),&'a str>{
        *(self.keyboard.get_mut(x).unwrap()) = true;

        Ok(())
    }
    
    //sets all of the keys to the ones of the array passed in the arguments
    pub fn set_keys(&mut self,keys: &[bool]){
        for i in 0..16{
            self.keyboard[i] = keys[i];
        }
    }

    pub fn get_pixel(&self,x: usize,y: usize) -> u8 {
        self.monitor[y % SCREEN_HEIGHT][x % SCREEN_WIDTH]
    } 
}

//opcodes for chip_8 (no chip-48 instructions)
impl Chip8 {
    fn no_opcode_found(&mut self) {
        self.PC += 2;
    }

    fn _00E0(&mut self) {
        self.monitor = [[0u8; SCREEN_WIDTH]; SCREEN_HEIGHT];
        self.PC += 2;
    }

    fn _00EE(&mut self) {
        self.PC = self.stack[self.SP];
        self.SP -= 1;
    }

    fn _1nnn(&mut self, addr: usize) {
        self.PC = addr;
    }

    fn _2nnn(&mut self, addr: usize) {
        self.PC += 2;
        self.SP += 1;
        self.stack[self.SP] = self.PC;
        self.PC = addr;
    }

    fn _3xkk(&mut self, Vx_reg: usize, kk: u8) {
        if self.Vx[Vx_reg] == kk {
            self.PC += 2;
        }
        self.PC += 2;
    }

    fn _4xkk(&mut self, Vx_reg: usize, kk: u8) {
        if self.Vx[Vx_reg] != kk {
            self.PC += 2;
        }
        self.PC += 2;
    }

    fn _5xy0(&mut self, Vx_reg: usize, Vy_reg: usize) {
        if self.Vx[Vx_reg] == self.Vx[Vy_reg] {
            self.PC += 2;
        }
        self.PC += 2;
    }

    fn _6xkk(&mut self, Vx_reg: usize, kk: u8) {
        self.Vx[Vx_reg] = kk;
        self.PC += 2;
    }

    fn _7xkk(&mut self, Vx_reg: usize, kk: u8) {
        self.Vx[Vx_reg] = self.Vx[Vx_reg].wrapping_add(kk);
        self.PC += 2;
    }

    fn _8xy0(&mut self, Vx_reg: usize, Vy_reg: usize) {
        self.Vx[Vx_reg] = self.Vx[Vy_reg];
        self.PC += 2;
    }

    fn _8xy1(&mut self, Vx_reg: usize, Vy_reg: usize) {
        self.Vx[Vx_reg] |= self.Vx[Vy_reg];
        self.PC += 2;
    }

    fn _8xy2(&mut self, Vx_reg: usize, Vy_reg: usize) {
        self.Vx[Vx_reg] &= self.Vx[Vy_reg];
        self.PC += 2;
    }

    fn _8xy3(&mut self, Vx_reg: usize, Vy_reg: usize) {
        self.Vx[Vx_reg] ^= self.Vx[Vy_reg];
        self.PC += 2;
    }

    fn _8xy4(&mut self, Vx_reg: usize, Vy_reg: usize) {
        let tmp: u16 = self.Vx[Vx_reg] as u16 + self.Vx[Vy_reg] as u16;
        self.Vx[15] = if tmp > 255 { 1 } else { 0 };
        self.Vx[Vx_reg] = (tmp & 0xFF) as u8;
        self.PC += 2;
    }

    fn _8xy5(&mut self, Vx_reg: usize, Vy_reg: usize) {
        self.Vx[15] = if self.Vx[Vx_reg] > self.Vx[Vy_reg] {
            1
        } else {
            0
        };
        self.Vx[Vx_reg] -= self.Vx[Vy_reg];
        self.PC += 2;
    }

    fn _8xy6(&mut self, Vx_reg: usize, Vy_reg: usize) {
        self.Vx[15] = if (self.Vx[Vx_reg] & 0x1) == 1 { 1 } else { 0 };
        self.Vx[Vx_reg] /= 2;
        self.PC += 2;
    }

    fn _8xy7(&mut self, Vx_reg: usize, Vy_reg: usize) {
        self.Vx[15] = if self.Vx[Vy_reg] > self.Vx[Vx_reg] {
            1
        } else {
            0
        };
        self.Vx[Vx_reg] = self.Vx[Vy_reg] - self.Vx[Vx_reg];
        self.PC += 2;
    }

    fn _8xyE(&mut self, Vx_reg: usize, Vy_reg: usize) {
        let shifted_register = self.Vx[Vx_reg] << 1;
        self.Vx[15] = if (shifted_register & 0x1) == 1 { 1 } else { 0 };
        self.Vx[Vx_reg] *= 2;
        self.PC += 2;
    }

    fn _9xy0(&mut self, Vx_reg: usize, Vy_reg: usize) {
        if self.Vx[Vx_reg] != self.Vx[Vy_reg] {
            self.PC += 2
        }
        self.PC += 2;
    }

    fn _Annn(&mut self, nnn: u16) {
        self.I = nnn;
        self.PC += 2;
    }

    fn _Bnnn(&mut self, nnn: u16) {
        self.PC = (nnn as usize) + (self.Vx[0] as usize);
    }

    fn _Cxkk(&mut self, Vx_reg: usize, kk: u8) {
        self.Vx[Vx_reg] = kk & rand::random::<u8>();
        self.PC += 2;
    }

    fn _Dxyn(&mut self, Vx_reg: usize, Vy_reg: usize, bytes_to_read: usize) {
        let row = self.Vx[Vy_reg];
        let col = self.Vx[Vx_reg];
        self.Vx[0xF] = 0;

        for i in 0..bytes_to_read {
            let memory_pixel = self.memory[(self.I as usize) + i];

            for j in 0..8 {
                let bit = (memory_pixel >> j) & 0x1;
                let pixel_screen = self.monitor[(row as usize + i) % SCREEN_HEIGHT]
                    [(col as usize + 7 - j) % SCREEN_WIDTH];

                if bit == 1 && pixel_screen == 1 {
                    self.Vx[0xF] = 1;
                }

                self.monitor[(row as usize + i) % SCREEN_HEIGHT]
                    [(col as usize + 7 - j) % SCREEN_WIDTH] ^= bit;
            }
        }

        self.PC += 2;
    }

    fn _Ex9E(&mut self, Vx_reg: usize) {
        if self.keyboard[self.Vx[Vx_reg] as usize] == true {
            self.keyboard[self.Vx[Vx_reg] as usize] = false;
            self.PC += 2;
        }
        self.PC += 2;
    }

    fn _ExA1(&mut self, Vx_reg: usize) {
        if self.keyboard[self.Vx[Vx_reg] as usize] == false {
            self.PC += 2;
        }
        self.PC += 2;
        self.keyboard[self.Vx[Vx_reg] as usize] = false;
/* 
        for i in &mut self.keyboard{
            *i = false;
        } */
    }

    fn _Fx07(&mut self, Vx_reg: usize) {
        self.Vx[Vx_reg] = self.DT;
        self.PC += 2;
    }

    fn _Fx0A(&mut self, Vx_reg: usize) {
        if let Some(k) = self.keyboard.iter().position(|x| *x == true){
            
            self.keyboard[k] = false;
            self.Vx[Vx_reg] = k as u8;
            self.PC += 2;
        }
    }

    fn _Fx15(&mut self, Vx_reg: usize) {
        self.DT = self.Vx[Vx_reg];
        self.PC += 2;
    }

    fn _Fx18(&mut self, Vx_reg: usize) {
        self.ST = self.Vx[Vx_reg];
        self.PC += 2;
    }

    fn _Fx1E(&mut self, Vx_reg: usize) {
        self.I = self.I.wrapping_add(self.Vx[Vx_reg] as u16);
        self.PC += 2;
    }

    fn _Fx29(&mut self, Vx_reg: usize) {
        self.I = (5 * self.Vx[Vx_reg]) as u16;
        self.PC += 2;
    }

    fn _Fx33(&mut self, Vx_reg: usize) {
        self.memory[self.I as usize] = (((self.Vx[Vx_reg] as i32) % 1000) / 100) as u8;
        self.memory[(self.I + 1) as usize] = (self.Vx[Vx_reg] % 100) / 10;
        self.memory[(self.I + 2) as usize] = (self.Vx[Vx_reg] % 10);
        self.PC += 2;
    }

    fn _Fx55(&mut self, Vx_reg: usize) {
        let mut tmp = self.I as usize;

        for i in 0..=Vx_reg {
            self.memory[tmp] = self.Vx[i];
            tmp += 1;
        }
        self.PC += 2;
    }

    fn _Fx65(&mut self, Vx_reg: usize) {
        let mut tmp = self.I as usize;

        for i in 0..=Vx_reg {
            self.Vx[i] = self.memory[tmp];
            tmp += 1;
        }
        self.PC += 2;
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn chip_8_testing_emulation() {
        use crate::chip_8::Chip8;
        use std::fs::File;
        use std::io::*;

        let mut file = File::options()
            .read(true)
            .create(false)
            .open("./TETRIS")
            .unwrap();
        //let mut file_content: [u8;2] = [0u8;2];
        let mut file_contents: Vec<u8> = Vec::new();
        file.read_to_end(&mut file_contents).unwrap();

        let mut chip_8 = Chip8::start(&file_contents[..]);

        while true {
            chip_8.execute_next_opcode();

            println!("");
        }
    }
}
