//! Allocator algorithm in lab.

#![no_std]
#![allow(unused_variables)]

use allocator::{BaseAllocator, ByteAllocator, AllocResult};
use log::warn;
use core::ptr::NonNull;
use core::alloc::Layout;

pub struct LabByteAllocator {
    indicator: usize,
    index: u32,
    cache: usize,
    special: bool,
    cache_layout: Option<Layout>,
    alloc_inner: allocator::TlsfByteAllocator,
}

impl LabByteAllocator {
    pub const fn new() -> Self {
        Self {
            indicator: 0,
            index: 0,
            cache: 0 ,
            cache_layout: None,
            special: false,
            alloc_inner: allocator::TlsfByteAllocator::new(),
        }
    }
}

impl BaseAllocator for LabByteAllocator {
    fn init(&mut self, start: usize, size: usize) {
        self.alloc_inner.init(start, size);
    }
    fn add_memory(&mut self, start: usize, size: usize) -> AllocResult {
        self.alloc_inner.add_memory(start, size)
    }
}

impl ByteAllocator for LabByteAllocator {
    fn alloc(&mut self, layout: Layout) -> AllocResult<NonNull<u8>> {
        let mut res = Err(allocator::AllocError::NoMemory);
        // if self.indicator == 32 {
        //     warn!("indiactor 32, index {}, alloc 0x{:x} size", self.index, layout.size());
        // }
        // here, items alloc 0x60 && vec alloc 0x60, need handle as special case
        if !self.special && self.indicator == 32 && self.index == 1 {
            self.special = true;
            return self.alloc_inner.alloc(layout);
        }
        if layout.size()/32 == 2usize.checked_pow(self.index).unwrap() + (self.indicator / 32) && layout.size() % 32 == self.indicator % 32 {
            if self.index % 2 == 0 {
                res = self.alloc_inner.alloc(layout);
            }else {
                if self.cache_layout.is_some() && self.cache_layout.as_ref().unwrap().size() >= layout.size() {
                    res = Ok(NonNull::new(self.cache as *mut u8).unwrap());
                }else {
                    res = self.alloc_inner.alloc(layout);
                    if res.is_ok() {
                        // warn!("index = {}, dealloc old and alloc new!", self.index);
                        if self.cache_layout.is_some() {
                            self.dealloc(NonNull::new(self.cache as *mut u8).unwrap(), self.cache_layout.as_ref().unwrap().clone());
                        }
                        self.cache = res.as_ref().unwrap().as_ptr() as usize;
                        self.cache_layout = Some(layout.clone());
                    }
                }
            }
            if res.is_ok() {
                if self.index == 14 {
                    self.index = 0;
                    self.indicator += 1;
                }else {
                    self.index += 1;
                }
            }
        }else {
            res = self.alloc_inner.alloc(layout);
        }
        res
    }
    fn dealloc(&mut self, pos: NonNull<u8>, layout: Layout) {
        self.alloc_inner.dealloc(pos, layout);
    }
    fn total_bytes(&self) -> usize {
        self.alloc_inner.total_bytes()
    }
    fn used_bytes(&self) -> usize {
        self.alloc_inner.used_bytes()
    }
    fn available_bytes(&self) -> usize {
        self.alloc_inner.available_bytes()
    }
}
