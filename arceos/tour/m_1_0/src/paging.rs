use axhal::trap::{register_trap_handler, PAGE_FAULT};
use axhal::mem::VirtAddr;
use axhal::paging::MappingFlags;
use axtask::TaskExtRef;

#[register_trap_handler(PAGE_FAULT)]
fn handle_page_fault(vaddr: VirtAddr, access_flags: MappingFlags, is_user: bool) -> bool {
    let task = axtask::current();
    if is_user {
        let is_handle = task.task_ext().aspace.lock().handle_page_fault(vaddr, access_flags);
        return is_handle;
    }
    false
}
