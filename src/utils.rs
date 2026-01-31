use std::iter::Sum;
use std::ops::{Add, AddAssign, Div, Neg, Sub, SubAssign};
pub(crate) trait MapUnit:
    Copy
    + Add<Output = Self>
    + AddAssign
    + Sub<Output = Self>
    + SubAssign
    + Div<Output = Self>
    + Sum<Self>
{
    fn zero() -> Self;
    fn two() -> Self;
    fn to_f32(&self) -> f32;
    fn from_f32(val: f32) -> Self;
    fn mul_f32(&self, val: f32) -> Self;
}

pub(crate) trait MinusAbleMatUnit: MapUnit + Neg<Output = Self> {}

macro_rules! impl_unit {
    ($Type:ident, $value:expr, $value_2:expr) => {
        impl MapUnit for $Type {
            fn zero() -> Self {
                $value
            }
            fn two() -> Self {
                $value_2
            }
            fn to_f32(&self) -> f32 {
                *self as f32
            }
            fn from_f32(val: f32) -> Self {
                val as $Type
            }
            fn mul_f32(&self, val: f32) -> Self {
                ((*self as f32) * val) as $Type
            }
        }
    };
}
macro_rules! impl_minus_able_unit {
    ($Type:ident, $value:expr, $value_2:expr) => {
        impl_unit!($Type, $value, $value_2);
        impl MinusAbleMatUnit for $Type {}
    };
}
impl_minus_able_unit!(f32, 0., 2.);
impl_minus_able_unit!(i32, 0, 2);
impl_unit!(u32, 0, 2);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

impl<T: MapUnit> Sub for Size<T> {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self {
            width: self.width - other.width,
            height: self.height - other.height,
        }
    }
}

impl<T: MapUnit> Size<T> {
    pub fn split_h(&self, pieces: T) -> Self {
        Self {
            width: self.width / pieces,
            ..*self
        }
    }
    pub fn split_v(&self, pieces: T) -> Self {
        Self {
            height: self.height / pieces,
            ..*self
        }
    }
    pub fn split(&self, pieces: T, way: InsertWay) -> Self {
        match way {
            InsertWay::Horizontal => self.split_h(pieces),
            InsertWay::Vertical => self.split_v(pieces),
        }
    }
    pub fn percent_h(&self, percent: f32) -> Self {
        Self {
            width: self.width.mul_f32(percent),
            height: self.height,
        }
    }
    pub fn percent_v(&self, percent: f32) -> Self {
        Self {
            width: self.width,
            height: self.height.mul_f32(percent),
        }
    }
    pub fn percent(&self, percent: f32, way: InsertWay) -> Self {
        match way {
            InsertWay::Horizontal => self.percent_h(percent),
            InsertWay::Vertical => self.percent_v(percent),
        }
    }
}

impl Size<f32> {
    pub fn whole() -> Self {
        Self {
            width: 1.,
            height: 1.,
        }
    }
    /// This compute the change of the percent
    pub fn change_expand(&self, way: InsertWay) -> Self {
        match way {
            InsertWay::Horizontal => Self {
                width: self.width,
                height: 0.,
            },
            InsertWay::Vertical => Self {
                width: 0.,
                height: self.height,
            },
        }
    }
}

impl<T: MapUnit> AddAssign for Size<T> {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}
impl<T: MapUnit> SubAssign for Size<T> {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl<T: MapUnit> Size<T> {
    pub fn zero() -> Self {
        Self {
            width: T::zero(),
            height: T::zero(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

impl<T: MapUnit> AddAssign for Position<T> {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl<T: MapUnit> Sub for Position<T> {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}
impl<T: MapUnit> SubAssign for Position<T> {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl<T: MapUnit> Position<T> {
    pub fn zero() -> Self {
        Self {
            x: T::zero(),
            y: T::zero(),
        }
    }
}
#[derive(Debug, Clone, Copy)]
pub struct SizeAndPos<T = f32> {
    pub size: Size<T>,
    pub position: Position<T>,
}

impl<T: MapUnit> Add for SizeAndPos<T> {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self {
            size: self.size + rhs.size,
            position: self.position + rhs.position,
        }
    }
}

impl<T: MapUnit> AddAssign for SizeAndPos<T> {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl<T: MapUnit> Sub for SizeAndPos<T> {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self {
            size: self.size - rhs.size,
            position: self.position - rhs.position,
        }
    }
}

impl<T: MapUnit> SubAssign for SizeAndPos<T> {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InsertWay {
    Vertical,
    Horizontal,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReMapDirection {
    Left,
    Right,
    Top,
    Bottom,
}

impl ReMapDirection {
    pub fn expend_way(insert_way: InsertWay, start: bool) -> Self {
        match (insert_way, start) {
            (InsertWay::Horizontal, true) => Self::Left,
            (InsertWay::Horizontal, false) => Self::Right,
            (InsertWay::Vertical, true) => Self::Top,
            (InsertWay::Vertical, false) => Self::Bottom,
        }
    }
}

impl<T: MinusAbleMatUnit> SizeAndPos<T> {
    pub fn expend_change(&self, direction: ReMapDirection) -> Self {
        match direction {
            ReMapDirection::Top => Self {
                position: Position {
                    x: T::zero(),
                    y: -self.position.y,
                },
                size: Size {
                    width: T::zero(),
                    height: self.size.height,
                },
            },
            ReMapDirection::Bottom => Self {
                position: Position::zero(),
                size: Size {
                    width: T::zero(),
                    height: self.size.height,
                },
            },
            ReMapDirection::Left => Self {
                position: Position {
                    x: -self.position.x,
                    y: T::zero(),
                },
                size: Size {
                    width: self.size.width,
                    height: T::zero(),
                },
            },
            ReMapDirection::Right => Self {
                position: Position::zero(),
                size: Size {
                    width: self.size.width,
                    height: T::zero(),
                },
            },
        }
    }
}
