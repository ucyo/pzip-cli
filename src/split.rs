//! An application for the encoding phase of the compression algorithm.
//!
//! This application should help identifying and optimizing methods for actual
//! encoding of the data to be compressed.

use super::graycodeanalysis::read_u32;
use bit_vec::BitVec;
use byteorder::{BigEndian, ByteOrder};
use pzip_huffman;

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

/// Split truth and prediction datasets to LZC, FZ and Residual datasets.
///
/// 1. Reads two separate files: Truth file and predictions file.
/// 2. Calculates the LZC, FZ and Residual (via Difference) of both files.
/// 3. Encodes LZC & FZ via Huffman Codes and via Arithmetic Encoder
/// 4. Prints "outbytes" with Huff(LZC) + Huff(FZ) + Residuals
pub fn split(matches: &clap::ArgMatches) {
    let pfile = String::from(matches.value_of("prediction").unwrap());
    let tfile = String::from(matches.value_of("truth").unwrap());

    let predictions = read_u32(&pfile);
    let truth = read_u32(&tfile);

    let lzc: Vec<u8> = predictions
        .iter()
        .zip(truth.iter())
        .map(|(&p, &t)| (p ^ t).leading_zeros() as u8)
        .collect();

    let fz: Vec<u8> = predictions
        .iter()
        .zip(truth.iter())
        .map(|(&p, &t)| (p.max(t) - p.min(t)).leading_zeros() as u8)
        .zip(lzc.iter())
        .filter(|(_d, &xor)| xor != 32) // or where d != 0 (only accept values where LZC != 32 )
        .map(|(d, &xor)| d - xor)
        .collect();

    let residual = predictions
        .iter()
        .zip(truth.iter())
        .map(|(&p, &t)| p.max(t) - p.min(t))
        .map(|diff| diff - (diff.next_power_of_two() >> 1))
        .collect::<Vec<u32>>();
    // TODO: This might be wrong if the leading zeros are not safed.
    // TODO: Export function to eliminate first ones (but including all following zeros)
    // TODO: Use bitVec!!!

    let compact_residuals = Compact::NoLZC.compact_u32(residual);
    let compact_residuals = truncate(compact_residuals);
    // TODO: Check if compact_u32 is really doing what it is suppposed to do including adding leading zeros

    println!(
        "LZC: {:?}\nfz:{:?}\nResidual: {:?}\n",
        lzc.len(),
        fz.len(),
        compact_residuals.len()
    )
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
        let result = to_bitvec(&data, false);

        assert_eq!(to_u8(result), vec![169, 37, 247, 92]);
    }

    #[test]
    fn test_to_bitvec_skipped() {
        let data: Vec<u32> = vec![0b101010_01001001_01111101_11010111]; //
        let result = to_bitvec(&data, true);

        assert_eq!(to_u8(result), vec![82, 75, 238, 92 << 1]);
    }

    #[test]
    fn test_bitvec_to_u32() {
        let data: Vec<u32> = vec![62736423];
        let lz = data[0].leading_zeros();
        let result = to_bitvec(&data, false);
        println!("{:?}", result);
        let result = to_u32(result);

        for r in result.iter() {
            println!("{:#032b}", r);
        }

        assert_eq!(result[0], data[0] << lz);  // bitvec will fill the values with zeros
    }

    #[test]
    fn test_bitvec_to_u32_skipped() {
        let data: Vec<u32> = vec![62736423];
        let lz = data[0].leading_zeros() + 1;  // because first is skipped
        let result = to_bitvec(&data, true);
        println!("{:?}", result);
        let result = to_u32(result);

        for r in result.iter() {
            println!("{:#032b}", r);
        }

        assert_eq!(result[0], data[0] << lz);  // bitvec will fill the values with zeros
    }
}
