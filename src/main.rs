
use std::fs;
use std::io;

use pancurses::{initscr, endwin, Window, Input, noecho};

const APP_START: u16 = 0x200;
const SCREEN_WIDTH: usize = 64;
const SCREEN_HEIGHT: usize = 32;

fn main() {
    let mut chip8 = Chip8::new();
    let file_length = chip8.load("pong.ch8").unwrap();
    chip8.print_memory(0x200, Some(APP_START as usize + file_length));
    chip8.start();

    // let window = initscr();
    // window.refresh();
    // window.getch();
    // endwin();
}


#[derive(Debug)]
struct Chip8 {
    registers: Vec<u8>,
    memory: Vec<u8>,
    index: u16,
    pc: u16,
    stack: Vec<u16>,
    stack_pointer: u8,
    screen: Vec<u8>,
    window: Window
}

#[allow(dead_code)]
impl Chip8 {
    pub fn new() -> Chip8 {
        Chip8 {
            registers: (0..15).map(|_| 0).collect(),
            memory: (0..4096).map(|_| 0).collect(),
            index: 0,
            pc: 0,
            stack: Vec::with_capacity(16),
            stack_pointer: 0,
            screen: (0..(SCREEN_WIDTH * SCREEN_HEIGHT)).map(|_| 0).collect(),
            window: initscr()
        }
    }

    pub fn draw(&self) {
        self.window.erase();
        for x in self.screen.iter() {
            let y = format!("{:08b}", x).replace("1", "#").replace("0", " ");
            self.window.printw(y);
            self.window.printw("\n");
        }
        self.window.refresh();
    }

    pub fn start(&mut self) {
        noecho();
        self.window.erase();
        self.load_fonts();

        loop {
            self.cycle();
            match self.window.getch() {
                    Some(Input::Character(c)) => { self.window.addch(c); },
                    Some(Input::KeyDC) => break,
                    _ => self.draw()
                }
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

        let font_start = 0x50;
        for (i, x) in fonts.iter().enumerate() {
            self.memory[font_start as usize + i] = *x;
        }
    }

    pub fn load(&mut self, path: &str) -> Result<usize, io::Error>{
        let content = fs::read(path)?;

        let start_address: u16 = APP_START;
        for (i, x) in content.iter().enumerate() {
            self.memory[start_address as usize + i] = *x;
        }

        self.pc = APP_START;

        Ok(content.len())
    }

    pub fn print_memory(&self, start_address: usize, end_address: Option<usize>) {
        let mut c = 0;
        let end_address = end_address.unwrap_or(self.memory.len());
        for i in start_address..end_address {
            if c % 12 == 0 {
                self.window.printw("\n");
            }

            let x = self.memory[i];
            self.window.printw(format!("${:04X}: {:02X} \t", i, x));

            c += 1;
        }

        self.window.refresh();
        self.window.getch();
    }

    fn cycle(&mut self) {
        let opcode = (self.memory[self.pc as usize] as u16) << 8 | (self.memory[(self.pc + 1) as usize] as u16);
        self.pc += 2;
        self.window.addstr(format!("{:016b} {:X}", opcode, opcode));
        self.execute_opcode(opcode);
    }

    fn to_tuple(self, hex: u16) {

    }

    fn execute_opcode(&mut self, opcode: u16) {
        match opcode {
            (0, 0, E, 0)
        }

    }
}

impl Drop for Chip8 {
    fn drop(&mut self) {
        endwin();
    }
}