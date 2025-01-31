#[derive(Default, Copy, Clone)]
pub struct Cell {
    elevation: f32,
}

pub fn generate() {
    let mut rng = rand::rng();
    let f: [Cell; 64] = [Cell::new(); 64];
}
