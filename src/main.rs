use std::cell::RefCell;
use std::hash::Hash;
use std::rc::Rc;
use std::sync::atomic::{self, AtomicU64};

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
#[derive(Debug, Clone, Copy)]
struct SizeAndPos<T = u32>
where
    T: Copy,
{
    width: T,
    height: T,
    x: T,
    y: T,
}

macro_rules! impl_size_and_pos {
    ($Type: ident) => {
        impl SizeAndPos<$Type> {
            fn vertical(&mut self) -> Self {
                let width = self.width / 2;
                let height = self.height / 2;
                self.width = width;
                self.height = height;
                let y = self.y + height;
                Self {
                    width,
                    height,
                    x: self.x,
                    y,
                }
            }
            fn horizontal(&mut self) -> Self {
                let width = self.width / 2;
                let height = self.height / 2;
                self.width = width;
                self.height = height;
                let x = self.x + width;
                Self {
                    width,
                    height,
                    x,
                    y: self.y,
                }
            }
            fn split(&mut self, way: InsertWay) -> Self {
                match way {
                    InsertWay::Vertical => self.vertical(),
                    InsertWay::Horizontal => self.horizontal(),
                }
            }
        }
    };
}

impl_size_and_pos!(u32);
//impl_size_and_pos!(i32);

#[derive(Debug, Clone)]
enum ElementInfo {
    Empty,
    Window {
        id: Id,
        size_pos: Rc<RefCell<SizeAndPos>>,
    },
    Vertical {
        elements: Vec<ElementInfo>,
    },
    Horizontal {
        elements: Vec<ElementInfo>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InsertWay {
    Vertical,
    Horizontal,
}

trait InsertCallback {
    fn callback(&mut self, id: Id, s_a_p: Rc<RefCell<SizeAndPos>>);
}

impl<F> InsertCallback for F
where
    F: FnMut(Id, Rc<RefCell<SizeAndPos>>),
{
    fn callback(&mut self, id: Id, s_a_p: Rc<RefCell<SizeAndPos>>) {
        self(id, s_a_p)
    }
}

impl ElementInfo {
    fn new() -> Self {
        Self::Empty
    }
    fn id(&self) -> Option<Id> {
        match self {
            Self::Window { id, .. } => Some(*id),
            _ => None,
        }
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
    fn delete(&mut self, id: Id) {}

    /// The return shows the new inserted position. it should be saved. but you can know it during
    /// callback
    fn insert<F>(
        &mut self,
        id: Id,
        target: Id,
        way: InsertWay,
        f: &mut F,
    ) -> Option<Rc<RefCell<SizeAndPos>>>
    where
        F: InsertCallback,
    {
        let fit_way = match self {
            Self::Vertical { .. } => InsertWay::Vertical,
            // NOTE: it will only be used in pattern three, so,
            // this part will always be Self::Horizontal
            _ => InsertWay::Horizontal,
        };
        match self {
            Self::Empty => {
                let size = Rc::new(RefCell::new(DISPLAY_SIZE));
                let size_r = size.clone();
                f.callback(id, size.clone());
                *self = Self::Window { id, size_pos: size };
                Some(size_r)
            }
            Self::Window { size_pos, id: o_id } => {
                if *o_id != target {
                    return None;
                }
                let new_size_pos = Rc::new(RefCell::new(size_pos.borrow_mut().split(way)));
                f.callback(*o_id, size_pos.clone());
                let new_size_pos_r = new_size_pos.clone();
                f.callback(id, new_size_pos.clone());
                let elements = vec![
                    self.clone(),
                    ElementInfo::Window {
                        id,
                        size_pos: new_size_pos,
                    },
                ];
                *self = match way {
                    InsertWay::Vertical => ElementInfo::Vertical { elements },
                    InsertWay::Horizontal => ElementInfo::Horizontal { elements },
                };

                Some(new_size_pos_r)
            }
            Self::Vertical { elements } | Self::Horizontal { elements } => {
                let mut to_insert_index: Option<usize> = None;
                let mut to_return: Option<Rc<RefCell<SizeAndPos>>> = None;
                for (index, element) in elements.iter_mut().enumerate() {
                    if let ElementInfo::Window { id: o_id, size_pos } = element
                        && *o_id == target
                    {
                        if way == fit_way {
                            let new_size_pos =
                                Rc::new(RefCell::new(size_pos.borrow_mut().split(way)));
                            to_return = Some(new_size_pos);
                            to_insert_index = Some(index);
                            break;
                        }
                        return element.insert(id, target, way, f);
                    }
                    let insert_result = element.insert(id, target, way, f);
                    if insert_result.is_some() {
                        return insert_result;
                    }
                }
                if let (Some(index), Some(to_return)) = (to_insert_index, to_return) {
                    let size_pos = to_return.clone();
                    let window = ElementInfo::Window { id, size_pos };
                    elements.insert(index + 1, window);
                    return Some(to_return);
                }
                None
            }
        }
    }
}

const DISPLAY_SIZE: SizeAndPos = SizeAndPos {
    width: 1980,
    height: 1080,
    x: 0,
    y: 0,
};

fn main() {
    let mut abc = 10;
    let mut element_map = ElementInfo::new();
    dbg!(&element_map);
    element_map.insert(
        Id::unique(),
        Id::MAIN,
        InsertWay::Vertical,
        &mut |id, size| {
            abc += 1;
        },
    );
    println!("{abc}");
    dbg!(&element_map);
    //element_map.insert(Id::unique(), Id(0), InsertWay::Vertical);
    //dbg!(&element_map);
    //element_map.insert(Id::unique(), Id(0), InsertWay::Horizontal);
    //dbg!(&element_map);
}
