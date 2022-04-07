use fltk::{app, button::Button, frame::Frame, prelude::*, window::Window};
use std::{io::Result, io::prelude::*, io::BufReader, fs::File, path::PathBuf,option::Option};
use arboard::Clipboard;
use serde_json::Value;
use clap::Parser;
mod lex;

#[derive(Parser)]
struct Cli {
  #[clap(short,long)]
  graphical: bool,
  #[clap(parse(from_os_str))]
  path: PathBuf,
  pattern: Vec<String>,
}

fn do_thing(frame : &mut Frame) -> Result<()> {
    let f = File::open("sample.txt")?;
    let mut reader = BufReader::new(f);
    let mut buffer = String::new();
    let _ = reader.read_line(&mut buffer)?;
    frame.set_label(&buffer);
    let mut clipboard = Clipboard::new().unwrap();
    clipboard.set_text(buffer.into()).unwrap();
    let f2 = File::open("out.json")?;
    let reader2 = BufReader::new(f2);
    let v : Value = serde_json::from_reader(reader2)?;
    println!("{}",v[0]["word"]);
    Ok(())
}

fn main() -> Result<()> {
    let args = Cli::parse();

    if !args.graphical {
        let mut ws : Vec<lex::Word<_>> = Vec::new();
        let f2 = File::open(args.path)?;
        let reader2 = BufReader::new(f2);
        let json : Value = serde_json::from_reader(reader2)?;
        for s in &args.pattern {
            let mut data = s.split("+");
            let root = String::from(Option::unwrap(data.next()));
            let xs : Vec<String> = data.map(String::from).collect();
            let mut w = lex::get_word(&root,&json)?;
            for x in xs {
                let a = lex::get_attr(x,&json)?;
                w = lex::add_attr(w,a);
            }
            ws.push(w);
        }
        println!("{} {}",lex::inflect(&ws[0]),lex::inflect(&ws[1]));
        println!("{} {}",lex::gloss(&ws[0]),lex::gloss(&ws[1]));
        return Ok(())
    }
    let app = app::App::default();
    let mut wind = Window::new(100, 100, 400, 300, "Hello from rust");
    let mut frame = Frame::new(0, 0, 400, 200, "");
    let mut but = Button::new(160, 210, 80, 40, "Click me!");
    wind.end();

    wind.show();
    but.set_callback(move |_| {
        match do_thing(&mut frame) {
          Err(_) => { frame.set_label("We do a little trolling."); },
          Ok(_) => (),
        }
    });
    app.run().unwrap();
    Ok(())
}
