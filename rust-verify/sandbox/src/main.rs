fn main() {}

pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

pub fn fma(a: i32, b: i32, c: i32) -> i32 {
    a * b + c
}

#[repr(transparent)]
pub struct Pcg64Si {
    state: u64,
}

impl Pcg64Si {
    pub fn next_u64(&mut self) -> u64 {
        let old_state = self.state;
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);

        let word =
            ((old_state >> ((old_state >> 59) + 5)) ^ old_state).wrapping_mul(12605985483714917081);
        (word >> 43) ^ word
    }
}
