use super::graycodeanalysis::read_u32;
use super::pzip::transform::{Compact, CompactMapping};

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
        .filter(|(_d, &xor)| xor != 32)  // or where d != 0 (only accept values where LZC != 32 )
        .map(|(d, &xor)| d - xor)
        .collect();

    let residual = predictions
        .iter()
        .zip(truth.iter())
        .map(|(&p, &t)| p.max(t) - p.min(t))
        .map(|diff| diff - (diff.next_power_of_two() >> 1))
        .collect::<Vec<u32>>();

    // check residuals
    // for v in residual.iter() {
    //     println!("{:b}", v)
    // }

    let compact_residuals = Compact::NoLZC.compact_u32(residual);
    // TODO: This does fill up to 32. But should be 8. Maybe we rewrite this using bit vec?

    // check compact form
    // for v in compact_residuals.iter() {
    //     println!("{:b}", v)
    // }

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
