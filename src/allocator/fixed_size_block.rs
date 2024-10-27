use core::{
    alloc::{GlobalAlloc, Layout},
    mem,
    ptr::{self, NonNull},
};

use super::Locked;

/// The block sizes to use.
///
/// The block sizes must each be a power of 2 because they are also used as the block alignment
/// (alignments must always be powers of 2).
const BLOCK_SIZES: &[usize] = &[8, 16, 32, 64, 128, 256, 512, 1024, 2048];

struct ListNode {
    next: Option<&'static mut ListNode>,
}

pub struct FixedSizedBlockAllocator {
    list_heads: [Option<&'static mut ListNode>; BLOCK_SIZES.len()],
    fallback_allocator: linked_list_allocator::Heap,
}

impl FixedSizedBlockAllocator {
    /// Creates an empty FixedBlockSizeAllocator.
    pub const fn new() -> Self {
        Self {
            list_heads: [const { None }; BLOCK_SIZES.len()],
            fallback_allocator: linked_list_allocator::Heap::empty(),
        }
    }

    /// Initialize the heading with the given heap bounds.
    ///
    /// # Safety
    /// This function is unsafe because the caller must guarantee that the given heap bounds are
    /// valid and that the heap is unused. This method must be called only once.
    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.fallback_allocator.init(heap_start, heap_size);
    }

    /// Allocates using the fallback allocator.
    fn fallback_alloc(&mut self, layout: Layout) -> *mut u8 {
        // I don't fully get giving null pointers.
        match self.fallback_allocator.allocate_first_fit(layout) {
            Ok(ptr) => ptr.as_ptr(),
            Err(_) => ptr::null_mut(),
        }
    }
}

unsafe impl GlobalAlloc for Locked<FixedSizedBlockAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut allocator = self.lock();

        let Some(index) = list_index(&layout) else {
            return allocator.fallback_alloc(layout);
        };

        match allocator.list_heads[index].take() {
            Some(node) => {
                allocator.list_heads[index] = node.next.take();
                node as *mut ListNode as *mut u8
            }
            None => {
                let block_size = BLOCK_SIZES[index];
                let block_align = block_size;

                let layout = Layout::from_size_align(block_size, block_align)
                    .expect("block sizes weren't a power of 2?");

                allocator.fallback_alloc(layout)
            }
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let mut allocator = self.lock();

        match list_index(&layout) {
            Some(index) => {
                let new_node = ListNode {
                    next: allocator.list_heads[index].take(),
                };
                assert!(mem::size_of::<ListNode>() <= BLOCK_SIZES[index]);
                assert!(mem::align_of::<ListNode>() <= BLOCK_SIZES[index]);

                let new_node_ptr = ptr as *mut ListNode;
                new_node_ptr.write(new_node);
                allocator.list_heads[index] = Some(&mut *new_node_ptr);
            }
            None => {
                let ptr = NonNull::new(ptr).unwrap();
                allocator.fallback_allocator.deallocate(ptr, layout);
            }
        }
    }
}

/// Chooses an appropriate block size for the given layout.
///
/// Returns an index of the `BLOCK_SIZES` array.
fn list_index(layout: &Layout) -> Option<usize> {
    let required_block_size = layout.size().max(layout.align());
    BLOCK_SIZES.iter().position(|&s| required_block_size <= s)
}