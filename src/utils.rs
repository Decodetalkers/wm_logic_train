use std::iter::Sum;
use std::ops::{Add, Div};
pub(crate) trait MapUnit:
    Copy + Add<Output = Self> + Div<Output = Self> + Sum<Self>
{
    fn zero() -> Self;
    fn two() -> Self;
}

macro_rules! impl_unit {
    ($Type:ident, $value:expr, $value_2:expr) => {
        impl MapUnit for $Type {
            fn zero() -> Self {
                $value
            }
            fn two() -> Self {
                $value_2
            }
        }
    };
}

impl_unit!(f32, 0., 2.);
impl_unit!(i32, 0, 2);
impl_unit!(u32, 0, 2);

#[derive(Debug, Clone, Copy)]
pub struct Size<T = f32> {
    pub width: T,
    pub height: T,
}

impl<T: MapUnit> Add for Size<T> {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            width: self.width + other.width,
            height: self.height + other.height,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Position<T = f32> {
    pub x: T,
    pub y: T,
}

impl<T: MapUnit> Add for Position<T> {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}
#[derive(Debug, Clone, Copy)]
pub struct SizeAndPos<T = f32>
where
    T: Copy,
{
    pub size: Size<T>,
    pub position: Position<T>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InsertWay {
    Vertical,
    Horizontal,
}

impl<T: MapUnit> Add for SizeAndPos<T> {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            size: self.size + other.size,
            position: self.position + other.position,
        }
    }
}
impl<T: MapUnit> SizeAndPos<T> {
    fn vertical(&mut self) -> Self {
        let width = self.size.width / T::two();
        let height = self.size.height / T::two();
        self.size.width = width;
        self.size.height = height;
        let y = self.position.y + height;
        Self {
            size: Size { width, height },
            position: Position {
                x: self.position.x,
                y,
            },
        }
    }
    fn horizontal(&mut self) -> Self {
        let width = self.size.width / T::two();
        let height = self.size.height / T::two();
        self.size.width = width;
        self.size.height = height;
        let x = self.position.x + width;
        Self {
            size: Size { width, height },
            position: Position {
                x,
                y: self.position.y,
            },
        }
    }
    pub fn split(&mut self, way: InsertWay) -> Self {
        match way {
            InsertWay::Vertical => self.vertical(),
            InsertWay::Horizontal => self.horizontal(),
        }
    }
}

impl SizeAndPos<f32> {}
