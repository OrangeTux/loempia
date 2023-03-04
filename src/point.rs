use std::marker::PhantomData;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Relative;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Absolute;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Coordinate<T> {
    pub x: i32,
    pub y: i32,
    _type: PhantomData<T>,
}

impl<T> Coordinate<T> {
    pub fn new(x: i32, y: i32) -> Self {
        Self {
            x,
            y,
            _type: Default::default(),
        }
    }
}
