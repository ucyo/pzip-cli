//! An application for the encoding phase of the compression algorithm.
//!
//! This application should help identifying and optimizing methods for actual
//! encoding of the data to be compressed.

use super::graycodeanalysis::read_u32;
use bit_vec::BitVec;
use byteorder::{BigEndian, ByteOrder};
use pzip_huffman;
use std::fs::File;
use std::io::{BufWriter, Write};
use super::mqanalysis::from_vec_u32_to_vec_u8;

fn most_significant_bit(val: u32) -> u32 {
    32 - val.leading_zeros()
}


/// Transforms a Vector of u32 to u8 and eliminates of zero values at the end of the Vector.
fn truncate(data: Vec<u32>) -> Vec<u8> {
    let src = &data[..];
    let mut ds: Vec<u8> = vec![0; 4 * data.len()];
    BigEndian::write_u32_into(src, &mut ds[..]);
    // TODO: Why do I need to use BigEndian? Are the other implementations using LittleEndian wrong?

    let mut last_element;
    loop {
        last_element = ds.pop().unwrap();
        if last_element != 0 {
            break;
        }
    }
    ds.push(last_element);
    ds
}

fn _calculate_xor_lzc(p: u32, t: u32) -> u8 {
    (p ^  t).leading_zeros() as u8
}

pub fn calcualte_xor_lzc(predictions: &Vec<u32>, truth: &Vec<u32>) -> Vec<u8>{
    predictions
        .iter()
        .zip(truth.iter())
        .map(|(&p, &t)| _calculate_xor_lzc(p, t))
        .collect::<Vec<u8>>()
}

fn _calculate_filling_zeros(p: u32, t: u32) -> u8 {
    let diff = p.max(t) - p.min(t);
    let xor = p ^ t;
    (diff.leading_zeros() - xor.leading_zeros()) as u8
}

pub fn calculate_filling_zeros(predictions: &Vec<u32>, truth: &Vec<u32>) -> Vec<u8>{
    predictions
        .iter()
        .zip(truth.iter())
        .filter(|&(p,t)| p^t != 0)
        .map(|(&p, &t)| _calculate_filling_zeros(p, t))
        .collect()
}

fn _calculate_abs_diff(p: u32, t: u32) -> u32 {
    p.max(t) - p.min(t)
}

pub fn calculate_abs_diff(predictions: &Vec<u32>, truth: &Vec<u32>) -> Vec<u32> {
    predictions
        .iter()
        .zip(truth.iter())
        .map(|(&p, &t)| _calculate_abs_diff(p, t))
        .filter(|&d| d != 0)  // do not save the residual being 0
        .collect()
}


