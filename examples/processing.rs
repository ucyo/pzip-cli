use std::fs;
use std::io::Read;

const BLOCKSIZE: usize = 128;

fn first_block(bytes: &Vec<u8>) -> Vec<f32> {
    let mut result = vec![0f32;BLOCKSIZE];
    for val in bytes.iter().take(BLOCKSIZE/8) {
        for j in 0..8 {
            let num = ((*val & 1) << j) > 0;
            let minus = result.remove(0);
            for element in result.iter_mut() {
                *element -= minus;
            }
            result.push(num as i32 as f32);
        }
    }

    for element in result.iter_mut() {
        *element /= BLOCKSIZE as f32;
    }
    result
}

fn calculate_correlation(one: &Vec<f32>, other: &Vec<f32>) -> f32 {
    unimplemented!();
}


fn main() {
    let filename = "";
    let threshold = 0.9f32;
    let mut file = fs::File::open(filename).unwrap();
    let mut bytes: Vec<u8> = Vec::new();
    let _ = file.read_to_end(&mut bytes).unwrap();

    let mut results_ones = first_block(&bytes);

    let mut candidates : Vec<Vec<f32>> = Vec::new();
    candidates.push(results_ones.clone());

    'out: for val in bytes.iter().skip(BLOCKSIZE/8) {
        for j in 0..8 {
            let num = ((val & 1) << j) > 0;
            let minus = results_ones.remove(0);
            for element in results_ones.iter_mut() {
                *element -= minus;
            }
            results_ones.push(num as i32 as f32);
            let corr : Vec<f32> = candidates.iter()
                  .map(|candidate| calculate_correlation(candidate, &results_ones))
                  .collect();
            for result in corr.iter() {
                if *result > threshold {
                    continue 'out
                }
            }
        }
        candidates.push(results_ones.clone())
    }
}
