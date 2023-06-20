use rand::Rng;

fn random_number(min: u32, max: u32) -> u32 {
    let mut generator = rand::thread_rng();

    generator.gen_range(min..max)
}

pub fn generate_random_link() -> String {
    let number_of_letters = random_number(10, 15);
    let mut link = String::with_capacity(number_of_letters as usize);

    for _ in 0..number_of_letters {
        let char = match random_number(0, 2) {
            0 => char::from(random_number(65, 90) as u8),
            _ => char::from(random_number(97, 122) as u8),
        };
        link.push(char);
    }
    link
}

pub fn is_link_valid<T: AsRef<str>>(link: T) -> bool {
    for x in link.as_ref().chars() {
        if ! ((x >= 'A' && x <= 'Z') || (x >= 'a' && x <= 'z')) {
            return false;
        }
    }

    return true;
}

#[test]
fn gen_links_works() {
    let link = generate_random_link();
    assert!(
        link.len() >= 10 && link.len() <= 16,
        "Link between 10 and 15 characters long"
    );
}