/// Split truth and prediction datasets to LZC, FZ and Residual datasets.
///
/// 1. Reads two separate files: Truth file and predictions file.
/// 2. Calculates the LZC, FZ and Residual (via Difference) of both files.
/// 3. Encodes LZC & FZ via Huffman Codes and via Arithmetic Encoder
/// 4. Prints "outbytes" with Huff(LZC) + Huff(FZ) + Residuals
///
/// Power function is a special encoding where the MSB(Diff) + 1 is either added or
/// subtracted (depending if prediction is lower or higher than truth) to 32. Therefore
/// one encodes the length of the residual incl. the direction in one codebase
pub fn split(matches: &clap::ArgMatches) {
    let pfile = String::from(matches.value_of("prediction").unwrap());
    let tfile = String::from(matches.value_of("truth").unwrap());

    let predictions = read_u32(&pfile);
    let truth = read_u32(&tfile);

    let lzc = calcualte_xor_lzc(&predictions, &truth);
    let (lzc_encoded, _) = pzip_huffman::hufbites::encode_itself_to_bytes(&lzc);
    let arlzc_encoded = pzip_redux::encode(&lzc, 8, 10, 12);

    let fz = calculate_filling_zeros(&predictions, &truth);
    let (fz_encoded, _) = pzip_huffman::hufbites::encode_itself_to_bytes(&fz);
    let arfz_encoded = pzip_redux::encode(&fz, 8, 10, 12);

    use super::graycodeanalysis::get_value_first;
    let diff: Vec<u32> = calculate_abs_diff(&predictions, &truth);
    let first_diff_bits : Vec<u8> = diff.iter().map(|&r| get_value_first(&r, 4) as u8).collect();
    let residual_diff : u32 = diff.iter().map(|&r| 4u32.min(32 - r.leading_zeros())).sum();
    // println!("{:?}", residual_diff);
    let compact_residuals = to_u8(pack(&diff, true));
    let first_diff_residuals = compact_residuals.len() - (1 + residual_diff as usize / 8);
    let (first_diff_bits_huff, _) = pzip_huffman::hufbites::encode_itself_to_bytes(&first_diff_bits);

    let lzcdiff: Vec<u8> = diff.iter().map(|&d| d.leading_zeros() as u8).collect();
    let (lzcdiff_encoded, _) = pzip_huffman::hufbites::encode_itself_to_bytes(&lzcdiff);
    let (lzcdiff_merged_encoded, mappings) = pzip_huffman::hufbites::encode_itself_by_merged_huffman_to_bytes(&lzcdiff);
    let compact_merged_residuals = to_u8(pack_with_mapping(&diff, true, &mappings));

    let power = calculate_power(&predictions, &truth);
    let (power_encoded, _) = pzip_huffman::hufbites::encode_itself_to_bytes(&power);

    let position = calculate_position_to_truth(&predictions, &truth);
    let (position_encoded, _) = pzip_huffman::hufbites::encode_itself_to_bytes(&position);


    // File output

    let basename = tfile[..tfile.len() - 4].to_string();
    // write lzc as raw u8
    let power_filename = basename.clone() + ".power";
    let mut power_writer = BufWriter::new(File::create(power_filename).unwrap());
    power_writer.write_all(power.as_slice()).unwrap();
    // write lzc as raw u8
    let lzc_filename = basename.clone() + ".lzc";
    let mut lzc_writer = BufWriter::new(File::create(lzc_filename).unwrap());
    lzc_writer.write_all(lzc.as_slice()).unwrap();
    // write fz as raw u8
    let fz_filename = basename.clone() + ".fz";
    let mut fz_writer = BufWriter::new(File::create(fz_filename).unwrap());
    fz_writer.write_all(fz.as_slice()).unwrap();
    // write diff as raw u8
    let diff_filename = basename + ".diff";
    let mut diff_writer = BufWriter::new(File::create(diff_filename).unwrap());
    let diffu8 = from_vec_u32_to_vec_u8(diff);
    diff_writer.write_all(diffu8.as_slice()).unwrap();


    // Follwing is just output formatting

    let nbytes = lzcdiff_encoded.len() + 1 + predictions.len()/8 + first_diff_bits_huff.len() + first_diff_residuals;
    let onbytes = predictions.len() * 4;

    println!(
        "{} + {} + {} + {}= {} of {} ({}% | {:.2}) [lzchuffman, signbare, first5, residual]",
        lzcdiff_encoded.len(),
        (1 + predictions.len()/8),
        first_diff_bits_huff.len(),
        first_diff_residuals,
        nbytes,
        onbytes,
        nbytes as f64 / onbytes as f64,
        onbytes as f64 / nbytes as f64
    );

    let nbytes = lzc.len() + fz.len() + compact_residuals.len();
    let onbytes = predictions.len() * 4;

    println!(
        "{} + {} + {} = {} of {} ({}% | {:.2}) [only packed residuals]",
        lzc.len(),
        fz.len(),
        compact_residuals.len(),
        nbytes,
        onbytes,
        nbytes as f64 / onbytes as f64,
        onbytes as f64 / nbytes as f64
    );

    let cnbytes = lzc_encoded.len() + fz_encoded.len() + compact_residuals.len();
    let conbytes = predictions.len() * 4;
    println!(
        "{} + {} + {} = {} of {} ({}% | {:.2}) [Huffman coded LZC, FZ]",
        lzc_encoded.len(),
        fz_encoded.len(),
        compact_residuals.len(),
        cnbytes,
        conbytes,
        cnbytes as f64 / conbytes as f64,
        conbytes as f64 / cnbytes as f64,
    );

    let arcnbytes = arlzc_encoded.len() + arfz_encoded.len() + compact_residuals.len();
    let arconbytes = predictions.len() * 4;
    println!(
        "{} + {} + {} = {} of {} ({}% | {:.2}) [Arithmetic Range coded LZC, FZ]",
        arlzc_encoded.len(),
        arfz_encoded.len(),
        compact_residuals.len(),
        arcnbytes,
        arconbytes,
        arcnbytes as f64 / arconbytes as f64,
        arconbytes as f64 / arcnbytes as f64,
    );

    let nbytes = power_encoded.len() + compact_residuals.len();
    let onbytes = predictions.len() * 4;

    println!(
        "{} + {} = {} of {} ({}% | {:.2}) [Power coded LZC inkl. direction]",
        power_encoded.len(),
        compact_residuals.len(),
        nbytes,
        onbytes,
        nbytes as f64 / onbytes as f64,
        onbytes as f64 / nbytes as f64
    );

    let nbytes = lzcdiff_encoded.len() + position.len() + compact_residuals.len();
    let onbytes = predictions.len() * 4;

    println!(
        "{} + {} + {} = {} of {} ({}% | {:.2}) [Huffman coded diff LZC, raw binary direction]",
        lzcdiff_encoded.len(),
        position.len(),
        compact_residuals.len(),
        nbytes,
        onbytes,
        nbytes as f64 / onbytes as f64,
        onbytes as f64 / nbytes as f64
    );

    let nbytes = lzcdiff_encoded.len() + position_encoded.len() + compact_residuals.len();
    let onbytes = predictions.len() * 4;

    println!(
        "{} + {} + {} = {} of {} ({}% | {:.2}) [Huffman coded diff LZC, binary direction]",
        lzcdiff_encoded.len(),
        position_encoded.len(),
        compact_residuals.len(),
        nbytes,
        onbytes,
        nbytes as f64 / onbytes as f64,
        onbytes as f64 / nbytes as f64
    );


    let nbytes = lzcdiff_merged_encoded.len() + position_encoded.len() + compact_merged_residuals.len();
    let onbytes = predictions.len() * 4;

    println!(
        "{} + {} + {} = {} of {} ({}% | {:.2}) [Merged Huffman coded diff LZC, binary direction]",
        lzcdiff_merged_encoded.len(),
        position_encoded.len(),
        compact_merged_residuals.len(),
        nbytes,
        onbytes,
        nbytes as f64 / onbytes as f64,
        onbytes as f64 / nbytes as f64
    );



}

