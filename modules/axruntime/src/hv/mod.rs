use axalloc::global_allocator;
use axhal::mem::{phys_to_virt, virt_to_phys, PAGE_SIZE_4K};
use hypercraft::{GuestPhysAddr, HostPhysAddr, HostVirtAddr, HyperCraftHal, VM};

use crate::GuestPageTable;

#[cfg(target_arch = "x86_64")]
mod vmx;

static mut CURRENT_VM: Option<usize> = None;

pub fn set_current_vm(vm: &VM<HyperCraftHalImpl, GuestPageTable>) {
    unsafe {
        CURRENT_VM = Some(vm as *const _ as usize);
    }
}

pub fn vm_ipa2pa(gpa: GuestPhysAddr) -> HostPhysAddr {
    unsafe {
        match CURRENT_VM {
            Some(vm_addr) => {
                let vm: *const VM<HyperCraftHalImpl, GuestPageTable> = vm_addr as *const _;
                (*vm).ipa2pa(gpa).map_or(0, |v| v)
            }
            None => {
                panic!("vm_ipa2pa shouldn't fail");
            }
        }
    }
}

/// An empty struct to implementate of `HyperCraftHal`
pub struct HyperCraftHalImpl;

impl HyperCraftHal for HyperCraftHalImpl {
    fn alloc_pages(num_pages: usize) -> Option<hypercraft::HostVirtAddr> {
        global_allocator()
            .alloc_pages(num_pages, PAGE_SIZE_4K)
            .map(|pa| pa as HostVirtAddr)
            .ok()
    }

    fn dealloc_pages(pa: HostVirtAddr, num_pages: usize) {
        global_allocator().dealloc_pages(pa as usize, num_pages);
    }

    #[cfg(target_arch = "x86_64")]
    fn phys_to_virt(pa: HostPhysAddr) -> HostVirtAddr {
        phys_to_virt(pa.into()).into()
    }

    #[cfg(target_arch = "x86_64")]
    fn virt_to_phys(va: HostVirtAddr) -> HostPhysAddr {
        virt_to_phys(va.into()).into()
    }

    #[cfg(target_arch = "x86_64")]
    fn vmexit_handler(vcpu: &mut VCpu<Self>) -> HyperResult {
        vmx::vmexit_handler(vcpu)
    }

    #[cfg(target_arch = "x86_64")]
    fn current_time_nanos() -> u64 {
        axhal::time::current_time_nanos()
    }
}
