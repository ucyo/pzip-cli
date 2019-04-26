//! Reading file and performing a first ones count compression
//!
use log::{debug, info};
use std::fs::metadata;
use super::graycodeanalysis::read_u32;
use bit_vec::BitVec;
use pzip_huffman::hufbites::encode_itself_to_bytes as encode;
use super::graycodeanalysis::{get_value_first, get_residual_size_after};

pub fn foc(matches: &clap::ArgMatches) {
    // println!("{:#?}", matches);
    let ifile = String::from(matches.value_of("input").unwrap());
    let mode = String::from(matches.value_of("mode").unwrap());
    let data = read_u32(&ifile);
    info!("Encode file: {} with mode: {}", ifile, mode);

    let mut outbytes = 0usize;
    if mode == "diff" {
        let n = String::from(matches.value_of("cut").unwrap()).parse().unwrap();
        outbytes = process_diff(&data, n);
    } else if mode == "xor" {
        outbytes = process_xor(&data);
    } else if mode == "power" {
        outbytes = process_power(&data);
    } else {
        println!("Can not understand!");
    }

    let fsize = metadata(ifile).unwrap().len();
    println!("outbytes={} ratio={:.2}", outbytes, fsize as f64/outbytes as f64);
}

// Implementation of
// huff(lzc) + raw(sign) + huff(6-residual) + raw(residual)
fn process_diff(data: &Vec<u32>, n: u32) -> usize {
    info!("Cut: {}", n);
    let mut signs = BitVec::new();
    for _ in 0..data.len() {
        signs.push(true);
    }
    let signs = signs.to_bytes();  // raw(sign)
    debug!("Size: Raw(Sign)={}", signs.len());

    let lzc = data.iter().map(|&x| x.leading_zeros() as u8).collect::<Vec<u8>>();
    let (lzc, lzc_codebook) = encode(&lzc);  // huff(lzc)
    debug!("Size: Huff(LZC)={}", lzc.len());

    let first6 = data.iter().map(|&x| get_value_first(&x, n) as u8).collect::<Vec<u8>>();
    let (first6, first6_codebook) = encode(&first6);  // huff(6-residual)
    debug!("Size: Huff(6-residual)={}", first6.len());

    let leftresidual = data.iter().map(|&x| get_residual_size_after(&x, n)).sum::<u32>();
    let mut _bv = BitVec::new();
    for _ in 0..leftresidual {
        _bv.push(true);
    }
    let leftresidual = _bv.to_bytes();  // raw(residual - 6)
    debug!("Size: Raw(residual-6)={}", leftresidual.len());

    return signs.len() + lzc.len() + first6.len() + leftresidual.len()
}

// Implementation of
// huff(lzc) + huff(foc) + raw(residual - first 0)
fn process_xor(data: &Vec<u32>) -> usize{
    unimplemented!()
}

// Implementation of
// huff(lzc +- power(residual)) + raw(residual)
fn process_power(data: &Vec<u32>) -> usize {
    unimplemented!()
}