fn _calculate_power(p: u32, t: u32) -> u8 {
    if p > t {
        (32 - 1 - most_significant_bit(p - t) - 1) as u8
    } else if t > p {
        (32 + 1 + most_significant_bit(t - p) - 1) as u8
    } else {
        32u8
    }
}

pub fn calculate_power(predictions: &Vec<u32>, truth: &Vec<u32>) -> Vec<u8>{
    let result : Vec<u8> = predictions
        .iter()
        .zip(truth.iter())
        .map(|(&p, &t)| _calculate_power(p, t)).collect();
    result
}

/// Calculates if prediction is too high (true) or too low (false)
fn calculate_position_to_truth(predictions: &Vec<u32>, truth: &Vec<u32>) -> Vec<u8> {
    let mut bv = BitVec::new();
    bv.push(true);

    for (&p, &t) in predictions.iter().zip(truth.iter()) {
        bv.push(p > t);
    }
    bv.to_bytes()
}

/// Transforms BitVec into a Vector of u8 values with padding on the last element
fn to_u8(bv: BitVec) -> Vec<u8> {
    let bv = eliminate_first_bit(bv);
    bv.to_bytes()
}

/// Transforms BitVec into a Vector of u32 values with padding on the last element
fn to_u32(bv: BitVec) -> Vec<u32> {
    let bv = eliminate_first_bit(bv);
    let data = bv.to_bytes();
    let mut result : Vec<u32> = vec![0; data.len() / 4];
    BigEndian::read_u32_into(&data, &mut result);
    result
}

