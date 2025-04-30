#![no_std]

use core::alloc::Layout;

use allocator::{BaseAllocator, ByteAllocator, PageAllocator};
/// Early memory allocator
/// Use it before formal bytes-allocator and pages-allocator can work!
/// This is a double-end memory range:
/// - Alloc bytes forward
/// - Alloc pages backward
///
/// [ bytes-used | avail-area | pages-used ]
/// |            | -->    <-- |            |
/// start       b_pos        p_pos       end
///
/// For bytes area, 'count' records number of allocations.
/// When it goes down to ZERO, free bytes-used area.
/// For pages area, it will never be freed!
///
pub struct EarlyAllocator<const PAGE_SIZE: usize> {
    start: usize,
    end: usize,
    byte_pos: usize,
    byte_allocations: usize,
    page_pos: usize,
    page_allocations: usize,
}
impl<const PAGE_SIZE: usize> EarlyAllocator<PAGE_SIZE> {
    /// Creates a new empty `EarlyAllocator`.
    pub const fn new() -> Self {
        Self {
            start: 0,
            end: 0,
            byte_pos: 0,
            byte_allocations: 0,
            page_pos: 0,
            page_allocations: 0,
        }
    }
}

impl<const PAGE_SIZE: usize> BaseAllocator for EarlyAllocator<PAGE_SIZE> {
    fn init(&mut self, start: usize, size: usize) {
        assert!(PAGE_SIZE.is_power_of_two());
        self.start = start;
        self.end = start + size;
        self.byte_pos = start;
        self.page_pos = start + size;
        self.byte_allocations = 0;
        self.page_allocations = 0;
    }

    fn add_memory(&mut self, _start: usize, _size: usize) -> allocator::AllocResult {
        Err(allocator::AllocError::NoMemory) // unsupported
    }
}

impl<const PAGE_SIZE: usize> ByteAllocator for EarlyAllocator<PAGE_SIZE> {
    fn alloc(
        &mut self,
        layout: core::alloc::Layout,
    ) -> allocator::AllocResult<core::ptr::NonNull<u8>> {
        let size = layout.size();
        let align = layout.align();
        if size == 0 || align == 0 {
            return Err(allocator::AllocError::InvalidParam);
        }
        let aligned_start = align_up(self.byte_pos, layout); // 只对其前端
        if aligned_start + size > self.page_pos {
            return Err(allocator::AllocError::NoMemory);
        }
        self.byte_allocations += 1;
        self.byte_pos += aligned_start + size;
        Ok(core::ptr::NonNull::new(aligned_start as *mut u8).unwrap())
    }

    fn dealloc(&mut self, _pos: core::ptr::NonNull<u8>, _layout: core::alloc::Layout) {
        self.byte_allocations -= 1;
        if self.byte_allocations == 0 {
            self.byte_pos = self.start;
        }
    }

    fn total_bytes(&self) -> usize {
        self.end - self.start
    }

    fn used_bytes(&self) -> usize {
        self.byte_pos - self.start
    }

    fn available_bytes(&self) -> usize {
        self.page_pos - self.byte_pos
    }
}

impl<const PAGE_SIZE: usize> PageAllocator for EarlyAllocator<PAGE_SIZE> {
    const PAGE_SIZE: usize = PAGE_SIZE;

    fn alloc_pages(
        &mut self,
        num_pages: usize,
        align_pow2: usize,
    ) -> allocator::AllocResult<usize> {
        if align_pow2 % PAGE_SIZE != 0 {
            return Err(allocator::AllocError::InvalidParam);
        }
        let align_pow2 = align_pow2 / PAGE_SIZE;
        let next = self.page_pos - align_pow2 * num_pages;
        if next < self.byte_pos {
            return Err(allocator::AllocError::NoMemory);
        }
        self.page_allocations += 1;
        self.page_pos = next;
        Ok(next)
    }

    fn dealloc_pages(&mut self, _pos: usize, _num_pages: usize) {
        self.page_allocations -= 1;
        if self.page_allocations == 0 {
            self.page_pos = self.end;
        }
    }

    fn total_pages(&self) -> usize {
        (self.end - self.byte_pos) / PAGE_SIZE
    }

    fn used_pages(&self) -> usize {
        (self.end - self.page_pos) / PAGE_SIZE
    }

    fn available_pages(&self) -> usize {
        (self.page_pos - self.byte_pos) / PAGE_SIZE
    }
}

pub fn align_up(addr: usize, layout: Layout) -> usize {
    let align = layout.align();
    ((addr + align) / align) * align
}

pub fn align_down(addr: usize, layout: Layout) -> usize {
    let align = layout.align();
    ((addr + align) / align) * (align - 1)
}