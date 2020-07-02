use std::fs;
use std::io;
use rand::Rng;
use std::{thread, time};

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::video::Window;
use sdl2::render::Canvas;
use sdl2::Sdl;
use sdl2::rect::Rect;

const APP_START: u16 = 0x200;
const SCREEN_WIDTH: usize = 64;
const SCREEN_HEIGHT: usize = 32;
const NO_CURSES: bool = false;
//const NO_CURSES: bool = true;
//const BREAKPOINT: usize = 100;
const BREAKPOINT: usize = 0;

fn main() {
    //let numero = 0x3;
    //let numero2 = 0xF;
    //let numero3 = numero <<  4 | numero2;
    //println!("{:x}", numero3);
    let mut chip8 = Chip8::new();
    chip8.load("pong.ch8").unwrap();
    chip8.load("space.ch8").unwrap();
    //chip8.load("zero.ch8").unwrap();
    chip8.start();
}


struct Chip8 {
    registers: Vec<u8>,
    memory: Vec<u8>,
    index: u16,
    size: usize,
    pc: u16,
    stack: Vec<u16>,
    stack_pointer: u8,
    screen: Vec<u8>,
    cycles: usize,
    timer: u8,
    keypad: Vec<u8>,
    sound_timer: u8,
    logs: Vec<String>,
    canvas: Canvas<Window>,
    sdl_context: Sdl,
}

#[allow(dead_code)]
impl Chip8 {
    const SPRITE_LOCATION: u8 = 0x50;

    pub fn new() -> Chip8 {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        let window = video_subsystem.window("chip8", (SCREEN_WIDTH * 10) as u32, (SCREEN_HEIGHT * 10) as u32)
            .position_centered()
            .build()
            .expect("could not initialize video subsystem");

        let mut canvas = window.into_canvas().build()
            .expect("could not make a canvas");

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.present();

        Chip8 {
            registers: (0..=15).map(|_| 0).collect(),
            memory: (0..=4096).map(|_| 0).collect(),
            index: 0,
            pc: 0,
            stack: Vec::with_capacity(16),
            stack_pointer: 0,
            screen: (0..=(SCREEN_WIDTH * SCREEN_HEIGHT)).map(|_| 0).collect(),
            size: 0,
            cycles: 0,
            timer: 0,
            keypad: (0..=15).map(|_| 0).collect(),
            sound_timer: 0,
            logs: Vec::new(),
            canvas,
            sdl_context,

        }
    }

    pub fn draw(&mut self) {
        //self.print_debug();
        let mut x: i32 = 0;
        let mut y: i32 = 0;
        for i in self.screen.iter() {
            if x >= SCREEN_WIDTH as i32 {
                x = 0;
                y += 1;
            }

            if *i > 0 {
                self.canvas.set_draw_color(Color::RGB(255, 255, 255));
            } else {
                self.canvas.set_draw_color(Color::RGB(0, 0, 0));
            }

            let display_x: i32 = x * 10;
            let display_y: i32 = y * 10;
            self.canvas.fill_rect(Rect::new(display_x, display_y, 10, 10));
            x +=1;
        }
    }

    pub fn start(&mut self) {
        self.load_fonts();
        let mut event_pump = self.sdl_context.event_pump().unwrap();

        'running: loop {
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit {..} |
                    Event::KeyDown { keycode: Some(Keycode::Escape), .. } => { break 'running; }
                    Event::KeyDown { keycode: Some(Keycode::Num1), .. }  => { self.keypad[0] = 1; } // 1
                    Event::KeyDown { keycode: Some(Keycode::Num2), .. } => { self.keypad[1] = 1; } // 2
                    Event::KeyDown { keycode: Some(Keycode::Num3), .. } => { self.keypad[2] = 1; } // 3
                    Event::KeyDown { keycode: Some(Keycode::Num4), .. } => { self.keypad[4] = 1; } // C
                    Event::KeyDown { keycode: Some(Keycode::Q), .. } => { self.keypad[5] = 1; } // 4
                    Event::KeyDown { keycode: Some(Keycode::W), .. } => { self.keypad[6] = 1; } // 5
                    Event::KeyDown { keycode: Some(Keycode::E), .. } => { self.keypad[7] = 1; } // 5
                    Event::KeyDown { keycode: Some(Keycode::R), .. } => { self.keypad[8] = 1; } // D
                    Event::KeyDown { keycode: Some(Keycode::A), .. } => { self.keypad[9] = 1; } // 7
                    Event::KeyDown { keycode: Some(Keycode::S), .. } => { self.keypad[0xA] = 1; } // 8
                    Event::KeyDown { keycode: Some(Keycode::D), .. } => { self.keypad[0xB] = 1; } // 9
                    Event::KeyDown { keycode: Some(Keycode::F), .. } => { self.keypad[0xC] = 1; } // E
                    Event::KeyDown { keycode: Some(Keycode::Z), .. } => { self.keypad[0xD] = 1; } // A
                    Event::KeyDown { keycode: Some(Keycode::X), .. } => { self.keypad[0xE] = 1; } // 0
                    Event::KeyDown { keycode: Some(Keycode::C), .. } => { self.keypad[0xF] = 1; } // B
                    Event::KeyDown { keycode: Some(Keycode::V), .. } => { self.keypad[0x10] = 1; } // F
                    _ => {}
                }
            }

