//! Contains functions and macros for providing a runtime environment to the bechmarking suite
use core::{alloc::GlobalAlloc, cell::RefCell, panic::PanicInfo, ptr::NonNull};

use linked_list_allocator::Heap;

use crate::platform::{self, Platform};

#[global_allocator]
static ALLOCATOR: CustomHeap = CustomHeap::empty();

extern "C" {
    static _sheap: u8;
    static _heap_size: u8;
}

/// Initializes the heap and enables use of the alloc crate,
/// also initializes communication module and enables the use of the print & println macro
///
/// # Safety
///  - only call once
pub unsafe fn init() -> Result<(), &'static str> {
    let heap_bottom = &_sheap as *const u8 as usize;
    let heap_size = &_heap_size as *const u8 as usize;
    ALLOCATOR.init(heap_bottom, heap_size);

    // Safety:
    // This should be the first time the communication module is accessed,
    // invalidating previous references is ok
    let comm = platform::current().get_communication_module();
    comm.init()
}

/// Since the architecture is assumed to be on a single core and without atomic instructions
/// the GlobalAlloc Trait has to be manually implemented for Heap, therefore we define this
/// Wrapper type
struct CustomHeap(RefCell<Heap>);

impl CustomHeap {
    const fn empty() -> CustomHeap {
        CustomHeap(RefCell::new(Heap::empty()))
    }

    unsafe fn init(&self, heap_bottom: usize, heap_size: usize) {
        self.0.borrow_mut().init(heap_bottom, heap_size)
    }
}

unsafe impl Sync for CustomHeap {}

unsafe impl GlobalAlloc for CustomHeap {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        self.0
            .borrow_mut()
            .allocate_first_fit(layout)
            .ok()
            .map_or(0 as *mut u8, |addr| addr.as_ptr())
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        self.0
            .borrow_mut()
            .deallocate(NonNull::new_unchecked(ptr), layout)
    }
}

// Safety of calling get_communication_module() inside the macros:
// invalidating previous references is ok,
// because all macros reference the module only in a closed scope
// and the architecture is assumed to be on a single core
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => (unsafe {
        use crate::platform::Platform;
        write!($crate::platform::current().get_communication_module(), $($arg)*).unwrap();
    });
}

#[macro_export]
macro_rules! println {
    () => (unsafe {
        use crate::platform::Platform;
        writeln!($crate::platform::current().get_communication_module()).unwrap();
    });
    ($($arg:tt)*) => (unsafe {
        use crate::platform::Platform;
        writeln!($crate::platform::current().get_communication_module(), $($arg)*).unwrap();
    });
}

#[macro_export]
macro_rules! readln {
    () => {
        unsafe {
            use crate::platform::Platform;
            $crate::platform::current()
                .get_communication_module()
                .read_line()
        }
    };
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    unsafe {
        // Safety: invalidating previous references is ok, since we are in a unrecoverable state
        let comm = platform::current().get_communication_module();
        if comm.init().is_ok() {
            let _ = writeln!(comm, "! {}", info);
        }
    }

    platform::current().suspend(101)
}

#[cfg(test)]
pub trait TestFunction {
    fn test_run(&self) -> ();
}

#[cfg(test)]
impl<T> TestFunction for T
where
    T: Fn(),
{
    fn test_run(&self) {
        print!("{}... ", core::any::type_name::<T>());
        self();
        println!("[ok]");
    }
}

#[cfg(test)]
pub fn test_runner(tests: &[&dyn TestFunction]) {
    println!("Running {} tests", tests.len());
    for test in tests {
        test.test_run();
    }
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    unsafe {
        let comm = platform::current().get_communication_module();
        if comm.init().is_ok() {
            let _ = writeln!(comm, "[failed]");
            let _ = writeln!(comm, "Error: {}", info);
        }
    }

    platform::current().suspend(101)
}

#[cfg(test)]
mod tests {
    #[test_case]
    fn tests_are_working() {
        assert_eq!(1, 1);
    }

    #[test_case]
    fn basic_allocations_are_working() {
        use alloc::collections::vec_deque::VecDeque;

        let mut vector = VecDeque::new();
        for i in 0..10 {
            vector.push_back(i);
        }

        assert_eq!(vector.len(), 10)
    }
}
