use std::fs;
use std::io::{Read, BufReader};
use super::{parse_shape, parse_ring};
use byteorder::{ByteOrder, LittleEndian};
use pzip::predictors::PredictorTrait;
use pzip::residual::{RContext, ResidualTrait};
use super::foc::process_bwt_and_range;

pub fn best(matches: &clap::ArgMatches) {

    // parse cli
    let ifile = String::from(matches.value_of("input").unwrap());
    let shape = parse_shape(&matches);
    let ring = parse_ring(&matches);
    let size = (shape.x * shape.y * shape.z) as usize;
    let cut = 31;

    // read f32 file
    let file = fs::File::open(ifile).unwrap();
    let mut bytes: Vec<u8> = Vec::new(); // TODO: Optimize? we know the size
    let s = BufReader::new(file).read_to_end(&mut bytes).unwrap(); // TODO: Optimize? we know the size
    if s % 4 != 0 {
            panic!("Can not be read into f32");
        }
    let mut data: Vec<f32> = vec![0f32; s / 4];
    LittleEndian::read_f32_into(&bytes, &mut data);

    // get predictions
    let mut predictor = pzip::predictors::predictors::get_lorenz_f32();
    let predictions = predictor.consume(&data, &shape, ring);

    let data : Vec<u32> = data.iter().map(|&x| x.to_bits()).collect();
    let predictions : Vec<u32> = predictions.iter().map(|&x| x.to_bits()).collect();

    //calculate residuals
    let mut rctx = RContext::new(cut);
    let r = pzip::residual::ResidualCalculation::Shifted;
    let diff : Vec<u32> = data.iter().zip(predictions.iter()).map(|(&t,&p)| {
        let result = r.residual(&t, &p, &mut rctx);
        r.update(&t, &p, &mut rctx);
        result
    }).collect();

    let fc = process_bwt_and_range(&diff);
    println!("{} cr={}", fc, s as f64 / fc.nbytes() as f64);
}
