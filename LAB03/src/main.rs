const U32_MAX: u32 = u32::MAX;

fn is_prime(n: u16) -> bool {
    if n < 2 {
        return false;
    }
    for i in 2..=((n as f64).sqrt() as u16) {
        if n % i == 0 {
            return false;
        }
    }
    true
}

fn next_prime(x: u16) -> Option<u16> {
    let mut n = x + 1;
    while n <= u16::MAX {
        if is_prime(n) {
            return Some(n);
        }
        n += 1;
    }
    None
}

fn ex1()
{
    let mut prime = Some(2);
    
    while let Some(p) = prime {
        println!("{}", p);
        prime = next_prime(p);
        if prime.is_none() {
            break;
        }
    }
}

fn checked_add_u32(a: u32, b: u32) -> u32 {
    if a > U32_MAX - b {
        panic!("Addition overflowed!");
    }
    a + b
}

fn checked_mul_u32(a: u32, b: u32) -> u32 { 
    if a != 0 && b > U32_MAX / a {
        panic!("Multiplication overflowed!");
    }
    a * b
}

fn ex2()
{
    let a: u32 = 1_000;
    let b: u32 = 2_000;
    let c: u32 = u32::MAX;

    // adunare care merge
    println!("{} + {} = {}", a, b, checked_add_u32(a, b));

    // adunare care da overflow
    // println!("{} + {} = {}", a, c, checked_add_u32(a, c));

    // inmultire care merge
    println!("{} * {} = {}", a, b, checked_mul_u32(a, b));

    // inmultire care da overflow
    // println!("{} * {} = {}", a, c, checked_mul_u32(a, c));

}

fn ex3()
{
    let a: u32 = 1_000;
    let b: u32 = 2_000;
    let c: u32 = u32::MAX;

    match try_operations(a, b, c) {
        Ok(()) => println!("Operations completed successfully."),
        Err(e) => println!("An error occurred: {}", e),
    }
}

use std::fmt;

#[derive(Debug)]
enum MyError {
    AdditionOverflow,
    MultiplicationOverflow,
}

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MyError::AdditionOverflow => write!(f, "Addition overflow occurred!"),
            MyError::MultiplicationOverflow => write!(f, "Multiplication overflow occurred!"),
        }
    }
}

fn checked_add_u32_2(a: u32, b: u32) -> Result<u32, MyError> {
    if a > u32::MAX - b {
        Err(MyError::AdditionOverflow)
    } else {
        Ok(a + b)
    }
}

fn checked_mul_u32_2(a: u32, b: u32) -> Result<u32, MyError> {
    if a != 0 && b > u32::MAX / a {
        Err(MyError::MultiplicationOverflow)
    } else {
        Ok(a * b)
    }
}


fn try_operations(a: u32, b: u32, c: u32) -> Result<(), MyError> {
    
    let add_result = checked_add_u32_2(a, b)?;
    println!("{} + {} = {}", a, b, add_result);

    
    let mul_result = checked_mul_u32_2(a, c)?;
    println!("{} * {} = {}", a, c, mul_result);

    Ok(())
}

fn ex4()
{
    let c = 'A';
    match to_uppercase(c) {
        Ok(uc) => println!("Uppercase: {}", uc),
        Err(e) => print_error(e),
    }

    let d = '9';
    match char_to_number(d) {
        Ok(num) => println!("Number: {}", num),
        Err(e) => print_error(e),
    }

    let h = 'G';
    match char_to_number_hex(h) {
        Ok(num) => println!("Hex number: {}", num),
        Err(e) => print_error(e),
    }

    let non_printable = '\x07';
    match print_char(non_printable) {
        Ok(p) => println!("Printable: {}", p),
        Err(e) => print_error(e),
    }   
}

#[derive(Debug)]
enum CharError {
    NotAscii,
    NotDigit,
    NotBase16Digit,
    NotLetter,
    NotPrintable,
}

impl fmt::Display for CharError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CharError::NotAscii => write!(f, "Character is not ASCII."),
            CharError::NotDigit => write!(f, "Character is not a digit."),
            CharError::NotBase16Digit => write!(f, "Character is not a base16 digit."),
            CharError::NotLetter => write!(f, "Character is not a letter."),
            CharError::NotPrintable => write!(f, "Character is not printable."),
        }
    }
}

fn to_uppercase(c: char) -> Result<char, CharError> {
    if !c.is_ascii_alphabetic() {
        Err(CharError::NotLetter)
    } else {
        Ok(c.to_ascii_uppercase())
    }
}

fn to_lowercase(c: char) -> Result<char, CharError> {
    if !c.is_ascii_alphabetic() {
        Err(CharError::NotLetter)
    } else {
        Ok(c.to_ascii_lowercase())
    }
}

fn print_char(c: char) -> Result<char, CharError> {
    if c.is_ascii_graphic() || c.is_ascii_whitespace() {
        Ok(c)
    } else {
        Err(CharError::NotPrintable)
    }
}

fn char_to_number(c: char) -> Result<u32, CharError> {
    if !c.is_ascii() {
        Err(CharError::NotAscii)
    } else if !c.is_ascii_digit() {
        Err(CharError::NotDigit)
    } else {
        Ok(c.to_digit(10).unwrap())
    }
}

fn char_to_number_hex(c: char) -> Result<u32, CharError> {
    if !c.is_ascii() {
        Err(CharError::NotAscii)
    } else if !c.is_ascii_hexdigit() {
        Err(CharError::NotBase16Digit)
    } else {
        Ok(c.to_digit(16).unwrap())
    }
}

fn print_error(e: CharError) {
    println!("{}", e);
}
fn ex5(){
    let numbers = vec![121, -121, 123, 1221, 10];
    
    for &n in &numbers {
        match is_palindrome(n) {
            Some(true) => println!("{} is a palindrome.", n),
            Some(false) => println!("{} is not a palindrome.", n),
            None => println!("{} is not a valid input for palindrome check.", n),
        }
    }
}

fn is_palindrome(n: i32) -> Option<bool> {  
    if n < 0 {
        return None;
    }

    let num_str = n.to_string();
    
    let reversed_str: String = num_str.chars().rev().collect();
    
    Some(num_str == reversed_str)
}

fn main() {
    let ok:u32 = 5;
    if ok == 1 {
        ex1();
    }
    else if ok == 2{
        ex2();
    }
    else if ok == 3{
        ex3();
    }
    else if ok == 4{
        ex4();
    }
    else if ok == 5{
        ex5();
    }
}
