use std::fs;
use std::io;
use rand::Rng;

use pancurses::{initscr, endwin, Window, noecho, start_color, COLOR_CYAN, COLOR_BLACK, init_pair, COLOR_PAIR, Input};

const APP_START: u16 = 0x200;
const SCREEN_WIDTH: usize = 64;
const SCREEN_HEIGHT: usize = 32;
const NO_CURSES: bool = false;
//const NO_CURSES: bool = true;

fn main() {
    let mut chip8 = Chip8::new();
    chip8.load("pong.ch8").unwrap();
    //chip8.load("space.ch8").unwrap();
    //chip8.load("zero.ch8").unwrap();
    if NO_CURSES {
        endwin();
    }
    chip8.start();
}


#[derive(Debug)]
struct Chip8 {
    registers: Vec<u8>,
    memory: Vec<u8>,
    index: u16,
    size: usize,
    pc: u16,
    stack: Vec<u16>,
    stack_pointer: u8,
    screen: Vec<u8>,
    window: Window,
    window_memory: Window,
    display: Window,
    cycles: usize,
    timer: u8,
    keypad: Vec<u8>,
    sound_timer: u8,
}

#[allow(dead_code)]
impl Chip8 {
    const SPRITE_LOCATION: u8 = 0x50;

    pub fn new() -> Chip8 {
        let window = initscr();
        window.nodelay(false);
        noecho();

        let display = window.subwin(SCREEN_HEIGHT as i32, SCREEN_WIDTH as i32, 1, 100).unwrap();
        let window_memory = window.subwin(50, 80, 0, 0).unwrap();

        start_color();
        init_pair(1, COLOR_CYAN, COLOR_BLACK);

        Chip8 {
            registers: (0..=15).map(|_| 0).collect(),
            memory: (0..=4096).map(|_| 0).collect(),
            index: 0,
            pc: 0,
            stack: Vec::with_capacity(16),
            stack_pointer: 0,
            screen: (0..=(SCREEN_WIDTH * SCREEN_HEIGHT)).map(|_| 0).collect(),
            window: window,
            display: display,
            window_memory: window_memory,
            size: 0,
            cycles: 0,
            timer: 0,
            keypad: (0..=15).map(|_| 0).collect(),
            sound_timer: 0,
        }
    }

    pub fn draw(&mut self) {
        self.print_debug(0x200, Some(APP_START as usize + self.size));
        self.display.erase();
        for x in self.screen.iter() {
            if *x > 0 {
                self.display.addstr("#");
            } else {
                self.display.addstr("_");
            }
        }
        self.display.refresh();
    }

    pub fn start(&mut self) {
        self.load_fonts();
        loop {
            if !NO_CURSES {
                self.draw();
            }

            match self.window.getch() {
                Some(Input::KeyEnter) => break,
                Some(Input::Character('1')) => { self.keypad[0] = 1; }
                Some(Input::Character('2')) => { self.keypad[1] = 1; }
                Some(Input::Character('3')) => { self.keypad[2] = 1; }
                Some(Input::Character('C')) => { self.keypad[4] = 1; }
                Some(Input::Character('4')) => { self.keypad[5] = 1; }
                Some(Input::Character('5')) => { self.keypad[6] = 1; }
                Some(Input::Character('6')) => { self.keypad[7] = 1; }
                Some(Input::Character('7')) => { self.keypad[8] = 1; }
                Some(Input::Character('8')) => { self.keypad[9] = 1; }
                Some(Input::Character('9')) => { self.keypad[0xA] = 1; }
                Some(Input::Character('E')) => { self.keypad[0xB] = 1; }
                Some(Input::Character('A')) => { self.keypad[0xC] = 1; }
                Some(Input::Character('0')) => { self.keypad[0xD] = 1; }
                Some(Input::Character('B')) => { self.keypad[0xE] = 1; }
                Some(Input::Character('F')) => { self.keypad[0xF] = 1; }
                _ => ()
            }

            self.cycle();
        }
    }

