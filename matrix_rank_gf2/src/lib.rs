pub mod plot;

pub const MATRIX_LEN: usize = 32;

pub fn expected_rank_distribution() -> [f64; MATRIX_LEN + 1] {
    let mut distribution = [0.0; MATRIX_LEN + 1];

    const L: i32 = MATRIX_LEN as i32;
    const K: i32 = u32::BITS as i32;

    distribution[0] = 2.0_f64.powi(-L * K);
    for x in 1..33 as i32 {
        let a = 2.0_f64.powi(x * (L + K - x) - L * K);
        let mut b = 1.0;
        for i in 0..x {
            b *= 1.0 - 2.0_f64.powi(i - L);
            b *= 1.0 - 2.0_f64.powi(i - K);
            b /= 1.0 - 2.0_f64.powi(i - x);
        }

        distribution[x as usize] = a * b;
    }
    distribution
}

// Matrix rank of bit matrix
pub fn matrix_rank(matrix: &mut [u32]) -> usize {
    let n = matrix.len();
    let mut rank = 0;
    for bit in 0..u32::BITS {
        let mut pivot = None;
        for i in rank..n {
            if (matrix[i] >> bit) & 1 == 1 {
                pivot = Some(i);
                break;
            }
        }
        if let Some(pivot) = pivot {
            matrix.swap(rank, pivot);
            for i in 0..n {
                if i != rank && (matrix[i] >> bit) & 1 == 1 {
                    matrix[i] ^= matrix[rank];
                }
            }
            rank += 1;
        }
    }
    rank
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_matrix_rank() {
        let mut matrix = vec![0b1100, 0b1010, 0b1001];
        assert_eq!(matrix_rank(&mut matrix), 3);
        let mut matrix = vec![0b1100, 0b1100, 0b1100];
        assert_eq!(matrix_rank(&mut matrix), 1);
        let mut matrix = vec![0b0000, 0b0000, 0b0000];
        assert_eq!(matrix_rank(&mut matrix), 0);
    }
}
