use std::{cmp::Ordering,io::Result,io::Error,collections::HashSet,collections::HashMap,fmt::Display};
use serde_json::Value;
use regex::Regex;

// static mut lookups : Option<HashMap<String,&String>> = None;

fn cuo(s : &str) -> Error {
    Error::new(std::io::ErrorKind::Other,s)
}

#[derive(Hash,PartialEq,Eq,Clone)]
pub enum DefaultWordclass { N, V, M, P, }

#[derive(Hash,PartialEq,Eq,Clone)]
pub enum Wordclass {
    Default(DefaultWordclass),
    Custom(u32), // this should probably be more than fine for now
}

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

#[derive(Hash,PartialEq,Eq,Clone)]
pub struct Word<Attr> {
    lemma : String,
    gloss : String,
    class : Wordclass,
    subclass : (), // idk might implement later
    attributes : Vec<Attr>
}

// Both of these are the same type, but one is for native words, the other is for translation/metalang lookups
pub struct LookupTable<'a> {
    lang : Option<HashMap<String,&'a Word<Attribute>>>,
    txln : Option<HashMap<String,&'a Word<Attribute>>>,
}

pub fn add_attr<Attr : Clone+Eq+Affect+Display>(w : Word<Attr>, a : Attr) -> Word<Attr> {
    let Word {lemma, gloss, class, subclass, mut attributes} = w;
    if a.can_affect(class.clone()) { attributes.push(a); }
    Word {lemma, gloss, class, subclass, attributes}
}

pub fn get_word<'a>(s : &'a String, json : &Value) -> Result<Word<Attribute>> {
    let lemma = String::from(s);
    let wordinfo = &json["vocab"][s.clone()];
    match wordinfo {
        Value::Null => {
            /*
            // TODO
            let lookup = Regex::new(r"^\{(.*)\}$").unwrap();
            return match lookup.captures(&s.as_str()) {
                None => Err(cuo(&format!("Word not found: «{}»!",&s.as_str()))),
                Some(m) => {
                    let res = _naive_lookup(String::from(&m[1]),&json)?;
                    _get_word_(res, json)
                }
            }
            */
            ()
        }
        _ => ()
    }
    let gloss = String::from(Option::unwrap(wordinfo["gloss"].as_str()));
    let class = match wordinfo["class"].as_str() {
        Some("N") => Ok(Wordclass::Default(DefaultWordclass::N)),
        Some("V") => Ok(Wordclass::Default(DefaultWordclass::V)),
        Some("M") => Ok(Wordclass::Default(DefaultWordclass::M)),
        Some("P") => Ok(Wordclass::Default(DefaultWordclass::P)),
        _ => Err(cuo("Unrecognized word class!")) // TODO check for custom class definitions
    }?;
    Ok(Word {lemma, gloss, class, subclass : (), attributes : Vec::new()})
}

// TODO add functions for compouding, derivation, and metalang->conlang lookups
fn _naive_lookup(term : String, json : &Value) -> Result<&String> {
    let mut table = HashMap::new();
    let entry_iter = match &json["vocab"] {
        Value::Object(o) => o.iter(),
        _ => return Err(cuo("Unparseable JSON!")),
    };
    for (word,entry) in entry_iter {
        let gloss = String::from(Option::unwrap(entry["gloss"].as_str()));
        assert!(!table.contains_key(&gloss));
        table.insert(gloss,word);
    }
    //lookups = Some(table);
    if !table.contains_key(&term) { return Err(cuo("nope")); }
    let translation = table.get(&term).unwrap();
    Ok(translation)
    /*
    match lookups {
        None => {
            let table = HashMap::new();
            let entry_iter = match json["vocab"] {
                Value::Object(o) => o.iter(),
                _ => return Err(cuo("Unparseable JSON!")),
            };
            for (word,entry) in entry_iter {
                let gloss = String::from(Option::unwrap(entry["gloss"].as_str()));
                assert!(!table.contains_key(&gloss));
                table.insert(gloss,word);
            }
            lookups = Some(table);
            Ok(table.get(&term).unwrap())
        },
        Some(table) => Ok(table.get(&term).unwrap()),
    }
    */
}

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
                    Some("N") => { affects.insert(Wordclass::Default(DefaultWordclass::N)); },
                    Some("V") => { affects.insert(Wordclass::Default(DefaultWordclass::V)); },
                    Some("M") => { affects.insert(Wordclass::Default(DefaultWordclass::M)); },
                    Some("P") => { affects.insert(Wordclass::Default(DefaultWordclass::P)); },
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
    affects.insert(Wordclass::Default(DefaultWordclass::N));
    affects.insert(Wordclass::Default(DefaultWordclass::V));
    affects.insert(Wordclass::Default(DefaultWordclass::M));
    affects.insert(Wordclass::Default(DefaultWordclass::P));
    Attribute {name,form,place,affects}
}

pub fn inflect(w : &Word<Attribute>) -> String {
    let (root,xs) = (&(w.lemma), &(w.attributes));
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
    let (gloss,xs) = (&(w.gloss), &(w.attributes));
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
