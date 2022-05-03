use serde_json::Value;
use regex::Regex;

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

fn from_cats(rule : SCRule, cats : &Vec<Category>) -> SCRule {
    let mut rule_new = rule;
    for cat in cats {
        let get = Regex::new(&cat.id).unwrap();
        let seqs = String::from("(") + &cat.seqs.join("|") + ")";
        /*
        let seqs_labeled = String::from("(?P<") + &cat.id.split_at(1).1 + ">"
                        + &cat.seqs.join("|") + ")";
                        */
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

pub fn to_orthography(token : String, sc : &Value, cats : &Value, verbose : bool) -> String {
    let mut s0 = String::from(&token);
    let rules =
        match sc {
            Value::Array(rs) => rs.iter().map(|s| s.as_str()).collect(),
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
        s0 = sca(&s0,from_cats(rule0,&cat_vec),verbose);
    }
    s0
}
