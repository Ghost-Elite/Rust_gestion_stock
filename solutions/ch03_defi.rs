// Solution du défi du chapitre 3.
fn main() {
    // FizzBuzz : match sur un tuple de deux restes
    for i in 1..=20 {
        match (i % 3, i % 5) {
            (0, 0) => println!("FizzBuzz"),
            (0, _) => println!("Fizz"),
            (_, 0) => println!("Buzz"),
            _ => println!("{i}"),
        }
    }

    // Bonus : loop qui renvoie une valeur
    let mut n = 50;
    let premier_multiple = loop {
        n += 1;
        if n % 7 == 0 {
            break n;
        }
    };
    println!("premier multiple de 7 après 50 : {premier_multiple}");
}
