mod opcodes;

extern crate sdl2;

struct Display {
    sdl: sdl2::Sdl,
    video_sys: Option<sdl2::VideoSubsystem>,
    window: Option<sdl2::video::Window>,
}

impl Display {
    pub fn new() -> Option<Display> {
        let mut disp = Display {
            sdl: sdl2::init().unwrap(),
            video_sys: None,
            window: None
        };
        disp.video_sys = Some(disp.sdl.video().unwrap());
        disp.window = Some(disp.video_sys.unwrap().window("Swim",1280,720).position_centered().opengl().build().unwrap());
        Some(disp)
    }
}

enum Opcode {

}

struct Program {

}

impl Program {

}

fn main() {

}
