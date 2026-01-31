use std::ops::Add;
#[derive(Debug, Clone, Copy)]
pub struct Size<T = i32> {
    pub width: T,
    pub height: T,
}

macro_rules! impl_size {
    ($Type: ident) => {
        impl Add for Size<$Type> {
            type Output = Self;
            fn add(self, other: Self) -> Self {
                Self {
                    width: self.width + other.width,
                    height: self.height + other.height,
                }
            }
        }
    };
}

impl_size!(i32);

#[derive(Debug, Clone, Copy)]
pub struct Position<T = i32> {
    pub x: T,
    pub y: T,
}

macro_rules! impl_position {
    ($Type: ident) => {
        impl Add for Position<$Type> {
            type Output = Self;
            fn add(self, other: Self) -> Self {
                Self {
                    x: self.x + other.x,
                    y: self.y + other.y,
                }
            }
        }
    };
}

impl_position!(i32);

#[derive(Debug, Clone, Copy)]
pub struct SizeAndPos<T = i32>
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

macro_rules! impl_size_and_pos {
    ($Type: ident, $Div:expr) => {
        impl Add for SizeAndPos<$Type> {
            type Output = Self;
            fn add(self, other: Self) -> Self {
                Self {
                    size: self.size + other.size,
                    position: self.position + other.position,
                }
            }
        }
        impl SizeAndPos<$Type> {
            fn vertical(&mut self) -> Self {
                let width = self.size.width / $Div;
                let height = self.size.height / $Div;
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
                let width = self.size.width / $Div;
                let height = self.size.height / $Div;
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
    };
}

//impl_size_and_pos!(u32, 2);
impl_size_and_pos!(i32, 2);
