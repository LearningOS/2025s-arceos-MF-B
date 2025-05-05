use allocator::{AllocResult, BaseAllocator, ByteAllocator};
use core::alloc::Layout;
use core::ptr::NonNull;
use allocator::AllocError;
pub struct Align8ByteAllocator {
    turn: u8, // 0: left, 1: right
    pub start: usize,
    pub end: usize,
    left: usize,
    left_count: usize,
    right: usize,
    right_count: usize,
}

impl Align8ByteAllocator {
    pub const fn new() -> Self {
        Self {
            turn: 0,
            start: 0,
            end: 0,
            left: 0,
            left_count: 0,
            right: 0,
            right_count: 0,
        }
    }
}

impl BaseAllocator for Align8ByteAllocator {
    fn init(&mut self, start: usize, size: usize) {
        self.start = start;
        self.end = start + size;
        self.left = start;
        self.right = start + size;
        self.left_count = 0;
        self.right_count = 0;
    }

    fn add_memory(&mut self, start: usize, size: usize) -> AllocResult {
        if self.end == 0 {
            self.start = start;
            self.left = start;
            self.right = start + size;
            self.end = start + size;
        }else if self.end == start && self.right == start {
            self.end = start + size;
            self.right = start + size;
        } else {
            unimplemented!()
        }
        Ok(())
    }
}

impl ByteAllocator for Align8ByteAllocator {
    fn alloc(&mut self, layout: Layout) -> AllocResult<NonNull<u8>> {
        match self.turn {
            0 => {
                if self.left + layout.size() > self.right {
                    panic!("Memory leak");
                }
                let ptr = NonNull::new(self.left as *mut u8).ok_or(AllocError::NoMemory)?;
                self.left += layout.size();
                self.left_count += 1;
                self.turn = 1;
                Ok(ptr)
            }
            1 => {
                if self.right - layout.size() < self.left {
                    panic!("Memory leak");
                }
                self.right -= layout.size();
                let ptr = NonNull::new(self.right as *mut u8).ok_or(AllocError::NoMemory)?;
                self.right_count += 1;
                self.turn = 0;
                Ok(ptr)
            }
            _ => unreachable!(),
            
        }
    }

    fn dealloc(&mut self, _pos: NonNull<u8>, _layout: Layout) {
        match self.turn {
            0 => {
                self.left_count -= 1;
                if self.left_count == 0 {
                    self.left = self.start;
                }
            }
            1 => {
                self.right_count -= 1;
                if self.right_count == 0 {
                    self.right = self.end;
                }
            }
            _ => unreachable!(),
            
        }
    }

    fn total_bytes(&self) -> usize {
        self.end - self.start
    }

    fn used_bytes(&self) -> usize {
        self.left - self.start + self.end - self.right
    }

    fn available_bytes(&self) -> usize {
        self.end - self.left + self.right - self.start
    }
}