    fn load_fonts(&mut self) {
        let fonts = vec![
            0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
            0x20, 0x60, 0x20, 0x20, 0x70, // 1
            0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
            0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
            0x90, 0x90, 0xF0, 0x10, 0x10, // 4
            0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
            0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
            0xF0, 0x10, 0x20, 0x40, 0x40, // 7
            0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
            0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
            0xF0, 0x90, 0xF0, 0x90, 0x90, // A
            0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
            0xF0, 0x80, 0x80, 0x80, 0xF0, // C
            0xE0, 0x90, 0x90, 0x90, 0xE0, // D
            0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
            0xF0, 0x80, 0xF0, 0x80, 0x80  // F
        ];

        for (i, x) in fonts.iter().enumerate() {
            self.memory[Self::SPRITE_LOCATION as usize + i] = *x;
        }
    }

    pub fn load(&mut self, path: &str) -> Result<(), io::Error>{
        let content = fs::read(path)?;

        let start_address: u16 = APP_START;
        for (i, x) in content.iter().enumerate() {
            self.memory[start_address as usize + i] = *x;
        }

        self.pc = APP_START;
        self.size = content.len();

        Ok(())
    }

    pub fn print_debug(&self, start_address: usize, end_address: Option<usize>) {
        let mut c = 0;
        let end_address = end_address.unwrap_or(self.memory.len());

        self.window_memory.erase();

        for i in start_address..end_address {
            if c % 6 == 0 {
                self.window_memory.addstr("\n");
            }

            let x = self.memory[i];

            self.window_memory.addstr(format!("{:04X}: ", i));
            if self.pc == i as u16 || (self.pc + 1) == i as u16 {
                self.window_memory.attron(COLOR_PAIR(1));
            } 

            self.window_memory.addstr(format!("{:02X} ", x));
            self.window_memory.attroff(COLOR_PAIR(1));

            c += 1;
        }

        self.window_memory.addstr("\n");
        self.window_memory.addstr("\n");

        for i in 0..=0xF {
            self.window_memory.addstr(format!("V{:X}: {:X} | ", i, self.registers[i]));
        }
        self.window_memory.addstr("\n");
        for i in 0..=0xF {
            self.window_memory.addstr(format!("{:?}: {:?} | ", i, self.keypad[i]));
        }

        self.window_memory.addstr("\n");

        self.window_memory.addstr(format!("pc {:X}\n", self.pc));
        self.window_memory.addstr(format!("index {:X}\n", self.index));
        self.window_memory.addstr(format!("cycles {:?}\n", self.cycles));
        self.window_memory.addstr(format!("timer {:?}\n", self.timer));

        let opcode1 = self.memory[self.pc as usize];
        let opcode2 = self.memory[self.pc as usize + 1];
        self.window_memory.addstr(format!("opcode {:02X}{:02X} \n", opcode1, opcode2));



        self.window_memory.refresh();
    }

    fn cycle(&mut self) {
        let opcode_1 = self.memory[self.pc as usize] >> 4;
        let opcode_2 = self.memory[self.pc as usize] & 0x0F;

        self.pc += 1;
        let opcode_3 = self.memory[self.pc as usize] >> 4;
        let opcode_4 = self.memory[self.pc as usize] & 0x0F;
        
        self.pc += 1;

        let opcode = (opcode_1, opcode_2, opcode_3, opcode_4);
        self.cycles += 1;

        if self.timer > 0 {
            self.timer -= 1;
        }

        if NO_CURSES {
            let opcode1 = self.memory[self.pc as usize - 2];
            let opcode2 = self.memory[self.pc as usize - 1];
            println!("{:?} opcode {:02X}{:02X}", self.cycles, opcode1, opcode2);
        }

        self.execute_opcode(opcode);
    }

