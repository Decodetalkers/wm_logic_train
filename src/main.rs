mod utils;

use std::hash::Hash;
use std::sync::atomic::{self, AtomicU64};

use crate::utils::{
    InsertWay, MapUnit, MinusAbleMatUnit, Position, ReMapDirection, Size, SizeAndPos,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
/// The id of the window.
///
/// Internally Iced reserves `window::Id::MAIN` for the first window spawned.
pub struct Id(u64);

static COUNT: AtomicU64 = AtomicU64::new(0);

impl Id {
    /// The reserved window [`Id`] for the first window in an Iced application.
    pub const MAIN: Self = Id(0);

    /// Creates a new unique window [`Id`].
    pub fn unique() -> Id {
        Id(COUNT.fetch_add(1, atomic::Ordering::Relaxed))
    }
}

#[derive(Debug, Clone)]
enum ElementMap<T: MapUnit = f32> {
    EmptyOutput(SizeAndPos<T>),
    Window {
        id: Id,
        size_pos: SizeAndPos<T>,
        percent: Size<f32>,
    },
    Vertical {
        elements: Vec<ElementMap<T>>,
        size_pos: SizeAndPos<T>,
        percent: Size<f32>,
    },
    Horizontal {
        elements: Vec<ElementMap<T>>,
        size_pos: SizeAndPos<T>,
        percent: Size<f32>,
    },
}
trait InsertCallback<T: MapUnit> {
    fn callback(&mut self, id: Id, s_a_p: SizeAndPos<T>);
}

impl<F, T: MapUnit> InsertCallback<T> for F
where
    F: FnMut(Id, SizeAndPos<T>),
{
    fn callback(&mut self, id: Id, s_a_p: SizeAndPos<T>) {
        self(id, s_a_p)
    }
}

impl<T: MinusAbleMatUnit> ElementMap<T> {
    fn new(size_pos: SizeAndPos<T>) -> Self {
        Self::EmptyOutput(size_pos)
    }

    fn id(&self) -> Option<Id> {
        match self {
            Self::Window { id, .. } => Some(*id),
            _ => None,
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

    pub fn percent(&self) -> Size {
        match self {
            Self::Vertical { percent, .. }
            | Self::Horizontal { percent, .. }
            | Self::Window { percent, .. } => *percent,
            Self::EmptyOutput(_) => Size::whole(),
        }
    }

    pub fn size(&self) -> Size<T> {
        self.size_pos().size
    }

    // NOTE: how to design it? what should I do with the size_pos? how does it mean?
    // maybe I need minus
    fn expand<F>(&mut self, change: SizeAndPos<T>, diff_percent: Size, f: &mut F)
    where
        F: InsertCallback<T>,
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
                f.callback(*id, *size_pos);
            }
            // NOTE: this time only change the percent of the top, and the size of the emelemts
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
                    let diff_change = s_and_p - new_s_and_p;
                    element.expand(diff_change, Size::zero(), f);
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
                    let diff_change = s_and_p - new_s_and_p;
                    element.expand(diff_change, Size::zero(), f);
                }
            }
        }
    }

    pub fn width(&self) -> T {
        self.size().width
    }

    pub fn set_percentage(&mut self, c_percent: Size) {
        match self {
            Self::EmptyOutput(_) => {}
            Self::Vertical { percent, .. }
            | Self::Horizontal { percent, .. }
            | Self::Window { percent, .. } => *percent = c_percent,
        }
    }

    pub fn set_size_and_pos(&mut self, c_size_pos: SizeAndPos<T>) {
        match self {
            Self::EmptyOutput(size_pos)
            | Self::Vertical { size_pos, .. }
            | Self::Horizontal { size_pos, .. }
            | Self::Window { size_pos, .. } => *size_pos = c_size_pos,
        }
    }

    pub fn height(&self) -> T {
        self.size().height
    }

    fn is_window(&self) -> bool {
        matches!(self, Self::Window { .. })
    }
    fn has_id(&self, target: Id) -> bool {
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

    // NOTE: not just find it, but return the insert position
    fn find_element_mut(&mut self, target: Id) -> Option<&mut Self> {
        match self {
            Self::EmptyOutput(_) => None,
            Self::Window { id, .. } => (*id == target).then(|| self),
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

    #[must_use]
    pub fn delete<F>(&mut self, target: Id, f: &mut F) -> bool
    where
        F: InsertCallback<T>,
    {
        let fit_way = match self {
            Self::Vertical { .. } => InsertWay::Vertical,
            // NOTE: it will only be used in pattern three, so,
            // this part will always be Self::Horizontal
            _ => InsertWay::Horizontal,
        };
        match self {
            Self::EmptyOutput(_) => false,
            // NOTE: this logic only comes when there is only one window exist
            Self::Window { id, size_pos, .. } => {
                if *id != target {
                    return false;
                }
                *self = Self::EmptyOutput(*size_pos);
                true
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
                    if element.delete(target, f) {
                        return true;
                    }
                }
                let (Some(pos), Some(s_a_p), Some(target_percent)) =
                    (position, window_s_a_p, target_percent)
                else {
                    return false;
                };
                let start = pos == 0;
                let adjust_pos = if start { 1 } else { pos - 1 };
                let expand_way = ReMapDirection::expend_way(fit_way, start);

                let element = &mut elements[adjust_pos];
                let changed_size_and_pos = s_a_p.expend_change(expand_way);
                element.expand(
                    changed_size_and_pos,
                    target_percent.change_expand(fit_way),
                    f,
                );
                // NOTE: now we need a new function to expand the element.
                // And we need a enum contains four fields
                let _ = element;
                // Then we remove the deleted guy
                elements.remove(pos);

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

                true
            }
        }
    }

    /// The return shows the new inserted position. it should be saved. but you can know it during
    /// the result show if the operation is succeeded
    #[must_use]
    pub fn insert<F>(&mut self, id: Id, target: Id, way: InsertWay, f: &mut F) -> bool
    where
        F: InsertCallback<T>,
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
                true
            }
            Self::Window {
                size_pos,
                id: o_id,
                percent,
            } => {
                if *o_id != target {
                    return false;
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
                true
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
                    if insert_result {
                        return insert_result;
                    }
                }
                if let (Some(index), Some(to_return), Some(percent)) =
                    (to_insert_index, to_return, new_percent)
                {
                    let size_pos = to_return.clone();
                    let window = Self::Window {
                        id,
                        size_pos,
                        percent,
                    };
                    elements.insert(index + 1, window);
                    return true;
                }
                return false;
            }
        }
    }
}

const DISPLAY_SIZE: SizeAndPos = SizeAndPos {
    size: Size {
        width: 1980.,
        height: 1080.,
    },
    position: Position { x: 0., y: 0. },
};

fn main() {
    let mut abc = 10;
    let mut element_map = ElementMap::new(DISPLAY_SIZE);
    dbg!(&element_map);
    let _ = element_map.insert(
        Id::unique(),
        Id::MAIN,
        InsertWay::Vertical,
        &mut |id, size| {
            abc += 1;
        },
    );
    dbg!(&element_map);
    let _ = element_map.insert(Id::unique(), Id(0), InsertWay::Vertical, &mut |id, size| {});
    dbg!(&element_map);
    let _ = element_map.insert(
        Id::unique(),
        Id(0),
        InsertWay::Horizontal,
        &mut |id, size| {},
    );
    dbg!(&element_map);

    println!("=== delete ===");
    let _ = element_map.delete(Id(0), &mut |id, size| {});
    dbg!(&element_map);
    let _ = element_map.delete(Id(1), &mut |id, size| {});
    dbg!(&element_map);
}
