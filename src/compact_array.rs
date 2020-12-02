use core::ptr;
use std::alloc::{AllocRef, Global, Layout};
use std::fmt::{Debug, Formatter};
use std::fmt;
use std::ops::Index;
use std::ptr::{NonNull, Unique};

// 196 MB => 1.2GB (times 6.12)
// 196 MB => 491MB = 2.5
// => 429MB = 2.1GB .. after 215 (with f32)
// what to do... graph compression
#[repr(packed)]
pub struct CompactVec<T> {
    len: u8,
    // 1
    ptr: Unique<T>, // 4
}

impl<T: Debug> Debug for CompactVec<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut x = f.debug_struct("CompactVec");
        x.field("len", &self.len);

        for i in 0..self.len {
            let elem = &self[i];
            x.field("0", elem);
        }
        x.finish()
    }
}

pub struct CompactVecIterator<'a, T: 'static> {
    compact_vec: &'a CompactVec<T>,
    idx: u8
}

impl <'a, T> Iterator for CompactVecIterator<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let len = self.compact_vec.len;
        let idx = self.idx;
        if idx < len {
            let res = &self.compact_vec[idx];
            self.idx+=1;
            return Some(res)
        }
        None
    }
}

impl<T> CompactVec<T> {

    pub fn len(&self) -> u8 {
        self.len
    }

    pub fn iterator(&self) -> CompactVecIterator<T> {
        CompactVecIterator {
            compact_vec: self,
            idx: 0
        }
    }

    pub fn push(&mut self, elem: T){
        self.add_len(1);
        self.insert(self.len - 1, elem);
    }

    pub fn push2(&mut self, elem1: T, elem2: T){
        self.add_len(2);
        self.insert(self.len - 2, elem1);
        self.insert(self.len - 1, elem2);
    }

    pub fn empty() -> CompactVec<T> {
        CompactVec {
            len: 0,
            ptr: Unique::dangling(), // lazily allocate
        }
    }

    pub fn add_len(&mut self, len: u8) {
        if len == 0 {
            return;
        }
        let new_len = self.len + len;
        let new_layout = Layout::array::<T>(new_len as usize).unwrap();
        let old_len = self.len as usize;
        let ptr = if old_len == 0 {
            Global.alloc(new_layout)
        } else {
            // init alloc
            let old_layout = Layout::array::<T>(old_len).unwrap();
            let c: NonNull<T> = self.ptr.into();
            unsafe {
                Global.grow(c.cast(), old_layout, new_layout)
            }
        }.unwrap();

        self.ptr = unsafe {
            Unique::new_unchecked(ptr.as_ptr() as *mut _)
        };

        self.len = new_len;
    }

    #[inline]
    fn ptr(&mut self) -> *mut T {
        self.ptr.as_ptr()
    }

    pub fn insert(&mut self, index: u8, value: T) {
        let ptr = self.ptr();
        unsafe {
            ptr::write(ptr.offset(index as isize), value);
        }
    }

    #[allow(dead_code)]
    pub fn append(&mut self, vec: Vec<T>) {
        if vec.is_empty() {
            return; // don't append anything
        }
        self.add_len(vec.len() as u8);

        let unique_ptr = self.ptr();

        // write values
        vec.into_iter().enumerate().for_each(|(i, elem)| unsafe {
            ptr::write(unique_ptr.add(i), elem);
        });
    }
}

impl<T> Index<u8> for CompactVec<T> {
    type Output = T;

    fn index(&self, idx: u8) -> &Self::Output {
        assert!(idx < self.len);
        let ptr = self.ptr.as_ptr();
        unsafe {
            let elem = ptr.offset(idx as isize);
            &*elem
        }
    }
}

impl<T> Drop for CompactVec<T> {
    fn drop(&mut self) {
        if self.len == 0 {
            return;
        }
        unsafe {
            let c: NonNull<T> = self.ptr.into();
            Global.dealloc(c.cast(), Layout::array::<T>(self.len as usize).unwrap());
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::compact_array::CompactVec;

    #[test]
    fn vec_append() {
        let mut compact_vec = CompactVec::empty();
        let vec = vec![1u8, 2u8];
        compact_vec.append(vec);

        assert_eq!(1, compact_vec[0]);
        assert_eq!(2, compact_vec[1]);
    }

    #[test]
    fn len_add() {
        let mut compact_vec = CompactVec::empty();

        compact_vec.add_len(2);
        compact_vec.insert(0, 0);
        compact_vec.insert(1, 1);

        compact_vec.add_len(2);
        compact_vec.insert(2, 2);
        compact_vec.insert(3, 3);

        assert_eq!(0, compact_vec[0]);
        assert_eq!(1, compact_vec[1]);
        assert_eq!(2, compact_vec[2]);
        assert_eq!(3, compact_vec[3]);
    }
}
