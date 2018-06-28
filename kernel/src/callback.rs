//! Data structure for storing a callback to userspace or kernelspace.

use core::ptr::NonNull;
use process;
use sched::Kernel;

/// Userspace app identifier.
#[derive(Clone, Copy)]
pub struct AppId {
    idx: usize,
    kernel: &'static Kernel,
}

/// The kernel can masquerade as an app. IDs >= this value are the kernel.
/// These IDs are used to identify which kernel container is being accessed.
const KERNEL_APPID_BOUNDARY: usize = 100;

impl AppId {
    pub(crate) fn new(kernel: &'static Kernel, idx: usize) -> AppId {
        AppId { idx: idx, kernel: kernel }
    }

    pub(crate) const fn kernel_new(kernel: &'static Kernel, idx: usize) -> AppId {
        AppId { idx: idx, kernel: kernel }
    }

    pub const fn is_kernel(self) -> bool {
        self.idx >= KERNEL_APPID_BOUNDARY
    }

    pub const fn is_kernel_idx(idx: usize) -> bool {
        idx >= KERNEL_APPID_BOUNDARY
    }

    pub fn idx(&self) -> usize {
        self.idx
    }

    /// Returns the full address of the start and end of the flash region that
    /// the app owns and can write to. This includes the app's code and data and
    /// any padding at the end of the app. It does not include the TBF header,
    /// or any space that the kernel is using for any potential bookkeeping.
    pub fn get_editable_flash_range(&self) -> (usize, usize) {

        // pub fn get_editable_flash_range(app_idx: usize) -> (usize, usize) {
        //     let procs = unsafe { &mut PROCS };
            if self.idx >= self.kernel.processes.len() {
                return (0, 0);
            }

            match self.kernel.processes[self.idx] {
                None => (0, 0),
                Some(ref mut p) => {
                    let start = p.flash_non_protected_start() as usize;
                    let end = p.flash_end() as usize;
                    (start, end)
                }
            }
        // }
        // process::get_editable_flash_range(self.idx)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum RustOrRawFnPtr {
    Raw {
        ptr: NonNull<*mut ()>,
    },
    Rust {
        func: fn(usize, usize, usize, usize),
    },
}

/// Wrapper around a function pointer.
#[derive(Clone, Copy)]
pub struct Callback {
    kernel: &'static Kernel,
    app_id: AppId,
    appdata: usize,
    fn_ptr: RustOrRawFnPtr,
}

impl Callback {
    pub(crate) fn new(kernel: &'static Kernel, appid: AppId, appdata: usize, fn_ptr: NonNull<*mut ()>) -> Callback {
        Callback {
            kernel: kernel,
            app_id: appid,
            appdata: appdata,
            fn_ptr: RustOrRawFnPtr::Raw { ptr: fn_ptr },
        }
    }

    pub(crate) const fn kernel_new(
        kernel: &'static Kernel,
        appid: AppId,
        fn_ptr: fn(usize, usize, usize, usize),
    ) -> Callback {
        Callback {
            kernel: kernel,
            app_id: appid,
            appdata: 0,
            fn_ptr: RustOrRawFnPtr::Rust { func: fn_ptr },
        }
    }

    pub fn schedule(&mut self, r0: usize, r1: usize, r2: usize) -> bool {
        if self.app_id.is_kernel() {
            let fn_ptr = match self.fn_ptr {
                RustOrRawFnPtr::Raw { ptr } => {
                    panic!("Attempt to rust_call a raw function pointer: ptr {:?}", ptr)
                }
                RustOrRawFnPtr::Rust { func } => func,
            };
            fn_ptr(r0, r1, r2, self.appdata);
            true
        } else {
            let fn_ptr = match self.fn_ptr {
                RustOrRawFnPtr::Raw { ptr } => ptr,
                RustOrRawFnPtr::Rust { func } => {
                    panic!("Attempt to schedule rust function: func {:?}", func)
                }
            };
            // self.kernel.schedule(
            //     process::FunctionCall {
            //         r0: r0,
            //         r1: r1,
            //         r2: r2,
            //         r3: self.appdata,
            //         pc: fn_ptr.as_ptr() as usize,
            //     },
            //     self.app_id,
            // )

            // pub fn schedule_callback(&self, callback: FunctionCall, appid: AppId) -> bool {
            // let procs = unsafe { &mut PROCS };
            let idx = self.app_id.idx();
            if idx >= self.kernel.processes.len() {
                return false;
            }

            match self.kernel.processes[idx] {
                None => false,
                Some(ref mut p) => {
                    p.schedule(process::FunctionCall {
                        r0: r0,
                        r1: r1,
                        r2: r2,
                        r3: self.appdata,
                        pc: fn_ptr.as_ptr() as usize,
                    })




                    // // If this app is in the `Fault` state then we shouldn't schedule
                    // // any work for it.
                    // if p.current_state() == process::State::Fault {
                    //     return false;
                    // }

                    // self.kernel.increment_work();

                    // let ret = p.tasks.enqueue(process::Task::FunctionCall(process::FunctionCall {
                    //     r0: r0,
                    //     r1: r1,
                    //     r2: r2,
                    //     r3: self.appdata,
                    //     pc: fn_ptr.as_ptr() as usize,
                    // }));

                    // // Make a note that we lost this callback if the enqueue function
                    // // fails.
                    // if ret == false {
                    //     p.debug
                    //         .dropped_callback_count
                    //         .set(p.debug.dropped_callback_count.get() + 1);
                    // }

                    // ret
                }
                // }
            }
        }
    }
}
