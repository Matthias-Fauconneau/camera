fn main() -> ui::Result<()> {
    let path = "/dev/video4";
    let device = v4l::Device::with_path(path)?;
    use v4l::video::Capture;
    let format = device.set_format(&v4l::format::Format{
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
    })?;
    //let params = device.params()?;
    struct View<'t> {
        format: v4l::format::Format,
        stream: v4l::io::mmap::Stream<'t>
    }
    impl ui::Widget for View<'_> { #[fehler::throws(ui::Error)] fn paint(&mut self, target: &mut ui::Target, _: ui::size, _: ui::int2) {
        use v4l::io::traits::CaptureStream;
        let (source, _) = self.stream.next().unwrap();
        use vector::xy;
        let source = image::Image::<&[u16]>::cast_slice(source, xy{x: self.format.width, y: self.format.height});
        let min = *source.iter().min().unwrap();
        let max = *source.iter().max().unwrap();
        for y in 0..target.size.y {
            for x in 0..target.size.x {
                let w = (source[xy{x: x*source.size.x/target.size.x, y: y*source.size.y/target.size.y}] - min) as u32 * ((1<<10)-1) / (max - min) as u32;
                target[xy{x,y}] = w | w<<10 | w<<20;
            }
        }
    } }

    ui::run(&mut View{format, stream: v4l::io::mmap::Stream::with_buffers(&device, v4l::buffer::Type::VideoCapture, 1)?}, &mut |_| Ok(true))
}
