//! Virtual Device Module
use kernel::prelude::*;
use kernel::file::{File, Operations};
use kernel::{miscdev, Module};

module! {
    type: VDev,
    name: "vdev",
    license: "GPL",
}

struct VDev {
    _dev: Pin<Box<miscdev::Registration<VDev>>>,
}

#[vtable]
impl Operations for VDev {
    fn open(_context: &(), _file: &File) -> Result {
        pr_info!("File was opened\n");
        Ok(())
    }
}

impl kernel::Module for VDev {
    fn init(_name: &'static CStr, _module: &'static ThisModule) -> Result<Self> {
        // Print a banner to make sure our module is working
        pr_info!("------------------------\n");
        pr_info!("starting virtual device!\n");
        pr_info!("------------------------\n");
        let reg = miscdev::Registration::new_pinned(fmt!("vdev"), ())?;
        Ok(Self { _dev:reg })
    }
}
