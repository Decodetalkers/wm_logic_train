mod utils;

use std::hash::Hash;
use std::sync::atomic::{self, AtomicU64};

use crate::utils::{InsertWay, MapUnit, Position, Size, SizeAndPos};

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
enum ElementMap<T = f32>
where
    T: MapUnit,
{
    Empty,
    Window { id: Id, size_pos: SizeAndPos<T> },
    Vertical { elements: Vec<ElementMap<T>> },
    Horizontal { elements: Vec<ElementMap<T>> },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ReMapDirection {
    Left,
    Right,
    Top,
    Bottom,
}

impl ReMapDirection {
    fn expend_way(insert_way: InsertWay, start: bool) -> Self {
        match (insert_way, start) {
            (InsertWay::Horizontal, true) => Self::Left,
            (InsertWay::Horizontal, false) => Self::Right,
            (InsertWay::Vertical, true) => Self::Top,
            (InsertWay::Vertical, false) => Self::Bottom,
        }
    }
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

impl<T: MapUnit> ElementMap<T> {
    fn new() -> Self {
        Self::Empty
    }
    fn id(&self) -> Option<Id> {
        match self {
            Self::Window { id, .. } => Some(*id),
            _ => None,
        }
    }
    pub fn size(&self) -> Size<T> {
        match self {
            Self::Empty => Size {
                width: T::zero(),
                height: T::zero(),
            },
            Self::Window { size_pos, .. } => size_pos.size,
            Self::Vertical { elements } => {
                let width = elements[0].width();
                let height = elements.iter().map(|w| w.height()).sum();
                Size { width, height }
            }
            Self::Horizontal { elements } => {
                let height = elements[0].height();
                let width = elements.iter().map(|w| w.width()).sum();
                Size { width, height }
            }
        }
    }

    // NOTE: how to design it? what should I do with the size_pos? how does it mean?
    // maybe I need minus
    fn expand<F>(&mut self, direction: ReMapDirection, size_pos: SizeAndPos<T>, f: &mut F)
    where
        F: InsertCallback<T>,
    {
        match self {
            Self::Empty => {}
            Self::Window { size_pos, .. } => {}
            _ => todo!(),
        }
    }

    pub fn width(&self) -> T {
        match self {
            Self::Empty => T::two(),
            Self::Window { size_pos, .. } => size_pos.size.width,
            Self::Vertical { elements } => elements[0].width(),
            Self::Horizontal { elements } => elements.iter().map(|w| w.width()).sum(),
        }
    }

    pub fn height(&self) -> T {
        match self {
            Self::Empty => T::two(),
            Self::Window { size_pos, .. } => size_pos.size.height,
            Self::Vertical { elements } => elements[0].height(),
            Self::Horizontal { elements } => elements.iter().map(|w| w.height()).sum(),
        }
    }

    fn is_window(&self) -> bool {
        matches!(self, Self::Window { .. })
    }
    fn has_id(&self, target: Id) -> bool {
        match self {
            Self::Window { id, .. } => *id == target,
            Self::Empty => true,
            Self::Vertical { elements } | Self::Horizontal { elements } => {
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
            Self::Empty => None,
            Self::Window { id, .. } => (*id == target).then(|| self),
            Self::Vertical { elements } | Self::Horizontal { elements } => {
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
    fn delete<F>(&mut self, target: Id, f: &mut F) -> bool
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
            Self::Empty => false,
            // NOTE: this logic only comes when there is only one window exist
            Self::Window { id, .. } => {
                if *id != target {
                    return false;
                }
                *self = Self::Empty;
                true
            }
            Self::Vertical { elements } | Self::Horizontal { elements } => {
                let mut position: Option<usize> = None;
                let mut window_s_a_p: Option<SizeAndPos<T>> = None;
                for (index, element) in elements.iter_mut().enumerate() {
                    if let Self::Window { id, size_pos } = element
                        && *id == target
                    {
                        position = Some(index);
                        window_s_a_p = Some(*size_pos);
                        break;
                    }
                    if element.delete(target, f) {
                        return true;
                    }
                }
                let (Some(pos), Some(s_a_p)) = (position, window_s_a_p) else {
                    return false;
                };
                let start = pos == 0;
                let adjust_pos = if start { 1 } else { pos - 1 };
                let expand_way = ReMapDirection::expend_way(fit_way, start);

                let element = &mut elements[adjust_pos];
                element.expand(expand_way, s_a_p, f);
                // NOTE: now we need a new function to expand the element.
                // And we need a enum contains four fields
                let _ = element;
                // Then we remove the deleted guy
                elements.remove(pos);

                // it the element only one existed, downgrade it
                if elements.len() == 1 {
                    *self = elements[0].clone();
                }

                todo!()
            }
        }
    }

    /// The return shows the new inserted position. it should be saved. but you can know it during
    /// the result show if the operation is succeeded
    #[must_use]
    pub fn insert<F>(
        &mut self,
        id: Id,
        target: Id,
        way: InsertWay,
        output_size: SizeAndPos<T>,
        f: &mut F,
    ) -> bool
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
            Self::Empty => {
                let size = output_size;
                f.callback(id, size);
                *self = Self::Window { id, size_pos: size };
                true
            }
            Self::Window { size_pos, id: o_id } => {
                if *o_id != target {
                    return false;
                }
                let new_size_pos = size_pos.split(way);
                f.callback(*o_id, *size_pos);
                f.callback(id, new_size_pos);
                let elements = vec![
                    self.clone(),
                    ElementMap::Window {
                        id,
                        size_pos: new_size_pos,
                    },
                ];
                *self = match way {
                    InsertWay::Vertical => ElementMap::Vertical { elements },
                    InsertWay::Horizontal => ElementMap::Horizontal { elements },
                };
                true
            }
            Self::Vertical { elements } | Self::Horizontal { elements } => {
                let mut to_insert_index: Option<usize> = None;
                let mut to_return: Option<SizeAndPos<T>> = None;
                for (index, element) in elements.iter_mut().enumerate() {
                    if let ElementMap::Window { id: o_id, size_pos } = element
                        && *o_id == target
                    {
                        if way == fit_way {
                            let new_size_pos = size_pos.split(way);
                            to_return = Some(new_size_pos);
                            to_insert_index = Some(index);
                            break;
                        }
                        return element.insert(id, target, way, output_size, f);
                    }
                    let insert_result = element.insert(id, target, way, output_size, f);
                    if insert_result {
                        return insert_result;
                    }
                }
                if let (Some(index), Some(to_return)) = (to_insert_index, to_return) {
                    let size_pos = to_return.clone();
                    let window = Self::Window { id, size_pos };
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
    let mut element_map = ElementMap::<f32>::new();
    dbg!(&element_map);
    let _ = element_map.insert(
        Id::unique(),
        Id::MAIN,
        InsertWay::Vertical,
        DISPLAY_SIZE,
        &mut |id, size| {
            abc += 1;
        },
    );
    println!("{abc}");
    dbg!(&element_map);
    let _ = element_map.insert(
        Id::unique(),
        Id(0),
        InsertWay::Vertical,
        DISPLAY_SIZE,
        &mut |id, size| {},
    );
    dbg!(&element_map);
    //element_map.insert(Id::unique(), Id(0), InsertWay::Horizontal);
    //dbg!(&element_map);
}
