use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use x86_64::{
    registers::control::Cr3,
    structures::paging::{
        FrameAllocator, Mapper, OffsetPageTable, Page, PageTable, PageTableFlags, PhysFrame,
        Size4KiB,
    },
    PhysAddr, VirtAddr,
};

/// A FrameAllocator that returns usable frames from the bootloader's memory map.
pub struct BootInfoFrameAllocator {
    memory_map: &'static MemoryMap,
    next: usize,
}

impl BootInfoFrameAllocator {
    /// Create a FrameAllocator from the passed memory map.
    ///
    /// # Safety
    /// This fn is unsafe because the caller must guarantee that the passed memory map is valid.
    /// The main requirement is that all frames that are marked as `USABLE` in it are really unused.
    ///
    pub unsafe fn init(memory_map: &'static MemoryMap) -> Self {
        Self {
            memory_map,
            next: 0,
        }
    }
    /// Returns an iterator over usable frames specified in the memory map.
    pub fn usuable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        self.memory_map
            .iter()
            .filter(|r| r.region_type == MemoryRegionType::Usable)
            .map(|r| r.range.start_addr()..r.range.end_addr())
            .flat_map(|r| r.step_by(4096))
            .map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        let frame = self.usuable_frames().nth(self.next);
        self.next += 1;

        frame
    }
}

/// A FrameAllocator that always returns `None`.
pub struct EmptyFrameAllocator;

unsafe impl FrameAllocator<Size4KiB> for EmptyFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        None
    }
}

pub fn create_example_mapping(
    page: Page,
    mapper: &mut OffsetPageTable,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) {
    let frame = PhysFrame::containing_address(PhysAddr::new(0xB8000));
    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

    let result_of_map_to = unsafe {
        // FIXME: this isn't safe and was only for testing
        mapper.map_to(page, frame, flags, frame_allocator)
    };
    result_of_map_to.expect("map_to failed").flush();
}

/// Returns a mutable reference to the active level 4 table.
///
/// # Safety
/// This function is unsafe because the caller must guarantee that the complete physical memory is
/// mapped to the virtual memory at the passed `physical_memory_offset`. Also, this function must
/// be only called once to avoid aliasing with `&mut` references (which is undefined behavior).
///
unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    #[inline]
    fn unsafe_spotting_ver(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
        let (level_4_table_frame, _) = Cr3::read();

        let phys = level_4_table_frame.start_address();
        let virt = physical_memory_offset + phys.as_u64();
        let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

        unsafe { &mut *page_table_ptr }
    }

    unsafe_spotting_ver(physical_memory_offset)
}

/// Initialize a new OffsetPageTable.
///
/// # Safety
/// This function is unsafe because the caller must guarantee that the complete physical memory is
/// mapped to the virtual memory at the passed `physical_memory_offset`. Also, this function must
/// be only called once to avoid aliasing with `&mut` references (which is undefined behavior).
///
pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    let level_4_table = active_level_4_table(physical_memory_offset);
    OffsetPageTable::new(level_4_table, physical_memory_offset)
}

/*
Poor thing's been abandoned...
Big Sadge^TM.

/// Translates the given virtual address to the mapped physical address, or `None` if the address
/// is not mapped.
///
/// # Safety
/// This function is unsafe because the caller must guarantee that the complete physical memory is
/// mapped to the virtual memory at the passed `physical_memory_offset`.
///
pub unsafe fn translate_addr(addr: VirtAddr, physical_memory_offset: VirtAddr) -> Option<PhysAddr> {
    #[inline]
    fn unsafe_spotting_ver(addr: VirtAddr, physical_memory_offset: VirtAddr) -> Option<PhysAddr> {
        let (mut frame, _) = Cr3::read();

        let indices = [
            addr.p4_index(),
            addr.p3_index(),
            addr.p2_index(),
            addr.p1_index(),
        ];

        for index in indices {
            let virt = physical_memory_offset + frame.start_address().as_u64();
            let table_ptr: *const PageTable = virt.as_ptr();

            let table = unsafe { &*table_ptr };

            frame = match table[index].frame() {
                Ok(frame) => frame,
                Err(FrameError::HugeFrame) => panic!("huge frames not supported for now"),
                Err(FrameError::FrameNotPresent) => return None,
            };
        }

        Some(frame.start_address() + u64::from(addr.page_offset()))
    }

    unsafe_spotting_ver(addr, physical_memory_offset)
}
*/
