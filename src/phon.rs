use serde_json::Value;
use regex::Regex;
//use std::borrow::Cow;

/**
 * BASIC FUNCTION:
 *  - Replace elements by index in category
 */
struct Category {
    id:String,
    seqs:Vec<String>,
}

struct SCRule {
    target:Regex,
    replacement:String,
    pos_env:(Regex,Regex),
    neg_env:(Regex,Regex),
}

impl std::fmt::Display for SCRule {
    fn fmt(&self, f : &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let neg = if self.neg_env.0.as_str().chars().count() == 0
                    && self.neg_env.1.as_str().chars().count() == 0
                {
                    String::new()
                } else {
                    format!(" /! {}_{}", &self.neg_env.0.as_str(), &self.neg_env.1.as_str())
                };
        write!(f, "`{}` → `{}` / {}_{}{}", &self.target, &self.replacement, &self.pos_env.0, &self.pos_env.1, neg)
    }
}

fn multigraph_to_unigraph<'a>(s : &'a str, multigraphs : &'a Vec<Regex>) -> String {
    let mut out = String::from(s);
    let mut unigraph = '\u{E000}'; // 1st character of unicode private use area
    // For now, we will naively assume there are fewer than 6400 elements in this list.
    // and that people won't be using custom fonts
    for mg in multigraphs {
        let mut seq = String::new();
        seq.push(unigraph);
        out = String::from(mg.replace_all(&out,&seq));
        unigraph = char::from_u32((unigraph as u32) + 1).unwrap();
    }
    out
}

fn unigraph_to_multigraph<'a>(s : &'a str, multigraphs : &'a Vec<Regex>) -> String {
    let mut out = String::from(s);
    let mut unigraph = '\u{E000}'; // 1st character of unicode private use area
    // For now, we will naively assume there are fewer than 6400 elements in this list.
    // and that people won't be using custom fonts
    for mg in multigraphs {
        let mut seq = String::new();
        seq.push(unigraph);
        let r = Regex::new(&seq).unwrap();
        out = String::from(r.replace_all(&out,mg.as_str()));
        unigraph = char::from_u32((unigraph as u32) + 1).unwrap();
    }
    out
}

fn from_cats(rule : SCRule, cats : &Vec<Category>, multigraphs : &Vec<Regex>, _verb : bool) -> SCRule {
    let mut rule_new = rule;
    for cat in cats {
        // replace each `@C` category abbreviation with the regular (c1|c2|c3|...) form
        let get = Regex::new(&cat.id).unwrap();
        let seqs = String::from("(") + &cat.seqs.join("|") + ")";
        let pos_l_repl = get.replace_all(rule_new.pos_env.0.as_str(),&seqs);
        rule_new.pos_env.0 = Regex::new(&pos_l_repl).unwrap();
        let pos_r_repl = get.replace_all(rule_new.pos_env.1.as_str(),&seqs);
        rule_new.pos_env.1 = Regex::new(&pos_r_repl).unwrap();
        let neg_l_repl = get.replace_all(rule_new.neg_env.0.as_str(),&seqs);
        rule_new.neg_env.0 = Regex::new(&neg_l_repl).unwrap();
        let neg_r_repl = get.replace_all(rule_new.neg_env.1.as_str(),&seqs);
        rule_new.neg_env.1 = Regex::new(&neg_r_repl).unwrap();
        let target_repl = get.replace_all(rule_new.target.as_str(),&seqs);
        rule_new.target = Regex::new(&target_repl).unwrap();
    }
    // replace each multigraph 
    let pos_l_repl = multigraph_to_unigraph(rule_new.pos_env.0.as_str(),multigraphs);
    rule_new.pos_env.0 = Regex::new(&pos_l_repl).unwrap();
    let pos_r_repl = multigraph_to_unigraph(rule_new.pos_env.1.as_str(),multigraphs);
    rule_new.pos_env.1 = Regex::new(&pos_r_repl).unwrap();
    let neg_l_repl = multigraph_to_unigraph(rule_new.neg_env.0.as_str(),multigraphs);
    rule_new.neg_env.0 = Regex::new(&neg_l_repl).unwrap();
    let neg_r_repl = multigraph_to_unigraph(rule_new.neg_env.1.as_str(),multigraphs);
    rule_new.neg_env.1 = Regex::new(&neg_r_repl).unwrap();
    let target_repl = multigraph_to_unigraph(rule_new.target.as_str(),multigraphs);
    rule_new.target = Regex::new(&target_repl).unwrap();
    let repstr_repl = multigraph_to_unigraph(&rule_new.replacement,multigraphs);
    rule_new.replacement = String::from(repstr_repl);
    // if _verb { println!("Multigraph «{}» mapped to '{}' U+{:x}",multigraph.as_str(),&seq,unigraph as u32); }
    rule_new
}

