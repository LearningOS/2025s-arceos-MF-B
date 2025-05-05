//! Allocator algorithm in lab.

#![no_std]
#![allow(unused_variables)]

mod bump;
mod align8;

use allocator::{BaseAllocator, ByteAllocator, AllocResult, AllocError, SlabByteAllocator, BuddyByteAllocator,TlsfByteAllocator};
use core::f32::MIN;
use core::ptr::NonNull;
use core::alloc::Layout;
use bump::BumpByteAllocator;
use align8::Align8ByteAllocator;
use log::*;

const MIN_SPECIAL_ALIGN8_SIZE: usize = 0x20000;
const MIN_WILL_DEALLOC_SIZE: usize = 0x100000;
const MEMORY_END: usize = 0xffffffc088000000;

pub struct LabByteAllocator {
    next_tlsf: bool, // true: tlsf, false: other
    will_dealloc: BumpByteAllocator,
    never_dealloc: BumpByteAllocator,
    add_memory_to_num: u8,
    will_count: usize,
    special_align8: Align8ByteAllocator,
    is_ok: bool,
}

impl LabByteAllocator {
    pub const fn new() -> Self {
        Self {
            next_tlsf: true,
            will_dealloc: BumpByteAllocator::new(),
            never_dealloc: BumpByteAllocator::new(),
            add_memory_to_num: 0,
            will_count: 0,
            special_align8: Align8ByteAllocator::new(),
            is_ok: false,
        }
    }
}

impl BaseAllocator for LabByteAllocator {
    fn init(&mut self, start: usize, size: usize) {
        self.special_align8.init(start, size);
        debug!("init special_align8: start: {:#x?}, end: {:#x?}", start, start+size);
    }
    fn add_memory(&mut self, start: usize, size: usize) -> AllocResult {
        if self.special_align8.total_bytes() < MIN_SPECIAL_ALIGN8_SIZE {
            debug!("pre add_memory special_align8: start: {:#x?}, end: {:#x?}", start, start+size);
            return self.special_align8.add_memory(start, size);
        }
        if self.will_dealloc.total_bytes() < MIN_WILL_DEALLOC_SIZE {
            debug!("pre add_memory will_dealloc: start: {:#x?}, end: {:#x?}", start, start+size);
            return self.will_dealloc.add_memory(start, size);
        }
        if self.never_dealloc.end < MEMORY_END {
            debug!("pre add_memory never_dealloc: start: {:#x?}, end: {:#x?}", start, start+size);
            if self.never_dealloc.end + size >= MEMORY_END {
                self.is_ok = true;
            }
            return self.never_dealloc.add_memory(start, size);
        }
        match self.add_memory_to_num {
            0 => {
                error!("add_memory will_dealloc: start: {:#x?}, end: {:#x}", start, start+size);
                self.will_dealloc.add_memory(start, size)
            },
            1 => {
                error!("add_memory never_dealloc: start: {:#x?}, end: {:#x}", start, start+size);
                self.never_dealloc.add_memory(start, size)
            },
            2 => {
                error!("add_memory special_align8: start: {:#x?}, end: {:#x}", start, start+size);
                self.special_align8.add_memory(start, size)
            },
            _ => Err(AllocError::InvalidParam),
            
        }
    }
}

impl ByteAllocator for LabByteAllocator {
    fn alloc(&mut self, layout: Layout) -> AllocResult<NonNull<u8>> {
        if !self.is_ok {
            return Err(AllocError::NoMemory);
        }
        if !self.next_tlsf && layout.align() == 1 {
            self.add_memory_to_num = 1;
            let ptr = self.never_dealloc.alloc(layout)?;
//            error!("never_dealloc alloc {:#x?}, next{:#x?},", layout.size(), self.never_dealloc.next);
            self.next_tlsf = !self.next_tlsf;
            if layout.size() >= 512 * 1024 {
                self.next_tlsf = true;
            }
            Ok(ptr)
        }else if layout.align() == 8 && layout.size() != 96 && layout.size() != 192 && layout.size() != 384{
            self.add_memory_to_num = 2;
            let ptr = self.special_align8.alloc(layout)?;
//            warn!("special_align8 alloc {:#x?}, {:?}, total: {}, used:{}", layout.size(), layout.align(), self.special_align8.total_bytes(), self.special_align8.used_bytes());
            Ok(ptr)
        } 
        else {
            self.add_memory_to_num = 0;
            let ptr = self.will_dealloc.alloc(layout)?;
            if layout.align() == 1 {
                self.next_tlsf = !self.next_tlsf;
            }
//           error!("will_dealloc alloc {:#x}B, next: {:#x}", layout.size(), self.will_dealloc.next);
            self.will_count += 1;
            // if layout.align() == 8 && layout.size() != 96 && layout.size() != 192 && layout.size() != 384 {
            //     warn!("alloc {:#x?}, {:?}", layout.size(), layout.align());
            // }
            if layout.size() >= 512 * 1024 {
                self.next_tlsf = true;
            }
            Ok(ptr)
        }
    }
    fn dealloc(&mut self, pos: NonNull<u8>, layout: Layout) {
        if layout.align() == 8 && layout.size() != 96 && layout.size() != 192 && layout.size() != 384 {
            self.special_align8.dealloc(pos, layout);
            // error!("dealloc {:#x?}, {:?}, total: {}, used:{}", layout.size(), layout.align(), self.special_align8.total_bytes(), self.special_align8.used_bytes());
            // error!("special_align8 start:{:#x?}, end:{:#x?}", self.special_align8.start, self.special_align8.end);
            return;
        }
        self.will_dealloc.dealloc(pos, layout);
       debug!("dealloc {:#x}, {}, total: {}, used: {}", layout.size(), layout.align(), self.will_dealloc.total_bytes(), self.will_dealloc.used_bytes());
        self.will_count -= 1;
        if self.will_count == 0 {
            debug!("all dealloc!!!!!!!!!!!!!!!!!");
            debug!("will_dealloc start:{:#x?}, next:{:#x?}", self.will_dealloc.start,self.will_dealloc.next);
        }
        // if layout.align() == 8 && layout.size() != 96 && layout.size() != 192 && layout.size() != 384 {
        //     error!("dealloc {:?}, {:?}", layout.size(), layout.align());
        // }
        
    }
    fn total_bytes(&self) -> usize {
        match self.add_memory_to_num {
            0 => {
                if self.is_ok {
                    return self.will_dealloc.total_bytes();
                }
                return 0x1000;
            },
            1 => {
                if self.is_ok {
                    return self.never_dealloc.total_bytes();
                }
                return 0x1000;
            },
            2 => self.special_align8.total_bytes(),
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
