use arc_swap::strategy::CaS;
use arc_swap::{ArcSwapAny, AsRaw, Guard, RefCnt};
use std::ptr;

pub trait ArcSwapExt<T, S> {
    fn try_rcu<R, F, E>(&self, f: F) -> Result<T, E>
    where
        F: FnMut(&T) -> Result<R, E>,
        R: Into<T>;
}

impl<T, S> ArcSwapExt<T, S> for ArcSwapAny<T, S>
where
    T: RefCnt,
    S: CaS<T>,
{
    fn try_rcu<R, F, E>(&self, mut f: F) -> Result<T, E>
    where
        F: FnMut(&T) -> Result<R, E>,
        R: Into<T>,
    {
        let mut cur = self.load();

        loop {
            let new = f(&cur)?.into();
            let prev = self.compare_and_swap(&*cur, new);
            let swapped = ptr_eq(&*cur, &*prev);

            if swapped {
                return Ok(Guard::into_inner(prev));
            } else {
                cur = prev;
            }
        }
    }
}

fn ptr_eq<Base, A, B>(a: A, b: B) -> bool
where
    A: AsRaw<Base>,
    B: AsRaw<Base>,
{
    let a = a.as_raw();
    let b = b.as_raw();

    ptr::eq(a, b)
}
