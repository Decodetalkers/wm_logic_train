#[cfg(test)]
mod tests;
mod utils;

use std::hash::Hash;
use std::sync::atomic::{self, AtomicU64};
mod error;

pub use error::ElementNotFound as Error;

pub use crate::utils::{InsertWay, Percentage, Position, ReMapDirection, Size, SizeAndPos};

use crate::utils::{MapUnit, MinusAbleMatUnit};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
/// The id of the window.
///
/// Internally Iced reserves `window::Id::MAIN` for the first window spawned.
pub struct Id(pub u64);

impl std::fmt::Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Id: {}", self.0))
    }
}

pub type Result<T> = std::result::Result<T, Error>;

static COUNT: AtomicU64 = AtomicU64::new(0);

impl Id {
    /// The reserved window [`Id`] for the first window in an Iced application.
    pub const MAIN: Self = Id(0);

    /// Creates a new unique window [`Id`].
    pub fn unique() -> Id {
        Id(COUNT.fetch_add(1, atomic::Ordering::Relaxed))
    }

    /// It is used in unit test
    #[cfg(test)]
    fn next(&self) -> Id {
        Id(self.0 + 1)
    }
}

#[derive(Debug, Clone)]
pub enum ElementMap<T: MapUnit = f32> {
    EmptyOutput(SizeAndPos<T>),
    Window {
        id: Id,
        size_pos: SizeAndPos<T>,
        // This storage current percentage in the container (it is in container)
        percent: Percentage,
    },
    Vertical {
        elements: Vec<ElementMap<T>>,
        size_pos: SizeAndPos<T>,
        percent: Percentage,
    },
    Horizontal {
        elements: Vec<ElementMap<T>>,
        size_pos: SizeAndPos<T>,
        percent: Percentage,
    },
}

pub trait DispatchCallback<T: MapUnit> {
    fn callback(&mut self, id: Id, size_pos: SizeAndPos<T>);
}

impl<F, T: MapUnit> DispatchCallback<T> for F
where
    F: FnMut(Id, SizeAndPos<T>),
{
    fn callback(&mut self, id: Id, size_pos: SizeAndPos<T>) {
        self(id, size_pos)
    }
}

impl<T: MinusAbleMatUnit> ElementMap<T> {
    pub fn new(size_pos: SizeAndPos<T>) -> Self {
        Self::EmptyOutput(size_pos)
    }

    pub fn id(&self) -> Option<Id> {
        match self {
            Self::Window { id, .. } => Some(*id),
            _ => None,
        }
    }
    pub fn percent(&self) -> Percentage {
        match self {
            Self::Vertical { percent, .. }
            | Self::Horizontal { percent, .. }
            | Self::Window { percent, .. } => *percent,
            Self::EmptyOutput(_) => Size::whole(),
        }
    }

    pub fn size_pos(&self) -> SizeAndPos<T> {
        match self {
            Self::Vertical { size_pos, .. }
            | Self::Horizontal { size_pos, .. }
            | Self::Window { size_pos, .. }
            | Self::EmptyOutput(size_pos) => *size_pos,
        }
    }

    pub fn size(&self) -> Size<T> {
        self.size_pos().size
    }

    pub fn width(&self) -> T {
        self.size().width
    }

    pub fn height(&self) -> T {
        self.size().height
    }

