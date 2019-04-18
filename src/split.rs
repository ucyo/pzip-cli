use super::graycodeanalysis::read_u32;
use super::pzip::transform::{Compact, CompactMapping};
use byteorder::{BigEndian, ByteOrder};

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
}
