use libc::{c_char, c_void, input_event, ioctl, open, read, O_RDONLY};

const EVIOCGRAB: u64 = 1074021776;

// From linux/input-event-codes.h
const EV_KEY: u16 = 0x01;
const KEY_CAPSLOCK: u16 = 58;

const KEY_H: u16 = 35;
const KEY_J: u16 = 36;
const KEY_K: u16 = 37;
const KEY_L: u16 = 38;

const KEY_UP: u16 = 103;
const KEY_LEFT: u16 = 105;
const KEY_RIGHT: u16 = 106;
const KEY_DOWN: u16 = 108;

const KEY_DELETE: u16 = 111;
const KEY_HOME: u16 = 102;
const KEY_END: u16 = 107;
const KEY_PAGE_UP: u16 = 104;
const KEY_PAGE_DOWN: u16 = 109;
const KEY_INSERT: u16 = 110;

const KEY_BACKSPACE: u16 = 14;
const KEY_LEFT_BRACE: u16 = 26;
const KEY_RIGHT_BRACE: u16 = 27;
const KEY_ENTER: u16 = 28;
const KEY_APOSTROPHE: u16 = 40;
const KEY_SLASH: u16 = 53;

const KEY_1: u16 = 2;
const KEY_2: u16 = 3;
const KEY_3: u16 = 4;
const KEY_4: u16 = 5;
const KEY_5: u16 = 6;
const KEY_6: u16 = 7;
const KEY_7: u16 = 8;
const KEY_8: u16 = 9;
const KEY_9: u16 = 10;
const KEY_0: u16 = 11;
const KEY_MINUS: u16 = 12;
const KEY_EQUAL: u16 = 13;

const KEY_F1: u16 = 59;
const KEY_F2: u16 = 60;
const KEY_F3: u16 = 61;
const KEY_F4: u16 = 62;
const KEY_F5: u16 = 63;
const KEY_F6: u16 = 64;
const KEY_F7: u16 = 65;
const KEY_F8: u16 = 66;
const KEY_F9: u16 = 67;
const KEY_F10: u16 = 68;
const KEY_F11: u16 = 87;
const KEY_F12: u16 = 88;

pub struct KeyboardHandler {
    fd: i32,
    uinput: uinput::Device,
    is_grabbed: bool,
    debug: bool,
    device_path: String,
}

impl KeyboardHandler {
    pub fn new(device_path: &String, debug: bool) -> KeyboardHandler {
        unsafe {
            let fd = open(device_path[..].as_ptr() as *const c_char, O_RDONLY);
            if fd == -1 {
                panic!("Cannot open input device: {}", device_path);
            }

            KeyboardHandler {
                device_path: device_path.to_string(),
                is_grabbed: false,
                uinput: uinput::default()
                    .unwrap()
                    .name(format!("C-HJKL Output for {}", device_path))
                    .unwrap()
                    .event(uinput::event::Keyboard::All)
                    .unwrap()
                    .create()
                    .unwrap(),
                debug,
                fd,
            }
        }
    }

    fn grab(&mut self) {
        unsafe {
            if !self.is_grabbed && ioctl(self.fd, EVIOCGRAB, 1) != -1 {
                self.is_grabbed = true;
            }
        }
    }

    #[allow(dead_code)]
    fn ungrab(&mut self) {
        unsafe {
            ioctl(self.fd, EVIOCGRAB, 0);
            self.is_grabbed = false;
        }
    }

    fn read(&self) -> input_event {
        unsafe {
            let mut ev: input_event = std::mem::zeroed();
            if read(
                self.fd,
                &mut ev as *mut _ as *mut c_void,
                std::mem::size_of::<input_event>(),
            ) != (std::mem::size_of::<input_event>() as _)
            {
                panic!("Read a partial event");
            }
            ev.clone()
        }
    }

    fn write(&mut self, ev: &input_event) {
        self.uinput
            .write(ev.type_ as _, ev.code as _, ev.value)
            .unwrap();
    }

    pub fn run_forever(&mut self) {
        let mut caps_pressed = false;

        std::thread::sleep(std::time::Duration::from_secs(1));

        self.grab();
        loop {
            let mut input = self.read();

            if self.debug {
                println!(
                    "[{}] caps:{}, ev: {} {} {}",
                    self.device_path, caps_pressed, input.type_, input.code, input.value
                );
            }

            // Maintain caps flag, intercept the press so default caps behaviour gets disabled
            if input.type_ == EV_KEY && input.code == KEY_CAPSLOCK {
                caps_pressed = input.value != 0;
                continue;
            }

            // Handle Caps-hjkl
            if input.type_ == EV_KEY && input.value >= 1 && caps_pressed {
                let key_to_press = if input.code == KEY_H {
                    KEY_LEFT
                } else if input.code == KEY_J {
                    KEY_DOWN
                } else if input.code == KEY_K {
                    KEY_UP
                } else if input.code == KEY_L {
                    KEY_RIGHT
                } else if input.code == KEY_BACKSPACE {
                    KEY_DELETE
                } else if input.code == KEY_LEFT_BRACE {
                    KEY_HOME
                } else if input.code == KEY_RIGHT_BRACE {
                    KEY_END
                } else if input.code == KEY_APOSTROPHE {
                    KEY_PAGE_UP
                } else if input.code == KEY_SLASH {
                    KEY_PAGE_DOWN
                } else if input.code == KEY_ENTER {
                    KEY_INSERT
                } else if input.code == KEY_1 {
                    KEY_F1
                } else if input.code == KEY_2 {
                    KEY_F2
                } else if input.code == KEY_3 {
                    KEY_F3
                } else if input.code == KEY_4 {
                    KEY_F4
                } else if input.code == KEY_5 {
                    KEY_F5
                } else if input.code == KEY_6 {
                    KEY_F6
                } else if input.code == KEY_7 {
                    KEY_F7
                } else if input.code == KEY_8 {
                    KEY_F8
                } else if input.code == KEY_9 {
                    KEY_F9
                } else if input.code == KEY_0 {
                    KEY_F10
                } else if input.code == KEY_MINUS {
                    KEY_F11
                } else if input.code == KEY_EQUAL {
                    KEY_F12
                } else {
                    0
                };

                if key_to_press > 0 {
                    input.code = key_to_press;
                    input.value = 1;
                    self.write(&input);

                    input.value = 0;
                    self.write(&input);

                    continue;
                }
            }

            // Pass-through
            self.write(&input);
        }
    }
}
