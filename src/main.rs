#![allow(dead_code)]
#![feature(duration_float)]

use pzip;
use clap::{App, load_yaml};
use std::time::Instant;
use std::fs::metadata;
use env_logger;

mod graycodeanalysis;
mod mqanalysis;

fn main() {
    env_logger::init();
    let yaml = load_yaml!("cli.yaml");
    let matches = App::from_yaml(yaml).get_matches();

    if matches.is_present("compress") {
        compress_with_information(&matches.subcommand_matches("compress").unwrap());
    } else if matches.is_present("analysis") {
        graycode(&matches.subcommand_matches("analysis").unwrap());
    } else if matches.is_present("mqanalysis"){
        mqanalysis::mqanalysis(&matches.subcommand_matches("mqanalysis").unwrap());
    } else {
        App::from_yaml(yaml).print_help().unwrap();
    }
}

fn lzcanalysis(matches: &clap::ArgMatches) -> (u32, f64) {
    let input = String::from(matches.value_of("output").unwrap());
    let data = graycodeanalysis::read_u32(&input);
    let lzc: u32 = data.iter().map(|x| x.leading_zeros()).sum();
    (lzc, lzc as f64 / data.len() as f64)
}

fn compress_with_information(matches: &clap::ArgMatches) {
    let start = Instant::now();
    compress(&matches);
    let duration = start.elapsed().as_float_secs();

    let fsize = metadata(matches.value_of("input").unwrap()).unwrap().len();
    let mbytes = fsize / 1024 / 1024;

    if matches.value_of("type").unwrap() == "f32" {
        let (lzc,mean_lzc) = lzcanalysis(&matches);
        let of = fsize * 8;
        print!("LZC: {} ({:.15}% | {:.3}) ", lzc, (lzc as f64 / of as f64) * 100.0, mean_lzc)

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

use pzip::transform::{Inter, Intra, Byte, Compact};
use pzip::predictors::Ignorant;
fn compress(matches: &clap::ArgMatches) {
    let input = String::from(matches.value_of("input").unwrap());
    let output = String::from(matches.value_of("output").unwrap());
    let shape = parse_shape(&matches);
    let predictor = parse_predictor(&matches);
    let ring = parse_ring(&matches);
    let cut = parse_cut(&matches);
    let parts = parse_parts(&matches);

    let alg_inter = parse_inter_algorithm(&matches);
    let alg_intra = parse_intra_algorithm(&matches);
    let alg_byte = parse_byte_algorithm(&matches);
    let alg_compact = parse_compact_algorithm(&matches);
    let alg_correct = parse_correction_algorithm(&matches);
    let alg_residual = parse_residual_algorithm(&matches);

    if matches.value_of("type").unwrap() == "f32" {
        let mut setup = pzip::Setup::<f32>::new(&input, shape, predictor);
        setup.write(alg_inter, alg_intra, alg_byte, alg_compact, alg_residual, alg_correct, ring, cut, parts, &output);
    } else {
        panic!("Support for f64 deactivated!")
    }
    // TODO: Support for f64 deactivated!
    // } else if matches.value_of("type").unwrap() == "f64" {
    //     let setup = pzip::Setup::<f64>::new(&input, shape, predictor);
    //     setup.write(alg_inter, alg_intra, alg_byte, &output);
    // }
}

fn parse_shape(matches: &clap::ArgMatches) -> pzip::position::Position {
    let shape: Vec<i32> = matches.values_of("shape")
           .unwrap()
           .map(|x| String::from(x).parse::<i32>().unwrap_or_else(|e| panic!("Shape: {}", e)))
           .collect();
    pzip::position::Position{z:shape[0],y:shape[1],x:shape[2]}
}

fn parse_predictor(matches: &clap::ArgMatches) -> Ignorant<f32> {
   match matches.value_of("predictor").unwrap() {
        "lv" => return pzip::predictors::predictors::get_last_value_f32(),
        "lorenz" => return pzip::predictors::predictors::get_lorenz_f32(),
        _ => panic!("Unknown predictor")
   }
}

fn parse_inter_algorithm(matches: &clap::ArgMatches) -> Inter {
   match matches.value_of("intermapping").unwrap() {
        "untouched" => Inter::Untouched,
        "u" => Inter::Untouched,
        "ordered" => Inter::Ordered,
        "o" => Inter::Ordered,
        _ => panic!("Unknown inter mapping algorithm")
   }
}

fn parse_intra_algorithm(matches: &clap::ArgMatches) -> Intra {
   match matches.value_of("intramapping").unwrap() {
        "untouched" => Intra::Untouched,
        "u" => Intra::Untouched,
        "gray" => Intra::Gray,
        "g" => Intra::Gray,
        _ => panic!("Unknown intra mapping algorithm")
   }
}

fn parse_byte_algorithm(matches: &clap::ArgMatches) -> Byte {
   match matches.value_of("bytemapping").unwrap() {
        "untouched" => Byte::Untouched,
        "u" => Byte::Untouched,
        "monogray" => Byte::MonoGray,
        "mg" => Byte::MonoGray,
        _ => panic!("Unknown byte mapping algorithm")
   }
}

fn parse_compact_algorithm(matches: &clap::ArgMatches) -> Compact {
   match matches.value_of("compact").unwrap() {
        "untouched" => Compact::Untouched,
        "u" => Compact::Untouched,
        "nolzc" => Compact::NoLZC,
        _ => panic!("Unknown compact algorithm")
   }
}

use pzip::residual::ResidualCalculation;
fn parse_residual_algorithm(matches: &clap::ArgMatches) -> ResidualCalculation {
    match matches.value_of("residual").unwrap() {
        "xor" => ResidualCalculation::ExclusiveOR,
        "shifted" if matches.value_of("intramapping").unwrap() == "gray" => ResidualCalculation::ShiftedGray,
        "s" if matches.value_of("intramapping").unwrap() == "gray" => ResidualCalculation::ShiftedGray,
        "shifted" if matches.value_of("intramapping").unwrap() == "g" => ResidualCalculation::ShiftedGray,
        "s" if matches.value_of("intramapping").unwrap() == "g" => ResidualCalculation::ShiftedGray,
        "shifted" => ResidualCalculation::Shifted,
        "s" => ResidualCalculation::Shifted,
        "shiftedlzc" => ResidualCalculation::ShiftedLZC,
        "slzc" => ResidualCalculation::ShiftedLZC,
        _ => panic!("Unknown residual algorithm.")
    }
}

use pzip::correction::Correction;
fn parse_correction_algorithm(matches: &clap::ArgMatches) -> Correction {
    match matches.value_of("correction").unwrap() {
        "preverror" => Correction::PreviousError,
        "perr" => Correction::PreviousError,
        "delta2power2" => Correction::DeltaToPowerOf2,
        "d2p2" => Correction::DeltaToPowerOf2,
        "untouched" => Correction::Untouched,
        "u" => Correction::Untouched,
        _ => panic!("Unknown correction algorithm")
    }
}

fn parse_ring(matches: &clap::ArgMatches) -> bool {
    matches.is_present("ring")
}

fn parse_cut(matches: &clap::ArgMatches) -> u32 {
    let result: u32 = matches.value_of("cut").unwrap().parse().unwrap();
    result
}

fn parse_parts(matches: &clap::ArgMatches) -> u32 {
    let result: u32 = matches.value_of("parts").unwrap().parse().unwrap();
    result
}
