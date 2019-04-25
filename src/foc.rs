//! Reading file and performing a first ones count compression
//!
use super::graycodeanalysis::read_u32;

pub fn foc(matches: &clap::ArgMatches) {
    // println!("{:#?}", matches);
    let ifile = String::from(matches.value_of("input").unwrap());
    let data = read_u32(&ifile);
}