pub fn eliminate_first_bit(bv: BitVec) -> BitVec {
    let mut result = BitVec::new();
    for ix in 1..bv.len() {
        result.push(bv.get(ix).unwrap())
    }
    result
}

/// Packing of bits
/// All the set/unset bits of a vector are being squashed/packed together to represent the
/// most minimal representation of the data. The `skip` flag defines if the
/// first value (which will always be set) should be included or not.
fn pack(data: &Vec<u32>, skip: bool) -> BitVec {
    let mut result = BitVec::new();
    result.push(true);  // necessary for cases where the first value in the following is a false

    for value in data.iter() {
        let mut next = value.next_power_of_two() >> 1 + skip as usize;
        while next != 0 {
            result.push(next & value > 0);
            next >>= 1;
        }
    }
    debug!("Bitlength: {}", result.len());
    result
}

use std::collections::HashMap;
use log::debug;
fn pack_with_mapping(data: &Vec<u32>, skip: bool, map: &HashMap<u8,u8>) -> BitVec {
    let mut result = BitVec::new();
    result.push(true);  // necessary for cases where the first value in the following is a false

    let mut s = 0u32;
    for value in data.into_iter() {
        let mut next = value.next_power_of_two() >> 1 + skip as usize;
        match map.get(&(value.leading_zeros() as u8)) {
            Some(v) => {
                let mut key = *v;
                // debug!("Map {:?} to {:?}", value.leading_zeros(), key);
                s += 1;
                while key < value.leading_zeros() as u8 {
                    result.push(false);
                    // debug!("Push {:?}", false);
                    key = key + 1;
                }
            }
            _ => {}
        }
        while next != 0 {
            result.push(next & value > 0);
            next >>= 1;
        }
    }
    debug!("Changed {:?} values", s);
    debug!("Bitlength: {}", result.len());
    result
}


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_truncate_numbers() {
        let data: Vec<u32> = vec![164787381, 1 << 30, 4036830976, 3778694784];
        let expected_length = vec![4, 1, 3, 4];

        for (val, exp) in data.into_iter().zip(expected_length.into_iter()) {
            println!("{}", val);
            let single: Vec<u32> = vec![val];
            let result = truncate(single);
            println!("{:032b}", val);
            for s in result.iter() {
                print!("{:08b}", s);
            }
            print!("\n\n");
            assert_eq!(result.len(), exp);
        }
    }

    #[test]
    fn test_to_bitvec() {
        let data: Vec<u32> = vec![0b101010_01001001_01111101_11010111]; //
        let result = pack(&data, false);

        assert_eq!(to_u8(result), vec![169, 37, 247, 92]);
    }

    #[test]
    fn test_to_bitvec_skipped() {
        let data: Vec<u32> = vec![0b101010_01001001_01111101_11010111]; //
        let result = pack(&data, true);

        assert_eq!(to_u8(result), vec![82, 75, 238, 92 << 1]);
    }

    #[test]
    fn test_bitvec_to_u32() {
        let data: Vec<u32> = vec![62736423];
        println!("0b{:b}", data[0]);
        let lz = data[0].leading_zeros();
        let result = pack(&data, false);
        let result = to_u32(result);
        println!("0b{:b} {} ", result[0], lz);

        assert_eq!(result[0], data[0] << lz);  // bitvec will fill the values with zeros
    }

    #[test]
    fn test_bitvec_to_u32_skipped() {
        let data: Vec<u32> = vec![62736423];
        let lz = data[0].leading_zeros() + 1;  // because first is skipped
        let result = pack(&data, true);
        println!("{:?}", result);
        let result = to_u32(result);

        for r in result.iter() {
            println!("{:#032b}", r);
        }

        assert_eq!(result[0], data[0] << lz);  // bitvec will fill the values with zeros
    }
}
