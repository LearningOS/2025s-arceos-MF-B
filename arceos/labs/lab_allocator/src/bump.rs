use allocator::{AllocResult, BaseAllocator, ByteAllocator};
use core::alloc::Layout;
use core::ptr::NonNull;
use allocator::AllocError;
pub struct BumpByteAllocator {
    pub start: usize,
    pub end: usize,
    next: usize,
    count: usize,
}

impl BumpByteAllocator {
    pub const fn new() -> Self {
        Self {
            start: 0,
            end: 0,
            next: 0,
            count: 0,
        }
    }
}

impl BaseAllocator for BumpByteAllocator {
    fn init(&mut self, start: usize, size: usize) {
        self.start = start;
        self.end = start + size;
        self.next = start;
    }

    fn add_memory(&mut self, start: usize, size: usize) -> AllocResult {
        if self.end == start {
            self.end = start+size;
        } else if self.end == 0 {
            self.start = start;
            self.next = start;
            self.end = start + size;
        } else {
            panic!("Memory leak");
        }
        Ok(())
    }
}

impl ByteAllocator for BumpByteAllocator {
    fn alloc(&mut self, layout: Layout) -> AllocResult<NonNull<u8>> {
        if self.next + layout.size() > self.end {
            return Err(AllocError::NoMemory);
        }
        let ptr = NonNull::new(self.next as *mut u8).ok_or(AllocError::NoMemory)?;
        self.next += layout.size();
        self.count += 1;
        Ok(ptr)
    }

    fn dealloc(&mut self, _pos: NonNull<u8>, _layout: Layout) {
        self.count -= 1;
        if self.count == 0 {
            self.next = self.start;
        }
    }

    fn total_bytes(&self) -> usize {
        self.end - self.start
    }

    fn used_bytes(&self) -> usize {
        self.next - self.start
    }

    fn available_bytes(&self) -> usize {
        self.end - self.next
    }
}