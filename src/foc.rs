//! Reading file and performing a first ones count compression
//!
use super::graycodeanalysis::read_u32;
use super::graycodeanalysis::{get_residual_size_after, get_value_first};
use bit_vec::BitVec;
use log::{debug, info};
use pzip_huffman::hufbites::encode_itself_to_bytes as encode;
use std::collections::HashMap;
use std::fs::metadata;

struct FileContainer {
    huff_lzc: Vec<u8>,
    raw_sign: Vec<u8>,
    huff_6re: Vec<u8>,
    raw_res6: Vec<u8>,
    huff_lzc_codebook: HashMap<u8, BitVec>,
    huff_huff_6re_codebook: HashMap<u8, BitVec>,
}

impl FileContainer {
    pub fn new(
        huff_lzc: Vec<u8>,
        raw_sign: Vec<u8>,
        huff_6re: Vec<u8>,
        raw_res6: Vec<u8>,
        huff_lzc_codebook: HashMap<u8, BitVec>,
        huff_huff_6re_codebook: HashMap<u8, BitVec>,
    ) -> Self {
        FileContainer {
            huff_lzc,
            raw_sign,
            huff_6re,
            raw_res6,
            huff_lzc_codebook,
            huff_huff_6re_codebook,
        }
    }
    pub fn nbytes(&self) -> usize {
        self.huff_lzc.len() + self.raw_sign.len() + self.huff_6re.len() + self.raw_res6.len()
    }
}

impl std::fmt::Display for FileContainer {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "outbyte={}", self.nbytes())
    }
}

pub fn foc(matches: &clap::ArgMatches) {
    // println!("{:#?}", matches);
    let ifile = String::from(matches.value_of("input").unwrap());
    let mode = String::from(matches.value_of("mode").unwrap());
    let data = read_u32(&ifile);
    info!("Encode file: {} with mode: {}", ifile, mode);

    let fc : FileContainer;
    if mode == "diff" {
        let n = String::from(matches.value_of("cut").unwrap())
            .parse()
            .unwrap();
        fc = process_diff(&data, n);
    } else if mode == "xor" {
        fc = process_xor(&data);
    } else {
        // mode == "power"
        fc = process_power(&data);
    }

    let fsize = metadata(ifile).unwrap().len();
    println!(
        "outbytes={} ratio={:.2}",
        fc.nbytes(),
        fsize as f64 / fc.nbytes() as f64
    );
}

// Implementation of
// huff(lzc) + raw(sign) + huff(6-residual) + raw(residual)
fn process_diff(data: &Vec<u32>, n: u32) -> FileContainer {
    info!("Cut: {}", n);
    let mut signs = BitVec::new();
    for _ in 0..data.len() {
        signs.push(true);
    }
    let signs = signs.to_bytes(); // raw(sign)
    debug!("Size: Raw(Sign)={}", signs.len());

    let lzc = data
        .iter()
        .map(|&x| x.leading_zeros() as u8)
        .collect::<Vec<u8>>();
    let (lzc, lzc_codebook) = encode(&lzc); // huff(lzc)
    debug!("Size: Huff(LZC)={}", lzc.len());

    let first6 = data
        .iter()
        .map(|&x| get_value_first(&x, n) as u8)
        .collect::<Vec<u8>>();
    let (first6, first6_codebook) = encode(&first6); // huff(6-residual)
    debug!("Size: Huff(6-residual)={}", first6.len());

    let leftresidual = data
        .iter()
        .map(|&x| get_residual_size_after(&x, n))
        .sum::<u32>();
    let mut _bv = BitVec::new();
    for _ in 0..leftresidual {
        _bv.push(true);
    }
    let leftresidual = _bv.to_bytes(); // raw(residual - 6)
    debug!("Size: Raw(residual-6)={}", leftresidual.len());

    FileContainer::new(lzc, signs, first6, leftresidual, lzc_codebook, first6_codebook)

}

// Implementation of
// huff(lzc) + huff(foc) + raw(residual - first 0)
fn process_xor(data: &Vec<u32>) -> FileContainer {
    unimplemented!()
}

// Implementation of
// huff(lzc +- power(residual)) + raw(residual)
fn process_power(data: &Vec<u32>) -> FileContainer {
    unimplemented!()
}
