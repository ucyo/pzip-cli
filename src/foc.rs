//! Reading file and performing a first ones count compression
//!
use super::graycodeanalysis::read_u32;

pub fn foc(matches: &clap::ArgMatches) {
    // println!("{:#?}", matches);
    let ifile = String::from(matches.value_of("input").unwrap());
    let mode = String::from(matches.value_of("mode").unwrap());
    let data = read_u32(&ifile);

    if mode == "diff" {
        process_diff(&data);
    } else if mode == "xor" {
        process_xor(&data);
    } else if mode == "power" {
        process_power(&data);
    } else {
        println!("Can not understand!");
    }
}

fn process_diff(data: &Vec<u32>) {
    unimplemented!()
}

fn process_xor(data: &Vec<u32>) {
    unimplemented!()
}

fn process_power(data: &Vec<u32>) {
    unimplemented!()
}
