use std::{ops::Deref, sync::{Arc, Mutex}};

use log::debug;
use xalloc::{BitmapAlloc,BitmapAllocRegion};
use pci_driver::regions::{PciMemoryRegion, PciRegion, Permissions};

pub struct AllocatorInner {
    allocator: BitmapAlloc,
    granularity: usize,
    memory: PciMemoryRegion<'static>,
}

#[derive(Clone)]
pub struct Allocator(Arc<Mutex<AllocatorInner>>);

pub struct AllocationGuard {
    allocator: Allocator,
    region: Option<BitmapAllocRegion>,
    memory: PciMemoryRegion<'static>,
}

impl Allocator {
    pub fn new(memory: PciMemoryRegion<'static>, granularity: usize) -> Self {
        let size = (memory.len() as usize) / granularity;
        Self(Arc::new(Mutex::new(AllocatorInner {
            allocator: BitmapAlloc::new(size),
            granularity,
            memory,
        })))
    }

    pub fn alloc(&self, size: usize) -> Option<AllocationGuard> {
        let mut inner = self.0.lock().unwrap();
        if let Some((region, start)) = inner.allocator.alloc(size) {
            let memory = unsafe { PciMemoryRegion::new_raw(
                inner.memory.as_mut_ptr().unwrap().add(inner.granularity*start),
                inner.granularity*size,
                Permissions::ReadWrite
            ) };

            debug!("Allocated {} pages at {:?}", size, memory.as_ptr().unwrap());

            Some(AllocationGuard {
                allocator: self.clone(),
                region: Some(region),
                memory: memory
            })
        } else {
            None
        }
    }
}

impl Drop for AllocationGuard {
    fn drop(&mut self) {
        let mut allocator = self.allocator.0.lock().unwrap();
        debug!("Deallocating {:?}", self.memory.as_ptr().unwrap());
        if let Some(region) = self.region.take(){
            allocator.allocator.dealloc_relaxed(region);
        }
    }
}

impl Deref for AllocationGuard {
    type Target = PciMemoryRegion<'static>;

    fn deref(&self) -> &Self::Target {
        &self.memory
    }
}