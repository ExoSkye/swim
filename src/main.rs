extern crate sdl2;
extern crate derive_more;
use std::collections::HashMap;
use derive_more::{Add, Sub, Div, Mul, AddAssign, SubAssign, MulAssign, DivAssign};
use std::io::Read;
use std::fs::File;
use std::path::Path;
use std::env;
use sdl2::image::LoadTexture;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::hash::{Hash};
use std::convert::TryInto;
use std::time::Duration;


#[derive(Copy, Clone, PartialEq, Add, Sub, Div, Mul, AddAssign, SubAssign, MulAssign, DivAssign)]
struct Coord {
    x: f32,
    y: f32
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Add, Sub, Div, Mul, AddAssign, SubAssign, MulAssign, DivAssign)]
struct IntCoord {
    x: i32,
    y: i32
}

impl IntCoord {
    pub fn new(init: i32) -> IntCoord {
        IntCoord {
            x: init,
            y: init
        }
    }
}

impl Coord {
    pub fn new(init: f32) -> Coord {
        Coord {
            x: init,
            y: init
        }
    }

    pub fn get_char(&self) -> IntCoord {
        IntCoord {
            x: self.x as i32,
            y: self.y as i32
        }
    }
}

impl std::fmt::Display for Coord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}, {}", self.x, self.y)
    }
}

#[derive(Clone)]
struct Program {
   code: HashMap<IntCoord,char>
}

impl Program {
    pub fn new(file: String) -> Program {
        let path = Path::new(file.as_str());
        path.display();

        let mut program: Program = Program{
            code: HashMap::new()
        };

        let mut str: String = String::new();

        match File::open(&path) {
            Err(_) => {
                println!("Failed to open file, are you sure it exists?")
            },
            Ok(mut file) => {
                file.read_to_string(&mut str);
            },
        };

        let lines: Vec<&str> = str.split("\n").collect();

        let mut cur_coord: IntCoord = IntCoord::new(0);
        cur_coord.y = 8;

        for line in lines {
            cur_coord.x = 0;
            for char in line.chars() {
                match char {
                    ' ' => {},
                    _ => {
                        program.code.insert(cur_coord, char);
                    }
                }
                cur_coord.x += 1;
            }
            cur_coord.y -= 1;
        }

        program
    }
}

#[derive(Clone)]
struct State {
    pos: Coord,
    speed: Coord,
    accel: Coord,
    gravity: Coord,
    accumulator: f32,
    cells: [f32; 255],
    cell_ptr: usize,
    program: Program
}

impl State {
    pub fn new(file: String) -> State {
        State {
            pos: Coord{
                x: 0.0,
                y: 0.0
            },
            speed: Coord {
                x: 0.0,
                y: 0.0
            },
            accel: Coord {
                x: 0.0,
                y: 0.0
            },
            gravity: Coord {
                x: 0.0,
                y: 0.5
            },
            accumulator: 0.0,
            cells: [0.0f32;255],
            cell_ptr: 0,
            program: Program::new(file)
        }
    }

    pub fn update(&mut self,dt: f32) -> bool {
        self.accel -= self.gravity;
        self.pos.x += self.speed.x+(self.accel.x / 2.0);
        self.speed.x += self.accel.x;
        self.pos.y += self.speed.y+(self.accel.y / 2.0);
        self.speed.y += self.accel.y;
        self.speed.x *= dt.powf(0.1_f32);
        self.speed.y *= dt.powf(0.1_f32);
        self.accel += self.gravity;
        self.accel.x *= dt.powf(0.2_f32);
        self.accel.y *= dt.powf(0.2_f32);
        if self.pos.x < 0.0 {
            self.pos.x = 0.0;
        }
        if self.pos.y < -8.0f32 || self.pos.y > 8.0f32 {
            true
        }
        else {
            false
        }
    }
}