    fn execute_opcode(&mut self, opcode: (u8, u8, u8, u8)) {
        match opcode {
            (0, 0, 0xE, 0xE) => {
                // Return from a subroutine
                self.pc = self.stack.pop().unwrap();
            },
            (1, n1, n2, n3) => {
                // Jump to location nnn
                let addr: u16 = (n1 as u16) << 8 | (n2 as u16) << 4 | (n3 as u16);
                self.pc = addr;
            },
            (2, n1, n2, n3) => {
                // Call subroutine at nnn.
                self.stack.push(self.pc);
                let addr: u16 = (n1 as u16) << 8 | (n2 as u16) << 4 | (n3 as u16);
                self.pc = addr;
            },
            (3, x, k1, k2) => {
                //  Skip next instruction if Vx = kk.
                let vx = self.registers[x as usize];
                let kk: u8 = (k1 << 4) | k2; 

                if vx == kk {
                    self.pc += 2;
                }
            },
            (4, x, k1, k2) => {
                // Skip next instruction if Vx != kk.
                let vx = self.registers[x as usize];
                let kk: u8 = (k1 << 4) | k2; 

                if vx != kk {
                    self.pc += 2;
                }
            },
            (5, x, y, 0) => {
                // Skip next instruction if Vx = Vy
                let vx = self.registers[x as usize];
                let vy = self.registers[y as usize];
                if vx == vy {
                    self.pc += 2;
                }
            },
            (6, x, k1, k2) => {
                // Vx = kk
                let k: u8 = (k1 << 4) | k2; 
                self.registers[x as usize] = k;
            },
            (7, x, k1, k2) => {
                // Vx = Vx + kk
                let kk: u8 = (k1 << 4) | k2; 
                self.registers[x as usize] = (kk  as u16 + self.registers[x as usize] as u16) as u8;
            },
            (8, x, y, 0) => {
                // Set Vx = Vy.
                self.registers[x as usize] = self.registers[y as usize];
            },
            (8, x, y, 1) => {
                // Set Vx = Vx OR Vy.
                let vx = self.registers[x as usize];
                let vy = self.registers[y as usize];

                self.registers[x as usize] = vx | vy;
            },
            (8, x, y, 2) => {
                // Vx = Vx AND Vy
                let vx = self.registers[x as usize];
                let vy = self.registers[y as usize];

                self.registers[x as usize] = vx & vy;
            },
            (8, x, y, 3) => {
                // Vx = Vx AND Vy
                let vx = self.registers[x as usize];
                let vy = self.registers[y as usize];

                self.registers[x as usize] = vx ^ vy;
            },
            (8, x, y, 4) => {
                // Set Vx = Vx + Vy, set VF = carry.
                let vx = self.registers[x as usize];
                let vy = self.registers[y as usize];
                let value: u16 = vx as u16 + vy as u16;
                if value > 255 {
                    self.registers[0xF] = 1
                } else {
                    self.registers[0xF] = 0
                }

                self.registers[x as usize] = value as u8;
            },
            (8, x, y, 5) => {
                // Set Vx = Vx - Vy, set VF = NOT borrow.
                let vx = self.registers[x as usize];
                let vy = self.registers[y as usize];
                self.registers[x as usize] = vx.wrapping_sub(vy);

                if vx > vy {
                    self.registers[0xF] = 1
                } else {
                    self.registers[x as usize] = 0;
                }
            },
            (8, x, _ , 6) => {
                // Set Vx = Vx SHR 1.
                self.registers[0xF] = self.registers[x as usize] & 0b0000_0001;
                self.registers[x as usize] = self.registers[x as usize] >> 1;
            },
            (8, x, y, 7) => {
                // Set Vx = Vy - Vx, set VF = NOT borrow.
                let vx = self.registers[x as usize];
                let vy = self.registers[y as usize];

                self.registers[x as usize] = vy - vx;
                if vy > vx {
                    self.registers[0xF] = 1
                } else {
                    self.registers[0xF] = 0
                }
            },
            (8, x,_ , 0xE) => {
                //  Set Vx = Vx SHL 1.
                self.registers[0xF] = self.registers[x as usize] >> 7;
                self.registers[x as usize] = self.registers[x as usize] << 1;
            },
            (9, x, y, 0) => {
                //  Skip next instruction if Vx != Vy.
                let vx = self.registers[x as usize];
                let vy = self.registers[y as usize];

                if vx != vy {
                    self.pc += 2;
                }
            },
            (0xA, n1, n2, n3) => {
                // Set I = nnn.
                self.index = (n1 as u16) << 8 | (n2 as u16) << 4 | (n3 as u16);
            },
            (0xB, n1, n2, n3) => {
                //  Jump to location nnn + V0.
                let v0 = self.registers[0];
                let addr = (n1 as u16) << 8 | (n2 as u16) << 4 | (n3 as u16);
                self.pc = v0 as u16 + addr;
            },
            (0xC, x, k1, k2) => {
                // Set Vx = random byte AND kk.
                let kk: u8 = (k1 << 4) | k2; 
                let mut rng = rand::thread_rng();
                let value: u8 = rng.gen();
                self.registers[x as usize] = value & kk;
            },
            (0xD, x, y, n) => {
                // Draw sprite, x, y, number of bytes
                let x = self.registers[x as usize];
                let y = self.registers[y as usize];

                for row in 0..n {
                    let sprite = self.memory[(self.index + row as u16)as usize];
                    for column in 0..8 {
                        let pixel = (sprite >> (7 - column)) & 0b0000_0001;
                        let y = (y as usize + row as usize)  % SCREEN_HEIGHT;
                        let x = (x as usize + column) % SCREEN_WIDTH;
                        let coordinates = (y * SCREEN_WIDTH) + x; 
                        if pixel == 0 && self.screen[coordinates] == 1 {
                            if self.screen[coordinates] == 1 {
                                self.registers[0xF] = 1;
                            } 
                        }

                        self.screen[coordinates] ^= pixel;
                    }
                }
            },
            (0xE, x, 9, 0xE) => {
                // Skip next instruction if key with the value of Vx is pressed.
                let vx = self.registers[x as usize];
                //self.window.nodelay(false);
                if self.keypad[vx as usize] == 1 {
                    self.pc += 2;
                    self.keypad[vx as usize] = 0;
                }
            },
            (0xE, x, 0xA, 1) => {
                // Skip next instruction if key with the value of Vx is not pressed.
                let vx = self.registers[x as usize];
                //self.window.nodelay(false);
                if self.keypad[vx as usize] == 0 {
                    self.pc += 2;
                }
            },
            (0xF, x, 1, 5) => {
                // Set delay timer = Vx
                let vx = self.registers[x as usize];
                self.timer = vx;
            },
            (0xF, x, 0, 7) => {
                // Set Vx = delay timer value.
                self.registers[x as usize] = self.timer;
            },
            (0xF, x, 1, 8) => {
                // Set sound timer = Vx.
                self.sound_timer = self.registers[x as usize];
            },
            (0xF, x, 1, 0xE) => {
                // Set I = I + Vx.
                self.index += self.registers[x as usize] as u16;
            },
            (0xF, x, 2, 9) => {
                // Set I = location of sprite for digit Vx.
                let vx = self.registers[x as usize];
                self.index = vx as u16 * 15 + Self::SPRITE_LOCATION as u16;
            },
            (0xF, x, 3, 3) => {
                // Store BCD representation of Vx in memory locations I, I+1, and I+2.
                let vx = self.registers[x as usize];
                self.memory[self.index as usize + 2] = vx % 10;
                self.memory[self.index as usize + 1] = (vx % 10) / 10;
                self.memory[self.index as usize] = (vx % 10) / 100;
            },
            (0xF, x, 5, 5) => {
                //  Store registers V0 through Vx in memory starting at location I.
                for i in 0..=x {
                    self.memory[self.index as usize + i as usize] = self.registers[i as usize];
                }
            },
            (0xF, x, 6, 5) => {
                // Read registers V0 through Vx from memory starting at location I.
                for i in 0..=x {
                    let value = self.memory[self.index as usize + i as usize];
                    self.registers[i as usize] = value;
                }
            },
            (a, b, c, d) => {
                println!("missing opcode {:X}{:X}{:X}{:X}", a, b, c, d);
                panic!()
            }
        }

    }
}

impl Drop for Chip8 {
    fn drop(&mut self) {
        endwin();
    }
}