    // NOTE: how to design it? what should I do with the size_pos? how does it mean?
    // maybe I need minus
    fn expand<F>(&mut self, change: SizeAndPos<T>, diff_percent: Size, callback: &mut F)
    where
        F: DispatchCallback<T>,
    {
        match self {
            Self::EmptyOutput(_) => {}
            Self::Window {
                size_pos,
                id,
                percent,
            } => {
                *size_pos += change;
                *percent += diff_percent;
                callback.callback(*id, *size_pos);
            }
            // NOTE: this time only change the percent of the top, and the size of the elements
            Self::Vertical {
                elements,
                size_pos,
                percent,
            } => {
                *size_pos += change;
                *percent += diff_percent;
                let container_height = size_pos.size.height;
                let mut current_y = size_pos.position.y;

                for element in elements {
                    let percent_y = element.percent().height;
                    // This will be the width
                    let height = container_height.mul_f32(percent_y);

                    let new_s_and_p: SizeAndPos<T> = SizeAndPos {
                        size: Size {
                            width: size_pos.size.width,
                            height,
                        },
                        position: Position {
                            x: size_pos.position.x,
                            y: current_y,
                        },
                    };
                    // Now we can caculte current height
                    current_y += height;
                    let s_and_p = element.size_pos();
                    let diff_change = new_s_and_p - s_and_p;
                    element.expand(diff_change, Size::zero(), callback);
                }
            }
            Self::Horizontal {
                elements,
                size_pos,
                percent,
            } => {
                *size_pos += change;
                *percent += diff_percent;
                let container_width = size_pos.size.width;
                let mut current_x = size_pos.position.x;

                for element in elements {
                    let percent_x = element.percent().width;
                    // This will be the width
                    let width = container_width.mul_f32(percent_x);

                    let new_s_and_p: SizeAndPos<T> = SizeAndPos {
                        size: Size {
                            width,
                            height: size_pos.size.height,
                        },
                        position: Position {
                            x: current_x,
                            y: size_pos.position.y,
                        },
                    };
                    // Now we can caculte current height
                    current_x += width;
                    let s_and_p = element.size_pos();
                    let diff_change = new_s_and_p - s_and_p;
                    element.expand(diff_change, Size::zero(), callback);
                }
            }
        }
    }

    fn set_percentage(&mut self, c_percent: Size) {
        match self {
            Self::EmptyOutput(_) => {}
            Self::Vertical { percent, .. }
            | Self::Horizontal { percent, .. }
            | Self::Window { percent, .. } => *percent = c_percent,
        }
    }

    fn set_size_and_pos(&mut self, c_size_pos: SizeAndPos<T>) {
        match self {
            Self::EmptyOutput(size_pos)
            | Self::Vertical { size_pos, .. }
            | Self::Horizontal { size_pos, .. }
            | Self::Window { size_pos, .. } => *size_pos = c_size_pos,
        }
    }

    pub fn is_window(&self) -> bool {
        matches!(self, Self::Window { .. })
    }

    pub fn has_id(&self, target: Id) -> bool {
        match self {
            Self::Window { id, .. } => *id == target,
            Self::EmptyOutput(_) => true,
            Self::Vertical { elements, .. } | Self::Horizontal { elements, .. } => {
                for element in elements {
                    if element.has_id(target) {
                        return true;
                    }
                }
                false
            }
        }
    }

    fn find_element_mut(&mut self, target: Id) -> Option<&mut Self> {
        match self {
            Self::EmptyOutput(_) => None,
            Self::Window { id, .. } => (*id == target).then_some(self),
            Self::Vertical { elements, .. } | Self::Horizontal { elements, .. } => {
                for element in elements {
                    let try_find = element.find_element_mut(target);
                    if try_find.is_some() {
                        return try_find;
                    }
                }
                None
            }
        }
    }

    // NOTE: not just find it, but return the insert position
    fn find_duo_element_mut(
        &mut self,
        target_one: Id,
        target_two: Id,
    ) -> (Option<&mut Self>, Option<&mut Self>) {
        match self {
            Self::EmptyOutput(_) => (None, None),
            Self::Window { id, .. } => {
                if *id == target_one {
                    return (Some(self), None);
                }
                if *id == target_two {
                    return (None, Some(self));
                }
                (None, None)
            }
            Self::Vertical { elements, .. } | Self::Horizontal { elements, .. } => {
                let mut find_one = None;
                let mut find_two = None;
                for element in elements {
                    if find_two.is_some() && find_two.is_some() {
                        break;
                    }
                    if find_one.is_none() && find_two.is_none() {
                        let (new_one, new_two) =
                            element.find_duo_element_mut(target_one, target_two);
                        find_one = new_one;
                        find_two = new_two;
                        continue;
                    }
                    if find_one.is_none() && find_two.is_some() {
                        find_one = element.find_element_mut(target_one);
                        continue;
                    }
                    find_two = element.find_element_mut(target_two);
                }
                (find_one, find_two)
            }
        }
    }