fn gen_opcodes() -> std::collections::HashMap<char, fn(&mut State) -> ()> {
    let mut opcodes: HashMap<char, fn(&mut State) -> ()> = HashMap::new();
    opcodes.insert('^',|mut state| {state.accel.y += 1.0});
    opcodes.insert('>',|mut state| {state.accel.x += 1.0});
    opcodes.insert('<',|mut state| {state.accel.x -= 1.0});
    opcodes.insert('+',|mut state| {state.cell_ptr += 1});
    opcodes.insert('-',|mut state| {state.cell_ptr -= 1});
    opcodes.insert('a',|mut state| {state.accumulator += state.cells[state.cell_ptr]});
    opcodes.insert('s',|mut state| {state.accumulator -= state.cells[state.cell_ptr]});
    opcodes.insert('d',|mut state| {state.accumulator /= state.cells[state.cell_ptr]});
    opcodes.insert('m',|mut state| {state.accumulator *= state.cells[state.cell_ptr]});
    opcodes.insert('%',|mut state| {state.accumulator %= state.cells[state.cell_ptr]});
    opcodes.insert('n',|mut state| {
        let mut coord: Coord = state.pos;
        coord.x += 1.0;
        let mut digits = String::new();
        while state.program.code.get(&coord.get_char()) != None {
            digits += match state.program.code.get(&coord.get_char()) {
                None => {break;},
                Some(inp) => {inp}
            }.to_string().as_str();
            coord.x += 1.0;
        }
        if digits.len() == 0 {
            state.accumulator = 0.0;
        }
        else {
            let mut out: u32 = 0;
            let mut i: i32 = digits.len() as i32;
            while i != 0 {
                out += (digits.chars().nth(digits.len()-i as usize)).unwrap().to_digit(10).unwrap()*10_u32.pow((i-1).try_into().unwrap());
                i-=1;
            }
            state.accumulator = out as f32;
        }
    });
    opcodes.insert('p',|mut state| {
        print!("{}",match char::from_u32(state.accumulator as u32) {
            None => ' ',
            Some(ch) => ch
        })
    });
    opcodes.insert('i',|mut state| {
        use std::io::stdin;
        let mut output: [u8; 1] = [' ' as u8];
        stdin().read(&mut output);
        state.accumulator = output[0] as u32 as f32;
    });
    opcodes.insert('e',|mut state| {
        if state.accumulator == state.cells[state.cell_ptr] {
            state.accel.x += 1.0f32;
        }
        else {
            state.accel.y -= 1.0f32;
        }
    });
    opcodes.insert('b',|mut state| {
        if state.accumulator > state.cells[state.cell_ptr] {
            state.accel.x += 1.0f32;
        }
        else {
            state.accel.x -= 1.0f32;
        }
    });
    opcodes.insert('m',|mut state| {
        if state.accumulator < state.cells[state.cell_ptr] {
            state.accel.x += 10.0f32;
        }
        else {
            state.accel.y -= 10.0f32;
        }
    });
    opcodes.insert('c',|mut state| {state.cells[state.cell_ptr] = state.accumulator});
    opcodes.insert('l',|mut state| {state.accumulator = state.cells[state.cell_ptr]});

    opcodes
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        let mut state = State::new(args[1].clone());
        let opcodes = gen_opcodes();

        // Setup SDL

        let sdl = sdl2::init().unwrap();
        let video_sys = sdl.video().expect("Failed to setup video subsystem");
        let window = video_sys.window("Swim",1280,720).position_centered().opengl().build().expect("Failed to create window");
        let mut canvas = window.into_canvas().build().expect("Failed to create window canvas");

        let texture_creator = canvas.texture_creator();

        let turtle = texture_creator.load_texture("turtle.png").expect("Couldn't open turtle texture");

        let ttf = sdl2::ttf::init().unwrap();

        let font = ttf.load_font("Roboto-Regular.ttf",24).unwrap();

        let mut event_pump = sdl.event_pump().unwrap();

        let mut x_offset = 0;

        'running: loop {
            let start = std::time::Instant::now();
            if state.program.code.get(&state.pos.get_char()) != None {
                let command: char = *state.program.code.get(&state.pos.get_char()).unwrap();
                if command.is_digit(10) {
                    state.accumulator = command.to_digit(10).unwrap() as f32;
                }
                else {
                    opcodes[&command](&mut state);
                }
            }
            canvas.clear();
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit {..} |
                    Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                        break 'running
                    },
                    _ => {}
                }
            }

            for x in 0+x_offset..72+x_offset {
                for y in -8..8 {
                    if state.program.code.get(&IntCoord{x,y}) != None {
                        let character = state.program.code.get(&IntCoord{x,y}).unwrap();
                        let char_surf = font.render_char(*character).solid(sdl2::pixels::Color::RGB(255,255,255)).unwrap();
                        let width = char_surf.width();
                        let height = char_surf.height();
                        let char_tex = texture_creator.create_texture_from_surface(char_surf).unwrap();
                        canvas.copy(&char_tex, None, sdl2::rect::Rect::new(x*(1280/72), (9-y)*(720/17),width,height));
                    }
                }
            }
            canvas.copy(&turtle,None,sdl2::rect::Rect::new(((state.pos.x*32.0) % 1280.0)  as i32, 720-((state.pos.y+8.0)*45.0) as i32, 32, 32)).unwrap();
            canvas.present();
            match state.update(start.elapsed().as_secs_f32()) {
                true => {break;},
                false => {}
            }
            ::std::thread::sleep(Duration::new(0,1_000_000_000/60))
        };
    }
    else {
        panic!("Not enough arguments!")
    }
}
