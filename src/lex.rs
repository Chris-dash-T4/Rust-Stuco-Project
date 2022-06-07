use std::{cmp::Ordering,io::Result,io::Error,collections::hash_set::HashSet,fmt::Display};
use serde_json::Value;

fn cuo(s : &str) -> Error {
    Error::new(std::io::ErrorKind::Other,s)
}

#[derive(Hash,PartialEq,Eq,Clone)]
pub enum Wordclass { N, V, M, P, }

#[derive(PartialEq,Eq,Clone)]
pub struct Attribute {
    name: String,
    form: String,
    place: Ordering,
    affects: HashSet<Wordclass>,
}

pub trait Affect {
    fn can_affect(&self, c : Wordclass) -> bool;
}

impl Display for Attribute {
    fn fmt(&self, f : &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.name)
    }
}

impl Affect for Attribute {
    fn can_affect(&self, c : Wordclass) -> bool {
        self.affects.contains(&c)
    }
}

pub enum Word<Attr> {
    Noun(String,String,Vec<Attr>),
    Verb(String,String,Vec<Attr>),
    Modifier(String,String,Vec<Attr>),
    Particle(String,String,Vec<Attr>),
}

pub fn add_attr<Attr : Clone+Eq+Affect+Display>(w : Word<Attr>, a : Attr) -> Word<Attr> {
    match w {
        Word::Noun(root,gloss,mut xs) => {
            if a.can_affect(Wordclass::N) { xs.push(a); }
            Word::Noun(root,gloss,xs)
        },
        Word::Verb(root,gloss,mut xs) => {
            if a.can_affect(Wordclass::V) { xs.push(a); }
            Word::Verb(root,gloss,xs)
        },
        Word::Modifier(root,gloss,mut xs) => {
            if a.can_affect(Wordclass::M) { xs.push(a); }
            Word::Modifier(root,gloss,xs)
        },
        Word::Particle(root,gloss,mut xs) => {
            if a.can_affect(Wordclass::P) { xs.push(a); }
            Word::Particle(root,gloss,xs)
        },
    }
}

pub fn get_word<'a>(s : &'a String, json : &Value) -> Result<Word<Attribute>> {
    let root = String::from(s); let wordinfo = &json["vocab"][s.clone()];
    match wordinfo {
        Value::Null => {return Err(cuo(&format!("Word not found: «{}»!",&s.as_str())));}
        _ => ()
    }
    let gloss = String::from(Option::unwrap(wordinfo["gloss"].as_str()));
    match wordinfo["class"].as_str() {
        Some("N") => Ok(Word::Noun(root,gloss,Vec::new())),
        Some("V") => Ok(Word::Verb(root,gloss,Vec::new())),
        Some("M") => Ok(Word::Modifier(root,gloss,Vec::new())),
        Some("P") => Ok(Word::Particle(root,gloss,Vec::new())),
        _ => Err(cuo("Unrecognized word class!"))
    }
}

// TODO add functions for compouding, derivation, and metalang->conlang lookups

pub fn get_attr(name : String, json : &Value) -> Result<Attribute> {
    let attrinfo = &json["attributes"][name.clone()];
    match attrinfo {
        Value::Null => {return Ok(null_attr(name));}
        _ => ()
    }
    let form = String::from(Option::unwrap(attrinfo["form"].as_str()));
    let place = Option::unwrap(attrinfo["pos"].as_i64()).cmp(&0);
    let mut affects = HashSet::new();
    match &attrinfo["affects"] {
        Value::Array(cs) => {
            for c in cs {
                match c.as_str() {
                    Some("N") => { affects.insert(Wordclass::N); },
                    Some("V") => { affects.insert(Wordclass::V); },
                    Some("M") => { affects.insert(Wordclass::M); },
                    Some("P") => { affects.insert(Wordclass::P); },
                    _ => { return Err(cuo("Unrecognized word class!")); }
                }
            }
        }
        _ => {return Err(cuo("Malformed JSON!"));}
    }
    Ok(Attribute {name,form,place,affects})
}

pub fn null_attr(name : String) -> Attribute {
    let form = String::from("");
    let place = 0.cmp(&0);
    let mut affects = HashSet::new();
    affects.insert(Wordclass::N);
    affects.insert(Wordclass::V);
    affects.insert(Wordclass::M);
    affects.insert(Wordclass::P);
    Attribute {name,form,place,affects}
}

pub fn inflect(w : &Word<Attribute>) -> String {
    let (root,xs) = match &w {
        Word::Noun(root,_,xs) => (root,xs),
        Word::Verb(root,_,xs) => (root,xs),
        Word::Modifier(root,_,xs) => (root,xs),
        Word::Particle(root,_,xs) => (root,xs),
    };
    let mut out = String::from(root);
    for attr in xs {
        match attr.place {
            Ordering::Greater => out = out + "-" + &attr.form.clone(),
            Ordering::Less => out = attr.form.clone() + "-" + &out,
            Ordering::Equal => ()
        }
    }
    out
}

pub fn gloss(w : &Word<Attribute>) -> String {
    let (gloss,xs) = match &w {
        Word::Noun(_,gloss,xs) => (gloss,xs),
        Word::Verb(_,gloss,xs) => (gloss,xs),
        Word::Modifier(_,gloss,xs) => (gloss,xs),
        Word::Particle(_,gloss,xs) => (gloss,xs),
    };
    let mut out = String::from(gloss);
    for attr in xs {
        match attr.place {
            Ordering::Greater => out = out + "-" + &attr.name.clone(),
            Ordering::Less => out = attr.name.clone() + "-" + &out,
            Ordering::Equal => out = String::from("[") + &attr.name.clone() + "]" + &out,
        }
    }
    out
}