    /// Swap two elements
    pub fn swap<F>(&mut self, id: Id, target: Id, f: &mut F) -> Result<()>
    where
        F: DispatchCallback<T>,
    {
        let (Some(element_one), Some(element_two)) = self.find_duo_element_mut(id, target) else {
            return Err(Error);
        };
        // == swap size_pos ==
        let size_pos_one = element_one.size_pos();
        let size_pos_two = element_two.size_pos();
        element_one.set_size_and_pos(size_pos_two);
        element_two.set_size_and_pos(size_pos_one);
        // == swap percent ==
        let percent_one = element_one.percent();
        let percent_two = element_two.percent();
        element_one.set_percentage(percent_two);
        element_two.set_percentage(percent_one);
        f.callback(id, element_one.size_pos());
        f.callback(target, element_two.size_pos());
        Ok(())
    }

    /// Remap, when the container or the display changed, invoke this function
    pub fn remap<F>(&mut self, c_size_pos: SizeAndPos<T>, f: &mut F)
    where
        F: DispatchCallback<T>,
    {
        let fit_way = match self {
            Self::Vertical { .. } => InsertWay::Vertical,
            // NOTE: it will only be used in pattern three, so,
            // this part will always be Self::Horizontal
            _ => InsertWay::Horizontal,
        };
        match self {
            Self::EmptyOutput(size_pos) => *size_pos = c_size_pos,
            Self::Window { id, size_pos, .. } => {
                *size_pos = c_size_pos;
                f.callback(*id, *size_pos);
            }
            Self::Vertical {
                elements, size_pos, ..
            }
            | Self::Horizontal {
                elements, size_pos, ..
            } => {
                *size_pos = c_size_pos;
                let mut current_x = size_pos.position.x;
                let mut current_y = size_pos.position.y;
                let con_width = size_pos.size.width;
                let con_height = size_pos.size.height;
                for element in elements {
                    let Percentage {
                        width: w_p,
                        height: h_p,
                    } = element.percent();
                    let new_width = con_width.mul_f32(w_p);
                    let new_height = con_height.mul_f32(h_p);
                    let c_size_pos = SizeAndPos {
                        size: Size {
                            width: new_width,
                            height: new_height,
                        },
                        position: Position {
                            x: current_x,
                            y: current_y,
                        },
                    };
                    element.remap(c_size_pos, f);
                    match fit_way {
                        InsertWay::Vertical => current_y += new_height,
                        InsertWay::Horizontal => current_x += new_width,
                    }
                }
            }
        }
    }

    pub fn delete<F>(&mut self, target: Id, f: &mut F) -> Result<()>
    where
        F: DispatchCallback<T>,
    {
        let fit_way = match self {
            Self::Vertical { .. } => InsertWay::Vertical,
            // NOTE: it will only be used in pattern three, so,
            // this part will always be Self::Horizontal
            _ => InsertWay::Horizontal,
        };
        match self {
            Self::EmptyOutput(_) => Err(Error),
            // NOTE: this logic only comes when there is only one window exist
            Self::Window { id, size_pos, .. } => {
                if *id != target {
                    return Err(Error);
                }
                *self = Self::EmptyOutput(*size_pos);
                Ok(())
            }
            Self::Vertical {
                elements,
                percent,
                size_pos,
            }
            | Self::Horizontal {
                elements,
                percent,
                size_pos,
            } => {
                let mut position: Option<usize> = None;
                let mut window_s_a_p: Option<SizeAndPos<T>> = None;
                // We use the percent to record how much the size is expand
                // if it is +, means it expand
                // else it shrink
                let mut target_percent: Option<Size> = None;
                for (index, element) in elements.iter_mut().enumerate() {
                    if let Self::Window {
                        id,
                        size_pos,
                        percent,
                    } = element
                        && *id == target
                    {
                        position = Some(index);
                        window_s_a_p = Some(*size_pos);
                        target_percent = Some(*percent);
                        break;
                    }
                    if element.delete(target, f).is_ok() {
                        return Ok(());
                    }
                }
                let (Some(pos), Some(disappear_info), Some(target_percent)) =
                    (position, window_s_a_p, target_percent)
                else {
                    return Err(Error);
                };
                elements.remove(pos);

                let start = pos == 0;
                let adjust_pos = if start { 0 } else { pos - 1 };
                let expand_way = ReMapDirection::expend_way(fit_way, start);

                let element = &mut elements[adjust_pos];

                // use the direction to calculate the diff change
                let changed_size_and_pos = disappear_info.change_disappear(expand_way);
                element.expand(
                    changed_size_and_pos,
                    target_percent.change_expand(fit_way),
                    f,
                );
                // NOTE: now we need a new function to expand the element.
                // And we need a enum contains four fields
                let _ = element;
                // Then we remove the deleted guy

                // it the element only one existed, downgrade it
                if elements.len() == 1 {
                    let o_percent = *percent;
                    let o_size_pos = *size_pos;
                    // first, we clone all the information in the element[0]
                    *self = elements[0].clone();
                    // Since it means it replace all of the information, then the size and position
                    // of container won't change
                    // So we just give these to it
                    self.set_percentage(o_percent);
                    self.set_size_and_pos(o_size_pos);
                }

                Ok(())
            }
        }
    }

