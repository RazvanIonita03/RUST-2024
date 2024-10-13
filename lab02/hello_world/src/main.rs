fn add_chars_n(mut s: String, c: char, mut i: u32) -> String {
    while i != 0 {
        s.push(c);
        i = i - 1;
    }
    return s;
}
fn add_chars_n2(s: &mut String, c: char, mut i: u32) {
    while i != 0 {
        s.push(c);
        i = i - 1;
    }
}
fn add_space(mut s: String, mut i: u32) -> String {
    while i != 0 {
        s += " ";
        i = i - 1;
    }
    return s;
}
fn add_str(mut s: String, st: &String) -> String {
    s += st;
    return s;
}
fn add_integer(mut s: String, mut x: u32) -> String {
    let mut cifre = Vec::new();
    if x == 0 {
        s.push('0');
        return s;
    }

    while x != 0 {
        let ch = (x % 10) as u8;
        cifre.push(ch);
        x /= 10;
    }
    while let Some(ch) = cifre.pop() {
        s.push((ch + b'0') as char);
    }

    s
}

fn add_float(mut s: String, x: f32) -> String {
    let integer_part = x as u32;
    let mut fractional_part = x - (integer_part as f32);

    let mut int_digits = Vec::new();
    let mut temp_int = integer_part;

    if temp_int == 0 {
        s.push('0');
    } else {
        while temp_int > 0 {
            int_digits.push(temp_int % 10);
            temp_int /= 10;
        }

        while let Some(ch) = int_digits.pop() {
            s.push((ch as u8 + b'0') as char);
        }
    }
    s.push('.');

    fractional_part *= 1000.0;

    let mut frac_as_int = fractional_part as u32;

    let mut frac_digits = Vec::new();
    for _ in 0..3 {
        frac_digits.push(frac_as_int % 10);
        frac_as_int /= 10;
    }
    while let Some(ch) = frac_digits.pop() {
        s.push((ch as u8 + b'0') as char);
    }

    s
}

fn main() {
    //Problema 1 : ok = 1
    //Problema 2 : ok = 2
    //Problema 3 : altceva
    let ok = 2;
    if ok == 1 {
        print!("\nPROBLEMA1\n");
        let mut s = String::from("");
        let mut i = 0;
        while i < 26 {
            let c = (i as u8 + 'a' as u8) as char;
            s = add_chars_n(s, c, 26 - i);

            i += 1;
        }

        print!("{}\n", s);
    } else if ok == 2 {
        print!("\nPROBLEMA2\n");
        let mut s = String::from("");
        let ref_to_s: &mut String = &mut s;
        let mut i = 0;
        while i < 26 {
            let c = (i as u8 + 'a' as u8) as char;
            add_chars_n2(ref_to_s, c, 26 - i);

            i += 1;
        }
        print!("{}\n", s);
    } else {
        print!("\nPROBLEMA3\n");
        let mut s = String::new();
        let mut t = String::from("I");

        s = add_space(s, 40);
        s = add_str(s, &t);
        s = add_space(s, 1);
        t = String::from("💚\n");
        s = add_str(s, &t);
        s = add_space(s, 40);
        t = String::from("Rust");
        s = add_str(s, &t);
        s = add_space(s, 1);
        t = String::from(".\n\n");
        s = add_str(s, &t);
        s = add_space(s, 4);
        t = String::from("Most");
        s = add_str(s, &t);
        s = add_space(s, 12);
        t = String::from("crate");
        s = add_str(s, &t);
        s = add_space(s, 6);
        s = add_integer(s, 306437968);
        s = add_space(s, 12);
        t = String::from("and");
        s = add_str(s, &t);
        s = add_space(s, 5);
        t = String::from("latest");
        s = add_str(s, &t);
        s = add_space(s, 10);
        t = String::from("is\n");
        s = add_str(s, &t);
        s = add_space(s, 9);
        t = String::from("downloaded");
        s = add_str(s, &t);
        s = add_space(s, 8);
        t = String::from("has");
        s = add_str(s, &t);
        s = add_space(s, 13);
        t = String::from("downloaded");
        s = add_str(s, &t);
        s = add_space(s, 5);
        t = String::from("the");
        s = add_str(s, &t);
        s = add_space(s, 9);
        t = String::from("version");
        s = add_str(s, &t);
        s = add_space(s, 4);
        s = add_float(s, 2.038);
        
        println!("{}", s);
    }
}
