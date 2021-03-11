#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Position {
    x: f32,
    y: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ModelReference {
    asset_reference: String,
}