    /// The return shows the new inserted position. it should be saved. but you can know it during
    /// the result show if the operation is succeeded
    pub fn insert<F>(&mut self, id: Id, target: Id, way: InsertWay, f: &mut F) -> Result<()>
    where
        F: DispatchCallback<T>,
    {
        let fit_way = match self {
            Self::Vertical { .. } => InsertWay::Vertical,
            // NOTE: it will only be used in pattern three, so,
            // this part will always be Self::Horizontal
            _ => InsertWay::Horizontal,
        };
        match self {
            Self::EmptyOutput(size) => {
                f.callback(id, *size);
                *self = Self::Window {
                    id,
                    size_pos: *size,
                    percent: Size {
                        width: 1.,
                        height: 1.,
                    },
                };
                Ok(())
            }
            Self::Window {
                size_pos,
                id: o_id,
                percent,
            } => {
                if *o_id != target {
                    return Err(Error);
                }
                let origin_size_pos = *size_pos;
                let new_size_pos = size_pos.split(way);
                let old_percent = *percent;
                f.callback(*o_id, *size_pos);
                f.callback(id, new_size_pos);
                // NOTE: because it is will not become a window anymore, it will be the half of the
                // Vertical or Horizontal
                let new_percent = Size::whole().split(2., way);
                *percent = new_percent;
                let elements = vec![
                    self.clone(),
                    ElementMap::Window {
                        id,
                        size_pos: new_size_pos,
                        percent: new_percent,
                    },
                ];
                *self = match way {
                    InsertWay::Vertical => ElementMap::Vertical {
                        elements,
                        size_pos: origin_size_pos,
                        percent: old_percent,
                    },
                    InsertWay::Horizontal => ElementMap::Horizontal {
                        elements,
                        size_pos: origin_size_pos,
                        percent: old_percent,
                    },
                };
                Ok(())
            }
            Self::Vertical { elements, .. } | Self::Horizontal { elements, .. } => {
                let mut to_insert_index: Option<usize> = None;
                let mut to_return: Option<SizeAndPos<T>> = None;
                let mut new_percent: Option<Size> = None;
                for (index, element) in elements.iter_mut().enumerate() {
                    if let ElementMap::Window {
                        id: o_id,
                        size_pos,
                        percent,
                    } = element
                        && *o_id == target
                    {
                        if way == fit_way {
                            let new_size_pos = size_pos.split(way);
                            *percent = percent.split(2., way);
                            to_return = Some(new_size_pos);
                            to_insert_index = Some(index);
                            new_percent = Some(*percent);
                            break;
                        }
                        return element.insert(id, target, way, f);
                    }
                    let insert_result = element.insert(id, target, way, f);
                    if insert_result.is_ok() {
                        return Ok(());
                    }
                }
                if let (Some(index), Some(to_return), Some(percent)) =
                    (to_insert_index, to_return, new_percent)
                {
                    let size_pos = to_return;
                    let window = Self::Window {
                        id,
                        size_pos,
                        percent,
                    };
                    elements.insert(index + 1, window);
                    return Ok(());
                }
                Err(Error)
            }
        }
    }
}