            self.draw();
            self.canvas.present();
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

    pub fn print_debug(&self) {
        for i in 0..=0xF {
            print!("V{:X}: {:X} | ", i, self.registers[i]);
        }

        println!("\n");

        for i in 0..=0xF {
           print!("{:?}: {:?} | ", i, self.keypad[i]);
        }

        println!("\n");

        println!("pc {:X}\n", self.pc);
        println!("index {:X}\n", self.index);
        println!("cycles {:?}\n", self.cycles);
        println!("timer {:?}\n", self.timer);

        let opcode1 = self.memory[self.pc as usize];
        let opcode2 = self.memory[self.pc as usize + 1];
        println!("opcode {:02X}{:02X} \n", opcode1, opcode2);

        for log in self.logs.iter() {
          println!("{:?}", log);
        }
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
                self.logs.push(format!("{:04X}: 00EE\t RET", self.pc - 2));
                self.pc = self.stack.pop().unwrap();
            },
            (1, n1, n2, n3) => {
                // Jump to location nnn
                self.logs.push(format!("{:04X}: {:X}{n1:X}{n2:X}{n3:X} \t JUMP {n1:X}{n2:X}{n3:X}", self.pc - 2, 1, n1=n1, n2=n2, n3=n3));
                let addr: u16 = (n1 as u16) << 8 | (n2 as u16) << 4 | (n3 as u16);
                self.pc = addr;
            },
            (2, n1, n2, n3) => {
                // Call subroutine at nnn.
                self.stack.push(self.pc);
                let addr: u16 = (n1 as u16) << 8 | (n2 as u16) << 4 | (n3 as u16);
                self.pc = addr;
                self.logs.push(format!("{:04X}: {:X}{n1:X}{n2:X}{n3:X} \t CALL {n1:X}{n2:X}{n3:X}", self.pc - 2, 1, n1=n1, n2=n2, n3=n3));
            },
            (3, x, k1, k2) => {
                //  Skip next instruction if Vx = kk.
                let vx = self.registers[x as usize];
                let kk: u8 = (k1 << 4) | k2; 
                self.logs.push(format!("{:04X}: {:X}{n1:X}{n2:X}{n3:X} \t JE V{n1:X}, {n2:X}{n3:X}", self.pc - 2, 3, n1=x, n2=k1, n3=k2));
                if vx == kk {
                    self.pc += 2;
                }
            },
            (4, x, k1, k2) => {
                // Skip next instruction if Vx != kk.
                let vx = self.registers[x as usize];
                let kk: u8 = (k1 << 4) | k2; 

                self.logs.push(format!("{:04X}: {:X}{n1:X}{n2:X}{n3:X} \t JNE V{n1:X}, {kk:X}", self.pc - 2, 4, n1=x, n2=k1, n3=k2, kk=kk));
                if vx != kk {
                    self.pc += 2;
                }
            },
            (5, x, y, 0) => {
                // Skip next instruction if Vx = Vy
                let vx = self.registers[x as usize];
                let vy = self.registers[y as usize];

                self.logs.push(format!("{:04X}: {:X}{n1:X}{n2:X}{n3:X} \t JE V{n1:X}, V{n2:X}", self.pc - 2, 5, n1=x, n2=y, n3=0));
                if vx == vy {
                    self.pc += 2;
                }
            },
            (6, x, k1, k2) => {
                // Vx = kk
                self.logs.push(format!("{:04X}: {:X}{x:X}{k1:X}{k2:X} \t MOV V{x:X}, {k1:X}{k2:X}", self.pc - 2, 6, x=x, k1=k1, k2=k2));
                let k: u8 = (k1 << 4) | k2; 
                self.registers[x as usize] = k;
            },
            (7, x, k1, k2) => {
                // Vx = Vx + kk
                let kk: u8 = (k1 << 4) | k2; 
                self.logs.push(format!("{:04X}: {:X}{x:X}{k1:X}{k2:X} \t ADD V{x:X}, {k1:X}{k2:X}", self.pc - 2, 7, x=x, k1=k1, k2=k2));
                self.registers[x as usize] = (kk  as u16 + self.registers[x as usize] as u16) as u8;
            },
            (8, x, y, 0) => {
                // Set Vx = Vy.
                self.logs.push(format!("{:04X}: {:X}{x:X}{y:X}{k2:X} \t MOV V{x:X}, V{y:X}", self.pc - 2, 8, x=x, y=y, k2=0));
                self.registers[x as usize] = self.registers[y as usize];
            },
            (8, x, y, 1) => {
                // Set Vx = Vx OR Vy.
                self.logs.push(format!("{:04X}: {:X}{x:X}{y:X}{k2:X} \t OR V{x:X}, V{y:X}", self.pc - 2, 8, x=x, y=y, k2=1));
                let vx = self.registers[x as usize];
                let vy = self.registers[y as usize];

                self.registers[x as usize] = vx | vy;
            },
            (8, x, y, 2) => {
                // Vx = Vx AND Vy
                self.logs.push(format!("{:04X}: {:X}{x:X}{y:X}{k2:X} \t AND V{x:X}, V{y:X}", self.pc - 2, 8, x=x, y=y, k2=2));
                let vx = self.registers[x as usize];
                let vy = self.registers[y as usize];

                self.registers[x as usize] = vx & vy;
            },
            (8, x, y, 3) => {
                // Vx = Vx XOR Vy
                self.logs.push(format!("{:04X}: {:X}{x:X}{y:X}{k2:X} \t XOR V{x:X}, V{y:X}", self.pc - 2, 8, x=x, y=y, k2=3));
                let vx = self.registers[x as usize];
                let vy = self.registers[y as usize];

                self.registers[x as usize] = vx ^ vy;
            },
            (8, x, y, 4) => {
                // Set Vx = Vx + Vy, set VF = carry.
                self.logs.push(format!("{:04X}: {:X}{x:X}{y:X}{k2:X} \t ADC V{x:X}, V{y:X}", self.pc - 2, 8, x=x, y=y, k2=4));
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
                self.logs.push(format!("{:04X}: {:X}{x:X}{y:X}{k2:X} \t SUB V{x:X}, V{y:X}", self.pc - 2, 8, x=x, y=y, k2=5));
                let vx = self.registers[x as usize];
                let vy = self.registers[y as usize];

                if vx > vy {
                    self.registers[0xF] = 1
                } else {
                    self.registers[x as usize] = 0;
                }

                self.registers[x as usize] = vx.wrapping_sub(vy);
            },
            (8, x, k , 6) => {
                // Set Vx = Vx SHR 1.
                self.logs.push(format!("{:04X}: {:X}{x:X}{k1:X}{k2:X} \t SHR V{x:X}", self.pc - 2, 8, x=x, k1=k, k2=6));
                self.registers[0xF] = self.registers[x as usize] & 0b0000_0001;
                self.registers[x as usize] = self.registers[x as usize] >> 1;
            },
            (8, x, y, 7) => {
                // Set Vx = Vy - Vx, set VF = NOT borrow.
                self.logs.push(format!("{:04X}: {:X}{x:X}{y:X}{k2:X} \t SUBN V{x:X}, V{y:X}", self.pc - 2, 8, x=x, y=y, k2=7));
                let vx = self.registers[x as usize];
                let vy = self.registers[y as usize];

                if vy > vx {
                    self.registers[0xF] = 1
                } else {
                    self.registers[0xF] = 0
                }

                self.registers[x as usize] = vy - vx;
            },
            (8, x, k, 0xE) => {
                //  Set Vx = Vx SHL 1.
                self.logs.push(format!("{:04X}: {:X}{x:X}{k1:X}{k2:X} \t SHL V{x:X}", self.pc - 2, 8, x=x, k1=k, k2=0xE));
                self.registers[0xF] = self.registers[x as usize] >> 7;
                self.registers[x as usize] = self.registers[x as usize] << 1;
            },
            (9, x, y, 0) => {
                //  Skip next instruction if Vx != Vy.
                let vx = self.registers[x as usize];
                let vy = self.registers[y as usize];

                self.logs.push(format!("{:04X}: {:X}{n1:X}{n2:X}{n3:X} \t JNE V{n1:X}, V{n2:X}", self.pc - 2, 9, n1=x, n2=y, n3=0));
                if vx != vy {
                    self.pc += 2;
                }
            },
            (0xA, n1, n2, n3) => {
                // Set I = nnn.
                self.logs.push(format!("{:04X}: {:X}{n1:X}{n2:X}{n3:X} \t MOV I, {n1:X}{n2:X}{n3:X}", self.pc - 2, 0xA, n1=n1, n2=n2, n3=n3));
                self.index = (n1 as u16) << 8 | (n2 as u16) << 4 | (n3 as u16);
            },
            (0xB, n1, n2, n3) => {
                //  Jump to location nnn + V0.
                let v0 = self.registers[0];
                let addr = (n1 as u16) << 8 | (n2 as u16) << 4 | (n3 as u16);
                self.logs.push(format!("{:04X}: {:X}{n1:X}{n2:X}{n3:X} \t JMP {addr:X}", self.pc - 2, 0xB, n1=n1, n2=n2, n3=n3, addr=addr));
                self.pc = v0 as u16 + addr;
            },
            (0xC, x, k1, k2) => {
                // Set Vx = random byte AND kk.
                let kk: u8 = (k1 << 4) | k2; 
                let mut rng = rand::thread_rng();
                let value: u8 = rng.gen();
                self.logs.push(format!("{:04X}: {:X}{x:X}{k1:X}{k2:X} \t MOV V{x:x}, {value:X}", self.pc - 2, 0xC, x=x, k1=k1, k2=k2, value=value));
                self.registers[x as usize] = value & kk;
            },
            (0xD, vx, vy, n) => {
                // Draw sprite, x, y, number of bytes
                let base_x = self.registers[vx as usize];
                let base_y = self.registers[vy as usize];
                self.logs.push(format!("{:04X}: {:X}{vx:X}{vy:X}{n:X} \t DRW {x:X},{y:X}", self.pc - 2, 0xD, x=base_x, y=base_y, n=n, vx=vx,vy=vy));

                let mut vf_value = 0;

                for row in 0..n {
                    let sprite = self.memory[(self.index + row as u16)as usize];
                    let y = (base_y as usize + row as usize) % SCREEN_HEIGHT;

                    self.logs.push(format!("{:b}", sprite));
                    for column in 0..8 {
                        let pixel = (sprite >> (7 - column)) & 0b0000_0001;
                        //let x = (x as usize + column) % SCREEN_WIDTH;
                        let mut x = (base_x as usize + column);
                        if x >= SCREEN_WIDTH {
                            x = SCREEN_WIDTH - (x as usize % SCREEN_WIDTH);
                        }

                        let coordinates = (y * SCREEN_WIDTH) + x; 


                        if pixel == 1 && self.screen[coordinates] == 1 {
                            self.logs.push(format!("{:?}", column));
                            vf_value = 1;

                            if x == 61 && pixel==1 {
                                //self.window.nodelay(false);
                            }
                        }

                        self.screen[coordinates] ^= pixel;
                    }
                }

                self.registers[0xF] = vf_value;
                self.logs.push(format!("{:04X}: {:X}{vx:X}{vy:X}{n:X} \t MOV VF, {value:X}", self.pc - 2, 0xD, n=n, vx=vx,vy=vy, value=vf_value));
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

                self.keypad[vx as usize] = 0;
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
                self.index = vx as u16 * 5 + Self::SPRITE_LOCATION as u16;
            },
            (0xF, x, 3, 3) => {
                // Store BCD representation of Vx in memory locations I, I+1, and I+2.
                let vx = self.registers[x as usize];
                self.memory[self.index as usize] = vx / 100;
                self.memory[self.index as usize + 1] = (vx % 100) / 10;
                self.memory[self.index as usize + 2] = vx % 10;
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
            (0x0, 0x0, 0xE, 0) => {
                // CLEAN SCREEN
                self.logs.push(format!("{:04X}: 00E0 \t CLS", self.pc - 2));
                for i in 0..SCREEN_WIDTH * SCREEN_HEIGHT {
                    self.screen[i] = 0;
                }
            },
            (a, b, c, d) => {
                println!("missing opcode {:X}{:X}{:X}{:X}", a, b, c, d);
                panic!()
            }
        }

    }
}
