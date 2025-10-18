use std::io::Read;

use matrix_rank_gf2::{MATRIX_LEN, matrix_rank, plot::plot};

fn main() {
    // let expected = expected_rank_distribution();

    let mut counts = [0usize; MATRIX_LEN + 1];

    let trials = 1_000_000;

    let mut stdin = std::io::stdin();
    for _ in 0..trials {
        let mut matrix = [0u8; MATRIX_LEN * 4];
        stdin.read_exact(&mut matrix).unwrap();

        let mut bit_matrix = [0u32; MATRIX_LEN];
        for i in 0..MATRIX_LEN {
            bit_matrix[i] = u32::from_le_bytes([
                matrix[i * 4],
                matrix[i * 4 + 1],
                matrix[i * 4 + 2],
                matrix[i * 4 + 3],
            ]);
        }
        let rank = matrix_rank(&mut bit_matrix);
        counts[rank] += 1;
    }

    /*
    let dist = counts.map(|c| c as f64 / trials as f64);
    println!("Rank Distribution (Observed vs Expected):");
    println!("Rank\tObserved\tExpected");
    for i in 0..=32 {
        println!("{:>2}\t{:.6}\t{:.6}", i, dist[i], expected[i]);
    }
    */

    let dist = counts.map(|c| c as f32 / trials as f32);
    plot(&dist).unwrap();
}