fn sca(token : &str, rule : SCRule, verb : bool) -> String {
    if verb {println!("Evaluating {}->{}/{}_{} on «{}»",rule.target.as_str(),&rule.replacement,rule.pos_env.0.as_str(),rule.pos_env.1.as_str(),&token);}
    let mut out = String::new();
    let mut no_mat = rule.target.split(&token);
    let mut prev = no_mat.next().unwrap();
    let mut full_prev = String::from(prev.clone());
    let full = String::from(token.clone());
    for mat in rule.target.find_iter(&token) {
        let mut update = false;
        for l_env in rule.pos_env.0.find_iter(&full_prev) {
            let l_algt =
                    if l_env.as_str().chars().count() > 0 {
                        l_env.end()
                    } else {
                        mat.start()
                    };
            let next = match &mat.as_str().as_bytes().len() + &full_prev.as_bytes().len() {
                    0 => full.clone(),
                    x => {
                        //println!("{}, |mat|={}",&full_prev,mat.as_str().chars().count());
                        if x<full.chars().count() {
                            String::from(full.split_at(x).1)
                        } else {
                            String::new()
                        }
                    },
                };
            let r_env_start = match rule.pos_env.1.find(&next) {
                        Some(r) => r.start(),
                        None => token.chars().count(),
                        // usize is non-negative, but it really just needs to be nonzero
                };
            if verb {println!("Test: {}#{}; r_env:{}",&full_prev,next, r_env_start);}
            if verb {println!("l={}, match={} @{}",rule.pos_env.0.as_str(),l_env.as_str(),l_algt);}
            if verb {println!("r={}, match={} @{}",rule.pos_env.1.as_str(),
                    match rule.pos_env.1.find(&next) {Some(m)=>m.as_str(),None=>("NONE"),},r_env_start);}
            if (mat.start() == l_algt) && (r_env_start == 0) {
                let replaced = rule.target.replace(mat.as_str(),&rule.replacement);
                let r_neg = match rule.neg_env.1.find(&next) {
                    Some(r) => r,
                    None => Regex::new(r"").unwrap().find("").unwrap()
                };
                // Go thru ALL possible negative environments, if NONE match,
                for l_neg in rule.neg_env.0.find_iter(&full_prev) {
                    if verb {println!("Match! neg={}_{}",l_neg.as_str(),r_neg.as_str())}
                    match (l_neg.as_str().chars().count(),r_neg.as_str().chars().count(),
                            mat.start() == l_neg.end()+out.chars().count(),
                            r_neg.start() == 0) {
                        (0,0,_,_) => {update=true;out = out + &prev + &replaced;break},
                        (0,_,_,true) => {update=true;out = out + &prev + &mat.as_str();break},
                        (_,0,true,_) => {update=true;out = out + &prev + &mat.as_str();break},
                        (_,_,true,true) => {update=true;out = out + &prev + &mat.as_str();break},
                        _ => (),
                    };
                }
                if update {
                    break;
                } else {
                    match (r_neg.as_str().chars().count(),
                            r_neg.start() == 0) {
                        (0,_) => {update=true;out = out + &prev + &replaced;break},
                        (_,true) => {update=true;out = out + &prev + &mat.as_str();break},
                        _ => {update=true;out = out + &prev + &replaced;break},
                    };
                }
            }
        }
        if !update {
            out = out + &prev + &mat.as_str();
        }
        prev = no_mat.next().unwrap();
        full_prev = full_prev + &mat.as_str() + &prev;
    }
    out + &prev
}

pub fn to_orthography(token : String, sc : &Value, cats : &Value, multigraphs : &Value, verbose : bool) -> String {
    let rules =
        match sc {
            Value::Array(rs) => rs.iter().map(|s| s.as_str()).collect(),
            _ => Vec::<_>::new(),
        };
    let mg_rep =
        match multigraphs {
            Value::Array(ms) => ms.iter().map(|mg| Regex::new(mg.as_str().unwrap()).unwrap()).collect(),
            _ => Vec::<_>::new(),
        };
    let mut cat_vec : Vec<Category> = Vec::new();
    match cats {
        Value::Object(cs) => {
            for k in cs.keys() {
                let id = String::from(k);
                let mut seqs =
                    match &cs[k] {
                        Value::Array(toks) => toks.iter().map(
                                    |s| String::from(s.as_str().unwrap())
                                    ).collect(),
                        _ => Vec::<_>::new(),
                    };
                seqs.sort_by(|s1,s2| s2.len().cmp(&s1.len()));
                cat_vec.push(Category { id, seqs });
            }
        },
        _ => (),
    }
    // Run forward multigraph replacements 
    let mut s0 = multigraph_to_unigraph(token.as_str(),&mg_rep);

    // SC rules are generally of the form x->y/L_R(/NL_NR)
    // which can be read as ``x becomes y between L and R (except between NL and NR)''
    // all of these are regular expressions except y, which is just a String
    for rule_str in rules {
        let mut rule_coll = rule_str.unwrap().split("/");
        let change : Vec<&str> = rule_coll.next().unwrap().split("->").collect();
        let pos : Vec<&str> = rule_coll.next().unwrap().split("_").collect();
        let neg : Vec<&str> =
            match rule_coll.next() {
                Some(env) => env.split("_").collect(),
                None => vec!["",""],
            };
        assert_eq!(neg.len(),2);
        let rule0 = SCRule {
            target: Regex::new(change[0]).unwrap(),
            replacement: String::from(change[1]),
            pos_env: (Regex::new(pos[0]).unwrap(),Regex::new(pos[1]).unwrap()),
            neg_env: (Regex::new(neg[0]).unwrap(),Regex::new(neg[1]).unwrap()),
        };
        // TODO split multigraph parser into separate function
        // running it separately for every single rule is comically inefficient...
        s0 = sca(&s0,from_cats(rule0,&cat_vec,&mg_rep,verbose),verbose);
    }
    unigraph_to_multigraph(&s0,&mg_rep)
}
