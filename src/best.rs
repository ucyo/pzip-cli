use std::fs;
use std::io::{Read, BufReader};
use super::{parse_shape, parse_ring};
use byteorder::{ByteOrder, LittleEndian};
use pzip::predictors::PredictorTrait;
use pzip::residual::{RContext, ResidualTrait};
use super::foc::process_bwt_and_range;

pub fn best(matches: &clap::ArgMatches) {
    let start = std::time::Instant::now();
    // parse cli
    // let start = std::time::Instant::now();
    let ifile = String::from(matches.value_of("input").unwrap());
    let shape = parse_shape(&matches);
    let ring = parse_ring(&matches);
    let size = (shape.x * shape.y * shape.z) as usize;
    let cut = 31;
    // println!("      cli: {:.5} sec", start.elapsed().as_float_secs());

    // read f32 file
    // let start = std::time::Instant::now();
    let file = fs::File::open(ifile).unwrap();
    let mut bytes: Vec<u8> = Vec::with_capacity(size * 4);
    let s = BufReader::with_capacity(size * 4, file).read_to_end(&mut bytes).unwrap();
    // if s % 4 != 0 {
    //         panic!("Can not be read into f32");
    //     }
    // assert_eq!(s, size * 4);
    let mut data: Vec<f32> = vec![0f32; size];
    LittleEndian::read_f32_into(&bytes, &mut data);
    // println!("     read: {:.5} sec", start.elapsed().as_float_secs());

    // get predictions
    // let start = std::time::Instant::now();
    let mut predictor = pzip::predictors::predictors::get_lorenz_f32();
    let predictions = predictor.consume(&data, &shape, ring);
    let data : Vec<u32> = data.iter().map(|&x| x.to_bits()).collect();
    let predictions : Vec<u32> = predictions.iter().map(|&x| x.to_bits()).collect();
    // println!("    preds: {:.5} sec", start.elapsed().as_float_secs());

    //calculate residuals
    // let start = std::time::Instant::now();
    let mut rctx = RContext::new(cut);
    let r = pzip::residual::ResidualCalculation::Shifted;
    let diff : Vec<u32> = data.iter().zip(predictions.iter()).map(|(&t,&p)| {
        let result = r.residual(t, p, &mut rctx);
        r.update(t, p, &mut rctx);
        result
    }).collect();
    // println!("residuals: {:.5} sec", start.elapsed().as_float_secs());

    // let start = std::time::Instant::now();
    let fc = process_bwt_and_range(&diff);
    // println!("      enc: {:.5} sec", start.elapsed().as_float_secs());

    println!("{} ratio={:.2} throughput={:.2} MiB/s", fc, s as f64 / fc.nbytes() as f64, (size as f64 * 4_f64 /1024_f64/1024_f64) / start.elapsed().as_float_secs());
}

use pzip::predictors::Ignorant;
use pzip::position::Position;
use pzip::gen::GeneratorIteratorAdapter;
use pzip::ptraversal::single_neighbours_grouped_no_ring_float;
fn consume(predictor : &mut Ignorant<f32>, data : &Vec<f32>, shape: &Position) -> Vec<f32> {
    let spaces: Vec<Vec<f32>> = GeneratorIteratorAdapter(single_neighbours_grouped_no_ring_float(shape, &predictor.cells, data)).collect();
    let mut result = Vec::new();
    for (i, space) in spaces.iter().enumerate() {
        result.push(predictor.predict(space));
        predictor.update(data[i]);
    }
    result
}
