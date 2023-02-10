fn main() -> Result<(), Box<dyn std::error::Error>> {
    pub enum Type { VideoCapture = 1 }
    pub enum FieldOrder { /*Any,*/ Progressive=1 }
    pub enum TransferFunction { /*Default, Rec709, SRGB, OPRGB, SMPTE240M,*/ None=5 }
    pub enum Memory { Mmap = 1 }
    //if std::env::args().any(|arg| arg.contains("send")) {
    use rustix::fd::{AsFd,AsRawFd};
    let fd = rustix::fs::openat(rustix::fs::cwd(), "/dev/video0", rustix::fs::OFlags::RDWR/*|rustix::fs::OFlags::NONBLOCK*/, rustix::fs::Mode::empty())?;
    use v4l::*;
    unsafe{libc::ioctl(fd.as_fd().as_raw_fd(), linux::ioctl::VIDIOC_S_FMT as u64, &mut v4l2_format{type_: Type::VideoCapture as u32, fmt: v4l2_format__bindgen_ty_1{pix: v4l::v4l2_pix_format{width: 160, height: 120, pixelformat: u32::from_le_bytes(*b"Y16 "),
    field: FieldOrder::Progressive as u32, bytesperline: 160*2, sizeimage: 160*120*2, colorspace: 0, flags: 0, quantization: 0,
    xfer_func: TransferFunction::None as u32, ..std::mem::zeroed()}}} as *mut _ as *mut std::os::raw::c_void)};
    unsafe{libc::ioctl(fd.as_fd().as_raw_fd(), linux::ioctl::VIDIOC_REQBUFS as u64, &mut v4l2_requestbuffers{type_: Type::VideoCapture as u32, memory: Memory::Mmap as u32, count: 1, ..std::mem::zeroed()} as *mut _ as *mut std::os::raw::c_void)};
    let mut buffer = v4l2_buffer{type_: Type::VideoCapture as u32, memory: Memory::Mmap as u32, index: 0, ..unsafe { std::mem::zeroed() }};
    unsafe{libc::ioctl(fd.as_fd().as_raw_fd(), linux::ioctl::VIDIOC_QUERYBUF as u64, &mut buffer as *mut _ as *mut std::os::raw::c_void)};

    pub struct MemoryMap{ ptr: *mut core::ffi::c_void, len: usize }
    impl std::ops::Deref for MemoryMap { type Target = [u8]; fn deref(&self) -> &Self::Target { unsafe { std::slice::from_raw_parts(self.ptr as *const u8, self.len) } } }
    impl Drop for MemoryMap { fn drop(&mut self) { unsafe { rustix::mm::munmap(self.ptr, self.len).unwrap() } } }

    let data = MemoryMap{ptr: unsafe{rustix::mm::mmap(std::ptr::null_mut(), buffer.length as usize, rustix::mm::ProtFlags::READ, rustix::mm::MapFlags::SHARED, fd.as_fd(), buffer.m.offset as u64)?}, len: buffer.length as usize};

    unsafe{libc::ioctl(fd.as_fd().as_raw_fd(), linux::ioctl::VIDIOC_QBUF as u64, &mut buffer as *mut _ as *mut std::os::raw::c_void)};
    unsafe{libc::ioctl(fd.as_fd().as_raw_fd(), linux::ioctl::VIDIOC_STREAMON as u64, &mut (Type::VideoCapture as u32) as *mut _ as *mut std::os::raw::c_void)};
    //let socket = std::net::UdpSocket::bind("10.0.0.4:8888")?;
    loop {
        println!("poll");
        use rustix::{io::{PollFd,PollFlags}};
        let ref mut fds = [PollFd::new(&fd, PollFlags::IN)];
        rustix::io::poll(fds, -1)?;
        //fds.map(|fd| fd.revents().contains(PollFlags::IN)).any()
        let mut buffer = v4l::v4l2_buffer{type_: Type::VideoCapture as u32, memory: Memory::Mmap as u32, index: 0, ..unsafe { std::mem::zeroed() }};
        println!("dequeue");
        unsafe { libc::ioctl(fd.as_raw_fd(), linux::ioctl::VIDIOC_DQBUF as u64, &mut buffer as *mut _ as *mut std::os::raw::c_void); } //flags, field, timestamp, sequence
        println!("send");
        //socket.send_to(&data[..buffer.bytesused as usize], "10.0.0.3:6666")?;
        println!("sent");
        unsafe{libc::ioctl(fd.as_fd().as_raw_fd(), linux::ioctl::VIDIOC_QBUF as u64, &mut buffer as *mut _ as *mut std::os::raw::c_void)};
    }
}
