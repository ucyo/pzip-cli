//! Reading file and performing a first ones count compression
//!
use super::graycodeanalysis::read_u32;
use super::graycodeanalysis::{get_value_after, get_value_first};
use super::split::eliminate_first_bit;
use bit_vec::BitVec;
use log::{debug, info};
use pzip_huffman::hufbites::{encode_itself_to_bytes as encode, decode};
use std::collections::HashMap;
use std::fs::metadata;
use byteorder::{BigEndian, ByteOrder};

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
    debug!("LZC: {:?} [encoded]", lzc);
    let (lzc, lzc_codebook) = encode(&lzc); // huff(lzc)
    debug!("Size: Huff(LZC)={}", lzc.len());

    let first6 = data
        .iter()
        .map(|&x| get_value_first(&x, n) as u8)
        .collect::<Vec<u8>>();
    debug!("RE6: {:?} [encoded]", first6);
    let (first6, first6_codebook) = encode(&first6); // huff(6-residual)
    debug!("Size: Huff(6-residual)={}", first6.len());

    let leftresidual : Vec<Option<u32>> = data
        .iter()
        .map(|&x| get_value_after(&x, n))
        .collect();
    println!("Left residual {:?}", leftresidual);
    let mut _bv = BitVec::new();
    _bv.push(true);
    for left in leftresidual {
        match left {
            Some(v) => {
                let b = u32_to_bool(v);
                for i in b.into_iter() {
                    _bv.push(i);
                }
            },
            _ => {}
        }
    }
    println!("Encoded Res6: {:?}", _bv);
    let leftresidual = _bv.to_bytes(); // raw(residual - 6)
    debug!("Size: Raw(residual-6)={}", leftresidual.len());

    FileContainer::new(lzc, signs, first6, leftresidual, lzc_codebook, first6_codebook)

}

fn skip_first_zeros(bv: BitVec) -> BitVec {
    let mut ix = 0;
    while !bv.get(ix).unwrap() {
        ix += 1
    }
    let mut result = BitVec::new();
    for i in ix..bv.len() {
        result.push(bv.get(i).unwrap());
    }
    result
}

fn reverse_diff(fc: FileContainer) -> Vec<u32> {
    println!("Raw FC: {:?}", fc.raw_res6);
    let lzc = decode(BitVec::from_bytes(&fc.huff_lzc[..]), &fc.huff_lzc_codebook);
    debug!("LZC: {:?} [decoded]", lzc);
    let re6 = decode(BitVec::from_bytes(&fc.huff_6re[..]), &fc.huff_huff_6re_codebook);
    debug!("RE6: {:?} [decoded]", re6);  // might be longer
    let mut bv = BitVec::new();
    for v in re6 {
        let tmp = u32_to_bool(v as u32);
        for b in tmp {
            bv.push(b);
        }
    }
    let re6 = bv;
    debug!("Again RE6: {:?} [decoded]", re6);  // might be longer
    let res = BitVec::from_bytes(&fc.raw_res6[..]);
    debug!("Decoded Res6: {:?}", res);

    let mut res6ix = 0;
    let mut resix = 0;
    let mut result = BitVec::new();
    for l in lzc {

        // add LZC
        for _ in 0..l {
            result.push(false);
        }

        // add maximum 6, but at least remaining bits from first 6 Bits encoding
        let res6length = (32-l).min(6);
        if res6length == 0 {
            continue
        }
        for _ in 0..res6length {
            debug!("Pushing {}", re6.get(res6ix).unwrap());
            result.push(re6.get(res6ix).unwrap());
            res6ix += 1;
        }

        // fill remaining residuals if necessary
        let reslength = 32-l-res6length;
        if reslength == 0 {
            continue
        }
        for _ in 0..reslength {
            result.push(res.get(resix).unwrap());
            resix += 1
        }
        println!("Decoded {:?}", result)
    }

    let result = result.to_bytes();
    // for r in result.iter() {
    //     println!("0b{:b}", r)
    // }
    let mut data = vec![0_u32; result.len() / 4];
    BigEndian::read_u32_into(&result, &mut data);
    data
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


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compress_using_diff() {
        let data : Vec<u32> = vec![324, 9384,82, 1, 1 << 31, 9290182];
        for d in data.iter() {
            println!("{:032b}", d)
        }
        let cut = 6u32;

        let fc = process_diff(&data, cut);
        let reconstruct = reverse_diff(fc);

        assert_eq!(data, reconstruct)
    }
    #[test]
    fn test_u32_to_bool_vec() {
        let data : Vec<u32> = vec![324, 9384, 1 << 31, 9290182];
        let result : Vec<u32> = data.iter().map(|&x| u32_to_bool(x)).map(|vec| bool_to_u32(vec)).collect();
        assert_eq!(data, result)
    }
}

fn bool_to_u32(vec: Vec<bool>) -> u32 {
    vec.as_slice().iter().fold(0, |acc, &b| (acc << 1) + b as u32)
}

pub fn u32_to_bool(value: u32) -> Vec<bool> {
    let mut result: Vec<bool> = Vec::new();
    if value >= 1 << 31 {
        result.push(true)
    }
    let mut pow = value.next_power_of_two() >> 1;
    while pow > 0 {
        result.push(value & pow > 0);
        pow >>= 1;
    }
    result
}
