fn main() -> std::io::Result<()> {
    use v4l::{video::Capture, io::traits::CaptureStream};
    let format = v4l::format::Format{
        width: 160,
        height: 120,
        fourcc: v4l::FourCC::new(b"Y16 "),
        field_order:  v4l::format::field::FieldOrder::Progressive,
        stride: 160*2,
        size: 160*120*2,
        flags: v4l::format::Flags::empty(),
        colorspace: v4l::format::colorspace::Colorspace::Default,
        quantization: v4l::format::quantization::Quantization::FullRange,
        transfer: v4l::format::transfer::TransferFunction::None
    };
    //if std::env::args().any(|arg| arg.contains("send")) {
        let socket = std::net::UdpSocket::bind("10.0.0.4:8888")?;
        let device = v4l::Device::with_path("/dev/video0")?;
        device.set_format(&format)?;
        let mut stream = v4l::io::mmap::Stream::with_buffers(&device, v4l::buffer::Type::VideoCapture, 1)?;
        loop {
            println!("read");
            let (image, _) = stream.next().unwrap();
            println!("send");
            socket.send_to(image, "10.0.0.3:6666")?;
            println!("sent");
        }
    /*} else {
        struct View {
            format: v4l::format::Format,
            stream: std::net::UdpSocket,
        }
        impl ui::Widget for View { #[fehler::throws(ui::Error)] fn paint(&mut self, target: &mut ui::Target, _: ui::size, _: ui::int2) {
            use vector::xy;
            let mut image = image::Image::<Box<[u16]>>::zero(xy{x: self.format.width, y: self.format.height});
            println!("receive");
            let (len, _sender) = self.stream.recv_from(bytemuck::cast_slice_mut(&mut image))?;
            println!("received");
            assert_eq!(len, image.len()*2);
            let min = *image.iter().min().unwrap();
            let max = *image.iter().max().unwrap();
            if min <= 0 { return; }
            for y in 0..target.size.y {
                for x in 0..target.size.x {
                    let w = (image[xy{x: x*image.size.x/target.size.x, y: y*image.size.y/target.size.y}] - min) as u32 * ((1<<10)-1) / (max - min) as u32;
                    target[xy{x,y}] = w | w<<10 | w<<20;
                }
            }
        } }
        ui::run(&mut View{format, stream: std::net::UdpSocket::bind("10.0.0.1:6666")?}, &mut |_| Ok(true))
    }*/
}
