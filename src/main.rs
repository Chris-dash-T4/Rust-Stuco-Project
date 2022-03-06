use fltk::{app, button::Button, frame::Frame, prelude::*, window::Window};
use std::{io::Result, io::prelude::*, io::BufReader, fs::File};
use arboard::Clipboard;

fn do_thing(frame : &mut Frame) {
    let f_ = File::open("sample.txt");
    match f_ {
      Err(_) => { frame.set_label("We do a little trolling."); },
      Ok(f) => {
        let mut reader = BufReader::new(f);
        let mut buffer = String::new();
        match reader.read_line(&mut buffer) {
          Err(_) => { frame.set_label("We do a little trolling."); },
          Ok(_) => {
            frame.set_label(&buffer);
            let mut clipboard = Clipboard::new().unwrap();
            clipboard.set_text(buffer.into()).unwrap();
          }
        }
      }
    };
}

fn main() -> Result<()> {
    let app = app::App::default();
    let mut wind = Window::new(100, 100, 400, 300, "Hello from rust");
    let mut frame = Frame::new(0, 0, 400, 200, "");
    let mut but = Button::new(160, 210, 80, 40, "Click me!");
    wind.end();

    wind.show();
    but.set_callback(move |_| { do_thing(&mut frame) });
    app.run().unwrap();
    Ok(())
}
