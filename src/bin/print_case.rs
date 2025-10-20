use jsonrepair::{Options, repair_to_string};

fn main() {
    let mut s = String::from("{");
    s.push_str("k: [");
    for i in 0..50usize {
        if i > 0 {
            s.push_str("\n\n   \t \n");
        }
        s.push_str(&format!("{{i:{}}}", i));
    }
    s.push_str("]}");
    let out = repair_to_string(&s, &Options::default()).unwrap();
    println!("OUT:{}", out);
}

