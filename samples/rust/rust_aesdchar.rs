//! Virtual Device Module
use kernel::prelude::*;

use kernel::file::{File, Operations};
use kernel::io_buffer::{IoBufferReader, IoBufferWriter};
use kernel::sync::smutex::Mutex;
use kernel::sync::{Ref, RefBorrow};
use kernel::{miscdev, Module};

module! {
    type: RustAesdChar,
    name: "rust_aesdchar",
    license: "GPL",
}

const AESDCHAR_MAX_WRITE_SUPPORTED: usize = 10;

struct Device {
    number: usize,
    contents: Mutex<Vec<Vec<u8>>>,
    working_entry: Mutex<usize>,
    reading_entry: Mutex<usize>,
}

struct RustAesdChar {
    _devs: Pin<Box<miscdev::Registration<RustAesdChar>>>,
}

#[vtable]
impl Operations for RustAesdChar {
    type OpenData = Ref<Device>;
    type Data = Ref<Device>;

    fn open(context: &Ref<Device>, _file: &File) -> Result<Ref<Device>> {
        pr_info!("File for device {} was opened\n", context.number);
        Ok(context.clone())
    }

    fn read(
        data: RefBorrow<'_, Device>,
        _file: &File,
        writer: &mut impl IoBufferWriter,
        offset: u64,
    ) -> Result<usize> {
        pr_info!("File for device {} was read\n", data.number);
        let _offset: usize = offset.try_into()?;
        let mut reading_entry = data.reading_entry.lock();
        let buf: &Vec<Vec<u8>> = &*data.contents.lock();
        let vec: &Vec<u8> = &buf[*reading_entry];
        let len = core::cmp::min(writer.len(), vec.len());
        if len > 0 {
            // if all bytes are consumed
            if len == vec.len() {
                *reading_entry = (*reading_entry + 1) % AESDCHAR_MAX_WRITE_SUPPORTED;
            }
            writer.write_slice(&vec[..len])?;
        }
        Ok(len)
    }

    fn write(
        data: RefBorrow<'_, Device>,
        _file: &File,
        reader: &mut impl IoBufferReader,
        _offset: u64,
    ) -> Result<usize> {
        pr_info!("File for device {} was written\n", data.number);
        let _offset: usize = _offset.try_into()?;
        let copy = reader.read_all()?;
        let len = copy.len();
        let mut working_entry = data.working_entry.lock();
        let mut reading_entry = data.reading_entry.lock();
        let buf: &mut Vec<Vec<u8>> = &mut *data.contents.lock();
        let vec: &mut Vec<u8> = &mut buf[*working_entry];
        let mut terminated = false;

        // Check if we have '/n'
        for elem in copy.iter() {
            if *elem == 10 {
                terminated = true;
            }
        }

        // Copy into the selected vector
        for elem in copy.iter() {
            vec.try_push(*elem)?;
        }
        // If we terminated, then we change the working_entry
        if terminated {
            // Push the terminatation for string
            vec.try_push(0)?;
            *working_entry = (*working_entry + 1) % AESDCHAR_MAX_WRITE_SUPPORTED;
            if *working_entry == *reading_entry {
                *reading_entry = (*reading_entry + 1) % AESDCHAR_MAX_WRITE_SUPPORTED;
            }
            let vec_new: &mut Vec<u8> = &mut buf[*working_entry];
            vec_new.clear();
        }
        pr_info!("{:?}", buf);
        pr_info!("Written {} bytes", len);
        Ok(len)
    }
}

impl Module for RustAesdChar {
    fn init(_name: &'static CStr, _module: &'static ThisModule) -> Result<Self> {
        pr_info!("-----------------------\n");
        pr_info!("starting aesdchar dev..\n");
        pr_info!("-----------------------\n");
        let mut vec_of_vec: Vec<Vec<u8>> = Vec::new();
        for _ in 0..AESDCHAR_MAX_WRITE_SUPPORTED {
            let vec: Vec<u8> = Vec::new();
            vec_of_vec.try_push(vec)?;
        }
        pr_info!("{:?}", vec_of_vec);
        let dev = Ref::try_new(Device {
            number: 0,
            contents: Mutex::new(vec_of_vec),
            working_entry: Mutex::new(0),
            reading_entry: Mutex::new(0),
        })?;
        let reg = miscdev::Registration::new_pinned(fmt!("rust_aesdchar"), dev)?;
        Ok(Self { _devs: reg })
      }
}

