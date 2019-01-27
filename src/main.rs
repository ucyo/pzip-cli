#![allow(dead_code)]
#![feature(duration_float)]

use pzip;
use clap::{App, load_yaml};
use std::time::Instant;
use std::fs::metadata;

mod graycodeanalysis;

fn main() {
    let yaml = load_yaml!("cli.yaml");
    let matches = App::from_yaml(yaml).get_matches();

    // compress(&matches);
    compress_with_information(&matches);
    // graycode(&matches);
}

fn lzcanalysis(matches: &clap::ArgMatches) -> u32 {
    let input = String::from(matches.value_of("output").unwrap());
    let data = graycodeanalysis::read_u32(&input);
    let lzc: u32 = data.iter().map(|x| x.leading_zeros()).sum();
    lzc
}

fn compress_with_information(matches: &clap::ArgMatches) {
    let start = Instant::now();
    compress(&matches);
    let duration = start.elapsed().as_float_secs();

    let fsize = metadata(matches.value_of("input").unwrap()).unwrap().len();
    let mbytes = fsize / 1024 / 1024;

    if matches.value_of("type").unwrap() == "f32" {
        let lzc = lzcanalysis(&matches);
        let of = fsize * 8;
        print!("LZC: {} ({:.3}%) ", lzc, (lzc as f64 / of as f64) * 100.0)

    }
    let throughput = mbytes as f64 / duration;
    println!("Throughput: {:.5} MiB/sec", throughput)
}



fn graycode(matches: &clap::ArgMatches) {

    let input = String::from(matches.value_of("input").unwrap());
    let analysis = graycodeanalysis::analyse_file(&input);
    println!("num, leading_zeros, ms_ones, ms_zeros, remaining");
    for val in analysis{
        println!("{}", val);
    }
}


fn compress(matches: &clap::ArgMatches) {
    let input = String::from(matches.value_of("input").unwrap());
    let output = String::from(matches.value_of("output").unwrap());
    let shape = parse_shape(&matches);
    let predictor = parse_predictor(&matches);

    if matches.value_of("type").unwrap() == "f32" {
        let setup = pzip::Setup::<f32>::new(&input, shape, predictor);
        setup.write::<pzip::mapping::Raw, pzip::mapping::ClassicGray, pzip::mapping::Untouched, pzip::mapping::Untouched>(&output);
    } else if matches.value_of("type").unwrap() == "f64" {
        let setup = pzip::Setup::<f64>::new(&input, shape, predictor);
        setup.write::<pzip::mapping::Raw, pzip::mapping::ClassicGray, pzip::mapping::Untouched>(&output);
    }
}

fn parse_shape(matches: &clap::ArgMatches) -> pzip::Shape {
    let shape: Vec<usize> = matches.values_of("shape")
           .unwrap()
           .map(|x| String::from(x).parse::<usize>().unwrap_or_else(|e| panic!("Shape: {}", e)))
           .collect();
    pzip::Shape{z:shape[0],y:shape[1],x:shape[2]}
}

fn parse_predictor(matches: &clap::ArgMatches) -> Vec<pzip::Weight> {
   match matches.value_of("predictor").unwrap() {
        "lv" => return pzip::traversal::predictors::get_lastvalue(),
        "lorenz" => return pzip::traversal::predictors::get_lorenz(),
        _ => panic!("Unknown predictor")
   }
}
