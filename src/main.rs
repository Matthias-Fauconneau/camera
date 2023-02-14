fn main() -> Result<(), Box<dyn std::error::Error>> {
    const SIZE : (u32, u32) = (160, 120);
    if !std::env::args().any(|arg| arg.contains("receive")) {
        pub enum Type { VideoCapture = 1 }
        pub enum FieldOrder { /*Any,*/ Progressive=1 }
        pub enum TransferFunction { /*Default, Rec709, SRGB, OPRGB, SMPTE240M,*/ None=5 }
        pub enum Memory { Mmap = 1 }
        use rustix::fd::{AsFd,AsRawFd};
        let fd = rustix::fs::openat(rustix::fs::cwd(), "/dev/video0", rustix::fs::OFlags::RDWR/*|rustix::fs::OFlags::NONBLOCK*/, rustix::fs::Mode::empty())?;
        use v4l::*;
        unsafe{libc::ioctl(fd.as_fd().as_raw_fd(), linux::ioctl::VIDIOC_S_FMT as _, &mut v4l2_format{type_: Type::VideoCapture as u32, fmt: v4l2_format__bindgen_ty_1{pix: v4l::v4l2_pix_format{width: 160, height: 120, pixelformat: u32::from_le_bytes(*b"Y16 "),
        field: FieldOrder::Progressive as u32, bytesperline: 160*2, sizeimage: 160*120*2, colorspace: 0, flags: 0, quantization: 0,
        xfer_func: TransferFunction::None as u32, ..std::mem::zeroed()}}} as *mut _ as *mut std::os::raw::c_void)};
        unsafe{libc::ioctl(fd.as_fd().as_raw_fd(), linux::ioctl::VIDIOC_REQBUFS as _, &mut v4l2_requestbuffers{type_: Type::VideoCapture as u32, memory: Memory::Mmap as u32, count: 1, ..std::mem::zeroed()} as *mut _ as *mut std::os::raw::c_void)};
        let mut buffer = v4l2_buffer{type_: Type::VideoCapture as u32, memory: Memory::Mmap as u32, index: 0, ..unsafe { std::mem::zeroed() }};
        unsafe{libc::ioctl(fd.as_fd().as_raw_fd(), linux::ioctl::VIDIOC_QUERYBUF as _, &mut buffer as *mut _ as *mut std::os::raw::c_void)};

        pub struct MemoryMap{ ptr: *mut core::ffi::c_void, len: usize }
        impl std::ops::Deref for MemoryMap { type Target = [u8]; fn deref(&self) -> &Self::Target { unsafe { std::slice::from_raw_parts(self.ptr as *const u8, self.len) } } }
        impl Drop for MemoryMap { fn drop(&mut self) { unsafe { rustix::mm::munmap(self.ptr, self.len).unwrap() } } }

        let data = MemoryMap{ptr: unsafe{rustix::mm::mmap(std::ptr::null_mut(), buffer.length as usize, rustix::mm::ProtFlags::READ, rustix::mm::MapFlags::SHARED, fd.as_fd(), buffer.m.offset as u64)?}, len: buffer.length as usize};

        unsafe{libc::ioctl(fd.as_fd().as_raw_fd(), linux::ioctl::VIDIOC_QBUF as _, &mut buffer as *mut _ as *mut std::os::raw::c_void)};
        unsafe{libc::ioctl(fd.as_fd().as_raw_fd(), linux::ioctl::VIDIOC_STREAMON as _, &mut (Type::VideoCapture as u32) as *mut _ as *mut std::os::raw::c_void)};
        let socket = std::env::args().skip(1).next().map(|address| std::net::UdpSocket::bind(address).unwrap()); // 192.168.0.106:8888
        loop {
            println!("poll");
            use rustix::{io::{PollFd,PollFlags}};
            let ref mut fds = [PollFd::new(&fd, PollFlags::IN)];
            rustix::io::poll(fds, -1)?;
            //fds.map(|fd| fd.revents().contains(PollFlags::IN)).any()
            let mut buffer = v4l::v4l2_buffer{type_: Type::VideoCapture as u32, memory: Memory::Mmap as u32, index: 0, ..unsafe { std::mem::zeroed() }};
            println!("dequeue");
            unsafe { libc::ioctl(fd.as_raw_fd(), linux::ioctl::VIDIOC_DQBUF as _, &mut buffer as *mut _ as *mut std::os::raw::c_void); } //flags, field, timestamp, sequence
            let linux::general::__kernel_timespec{tv_sec,tv_nsec} = rustix::time::clock_gettime(rustix::time::ClockId::Monotonic);
            println!("{}", (tv_sec*1_000_000+tv_nsec/1000) as i64-(buffer.timestamp.tv_sec*1_000_000+buffer.timestamp.tv_usec) as i64);
            if let Some(socket) = socket.as_ref() { socket.send_to(&data[..buffer.bytesused as usize], std::env::args().skip(2).next().unwrap())?; } // 192.168.0.104:6666
            println!("sent");
            unsafe{libc::ioctl(fd.as_fd().as_raw_fd(), linux::ioctl::VIDIOC_QBUF as _, &mut buffer as *mut _ as *mut std::os::raw::c_void)};
        }
    } else {
        println!("{}", local_ip_address::local_ip()?);
        struct View(std::net::UdpSocket);
        use ui::*;
        impl Widget for View {
            #[throws] fn paint(&mut self, target: &mut Target, _: size, _: int2) {
                let mut image = image::Image::<Box<[u16]>>::zero(SIZE.into());
                println!("receive");
                let (len, _sender) = self.0.recv_from(bytemuck::cast_slice_mut(&mut image))?;
                println!("received");
                assert_eq!(len, image.len()*2);
                let min = *image.iter().min().unwrap();
                let max = *image.iter().max().unwrap();
                for value in image.iter_mut() { *value = (((*value - min) as u32 * ((1<<10)-1)) / (max - min) as u32) as u16; }
                for y in 0..target.size.y {
                    for x in 0..target.size.x {
                        let w = image[xy{x: image.size.x-1-y*image.size.x/target.size.y, y: x*image.size.y/target.size.x}] as u32;
                        target[xy{x,y}] = w | w<<10 | w<<20;
                    }
                }
            }
            fn event(&mut self, _: size, _: &mut Option<EventContext>, _: &ui::Event) -> Result<bool> { Ok(true) }
        }
        let ref address = std::env::args().skip(2).next().unwrap();
        ui::run(address, &mut View(std::net::UdpSocket::bind(address)?))
    }
}
