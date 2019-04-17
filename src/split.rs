use super::graycodeanalysis::read_u32;

pub fn split(matches: &clap::ArgMatches) {
    let input = String::from(matches.value_of("input").unwrap());
    let _data = read_u32(&input);

    println!("Jean-Claude")
}
