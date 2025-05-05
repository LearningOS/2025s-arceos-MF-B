//! Allocator algorithm in lab.

#![no_std]
#![allow(unused_variables)]

mod bump;

use allocator::{BaseAllocator, ByteAllocator, AllocResult, AllocError, SlabByteAllocator};
use core::ptr::NonNull;
use core::alloc::Layout;
use bump::BumpByteAllocator;

pub struct LabByteAllocator {
    next_tlsf: bool, // true: tlsf, false: other
    will_dealloc: SlabByteAllocator,
    never_dealloc: BumpByteAllocator,
    add_memory_to_num: u8,
}

impl LabByteAllocator {
    pub const fn new() -> Self {
        Self {
            next_tlsf: true,
            will_dealloc: SlabByteAllocator::new(),
            never_dealloc: BumpByteAllocator::new(),
            add_memory_to_num: 0,
        }
    }
}

impl BaseAllocator for LabByteAllocator {
    fn init(&mut self, start: usize, size: usize) {
        self.will_dealloc.init(start, size);
    }
    fn add_memory(&mut self, start: usize, size: usize) -> AllocResult {
        // if self.will_dealloc.total_bytes() < MIN_SLAB_SIZE {
        //     return self.will_dealloc.add_memory(start, size);
        // }
        match self.add_memory_to_num {
            0 => {
                self.will_dealloc.add_memory(start, size)
            },
            1 => {
                self.never_dealloc.add_memory(start, size)
            },
            _ => Err(AllocError::InvalidParam),
            
        }
    }
}

impl ByteAllocator for LabByteAllocator {
    fn alloc(&mut self, layout: Layout) -> AllocResult<NonNull<u8>> {
        if !self.next_tlsf && layout.align() == 1 {
            self.add_memory_to_num = 1;
            let ptr = self.never_dealloc.alloc(layout)?;
            self.next_tlsf = !self.next_tlsf;
            if layout.size() >= 512 * 1024 {
                self.next_tlsf = true;
            }
            Ok(ptr)
        } else {
            self.add_memory_to_num = 0;
            let ptr = self.will_dealloc.alloc(layout)?;
            if layout.align() == 1 {
                self.next_tlsf = !self.next_tlsf;
            }
            if layout.size() >= 512 * 1024 {
                self.next_tlsf = true;
            }
            Ok(ptr)
        }
    }
    fn dealloc(&mut self, pos: NonNull<u8>, layout: Layout) {
        self.will_dealloc.dealloc(pos, layout);
    }
    fn total_bytes(&self) -> usize {
        match self.add_memory_to_num {
            0 => self.will_dealloc.total_bytes(),
            1 => self.never_dealloc.total_bytes(),
            _ => 0,
        }
    }
    fn used_bytes(&self) -> usize {
        todo!()
    }
    fn available_bytes(&self) -> usize {
        todo!()
    }
}
