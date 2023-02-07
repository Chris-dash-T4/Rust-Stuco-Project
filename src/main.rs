use std::{io::Result, /*io::prelude::*,*/ io::BufReader, fs::File, path::PathBuf,option::Option};
use fltk::{app, button::Button, frame::Frame, prelude::*, window::Window, input::Input, };
use fltk_table::{SmartTable, TableOpts};
//use arboard::Clipboard;
use serde_json::{Value,json};
use clap::Parser;
use rfd::FileDialog;
mod lex;
mod phon;

/// A machine-based glosser designed for Conlangers
#[derive(Parser)]
struct Cli {
  /// Print extra information about sound changes applied
  #[clap(short,long)]
  verbose: bool,
  /// Use GUI instead of command-line
  #[clap(short,long)]
  graphical: bool,
  #[clap(long)]
  /// (Command-line only) print output as a TeX-formatted table instead of a '\t'-separated table
  tex: bool,
  /// Default path to the JSON file containing the language information
  #[clap(short,parse(from_os_str))]
  file: Option<PathBuf>,
  /// The expression to gloss, formatted as 'Lemma+ATTR+ATTR+...' or '{MetalangWord}+ATTR+ATTR'
  pattern: Vec<String>,
}

struct GlossTable {
    inflections: Vec<String>,
    phonetic: Vec<String>,
    orthographic: Vec<String>,
    glosses: Vec<String>,
    len: usize
}

fn get_gloss_info(toks: &Vec<String>, json: &Value, verbose: bool) -> Result<GlossTable> {
    let mut ws : Vec<lex::Word<_>> = Vec::new();
    for s in toks {
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
    let mut inflections = Vec::new();
    let mut orthographic = Vec::new();
    let mut phonetic = Vec::new();
    let mut glosses = Vec::new();
    for w in &ws {
        inflections.push(lex::inflect(w));
        let orth = phon::to_orthography(inflections[inflections.len()-1].clone(),&json["sc"],&json["cats"],&json["multigraphs"],verbose);
        orthographic.push(orth);
        phonetic.push(phon::to_orthography(orthographic[orthographic.len()-1].clone(),&json["phonetic"],&json["cats"],&json!(null),verbose));
        glosses.push(lex::gloss(w));
    }
    let len = (&glosses).len();
    Ok(GlossTable { inflections, orthographic, phonetic, glosses, len })
}

fn start_gui(path : PathBuf, json : Value, verbose : bool) -> Result<()> {
    let app = app::App::default();
    let mut wind = Window::new(100, 100, 600, 300, "Hello from rust");
    let _frame = Frame::new(20, 0, 400, 50, "Enter gloss string:");
    let text = Input::new(20, 50, 360, 30, "");
    let mut stat_frame = Frame::new(400, 0, 200, 180, "");
    let mut gb = Button::new(460, 190, 80, 30, "Get Gloss");
    let _cb = Button::new(460, 240, 80, 30, "Copy Output");
    let mut table = SmartTable::default().with_size(380,180).with_pos(10,100)
                .with_opts(TableOpts {
                    rows: 4,
                    cols: 0,
                    editable: true,
                    ..Default::default()
                });
    table.end();
    wind.end();

    // TODO populate with actual data
    let stats = format!("File: {}\n{} lexemes, {} attributes.\n{} sound change rules.",
                        path.as_path().file_name().unwrap().to_str().unwrap(),
                        69,69,69);
    stat_frame.set_label(stats.as_str());

    wind.show();
    gb.set_callback(move |_| {
        let raw = &text.value();
        let split = raw.split(" ").map(|s| String::from(s)).collect();
        match get_gloss_info(&split, &json, verbose) {
            Err(e) => { 
                eprintln!("You done fucked up!\n{:?}",e);
                table.set_label("We do a little trolling.");
            },
            Ok(gt) => {
                table.set_opts(TableOpts {
                    rows: 4,
                    cols: gt.len as i32,
                    editable: true,
                    ..Default::default()
                });
                for i in 0..(gt.len) {
                    let j = i as i32;
                    table.set_cell_value(0, j, &gt.orthographic[i]);
                    table.set_cell_value(1, j, &gt.phonetic[i]);
                    table.set_cell_value(2, j, &gt.inflections[i]);
                    table.set_cell_value(3, j, &gt.glosses[i]);
                }
                table.set_label("We engage in a nontrivial quantity of shenanigans.");
            },
        }
    });
    app.run().unwrap();
    Ok(())
}

fn main() -> Result<()> {
    let args = Cli::parse();
    let raw_args : Vec<_> = std::env::args().collect();
    let path : PathBuf = match args.file {
        Some(f) => f,
        None => {
            FileDialog::new().add_filter("JSON files",&["json"]).pick_file().unwrap()
        }
    };
    let f2 = File::open(path.clone())?;
    let reader2 = BufReader::new(f2);
    let json : Value = serde_json::from_reader(reader2)?;

    if args.graphical || (raw_args.len() <= 1) {
        start_gui(path,json,args.verbose)
    }
    else {
        let gt = get_gloss_info(&args.pattern, &json, args.verbose)?;
        if args.tex {
            println!("\\begin{{tabular}}{{{}}}","l".repeat(gt.len));
            println!("\\textbf{{{}}}\\\\",gt.orthographic.join("}&\\textbf{"));
            println!("/\\textipa{{{}}}/\\\\",gt.phonetic.join("}&\\textipa{"));
            println!("{}\\\\",gt.inflections.join("&"));
            println!("{}\\\\",gt.glosses.join("&"));
            println!("\\end{{tabular}}",);
        } else {
            println!("{}",gt.inflections.join("\t"));
            println!("{}",gt.orthographic.join("\t"));
            println!("{}",gt.phonetic.join("\t"));
            println!("{}",gt.glosses.join("\t"));
        }
        Ok(())
    }
}
