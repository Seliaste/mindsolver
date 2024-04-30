use nabo::{NotNan, Point};

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct Col(pub [NotNan<f64>; 3], pub char);

impl Point<f64> for Col {
    fn set(&mut self, index: u32, value: NotNan<f64>) {
        self.0[index as usize] = value;
    }
    fn get(&self, index: u32) -> NotNan<f64> {
        self.0[index as usize]
    }
    const DIM: u32 = 3;
}