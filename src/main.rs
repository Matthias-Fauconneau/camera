#![cfg_attr(feature="portable_simd", feature(portable_simd))]
//use {vector::xy, image::Image};
fn main() {//-> Result<(), Box<dyn std::error::Error>> {
	std::env::set_var("GENICAM_GENTL32_PATH","/usr/lib/ids/cti");
	// max UDP:65,527
  //const SIZE : xy<u32> = xy{x: 320, y: 200}; //1920x1200/6=320x200=64000 @6M/s (next 384x240=92K. @164fps=10M/s)
	//const SIZE : xy<u32> = xy{x: 288, y: 216}; //2592x1944/9=288x216=62208 (next 324x243=79K. @48fps=4M/s)
	//const SIZE : xy<u32> = xy{x: 160, y: 100}; //1920x1200/12 @2M/s
	/*if !std::env::args().any(|arg| arg.contains("ui")) {
		let socket = std::env::args().skip(2).next().map(|address| std::net::UdpSocket::bind(address).unwrap()); // 192.168.0.105:8888
		let to = std::env::args().skip(3).next().unwrap();*/
		/*let mut camera = cameleon::u3v::enumerate_cameras().unwrap().pop().unwrap();
		camera.open().unwrap();
		camera.load_context().unwrap();
		let payload_rx = camera.start_streaming(3).unwrap();
		loop {
			let Ok(payload) = payload_rx.recv_blocking() else { continue; };
			let &cameleon::payload::ImageInfo{width, height, ..} = payload.image_info().unwrap();
			let source = Image::new(xy{x: width as u32, y: height as u32}, payload.image().unwrap());
			#[cfg(feature="new_uninit")] let mut target = Image::uninitialized(SIZE); 
			#[cfg(not(feature="new_uninit"))] let mut target = Image::zero(SIZE);
			//for y in 0..target.size.y { for x in 0..target.size.x { target[xy{x,y}] = source[xy{x: x*6, y: y*6}]; /*FIXME: box*/ } }
			for y in 0..target.size.y { for x in 0..target.size.x { target[xy{x,y}] = source[xy{x: x*9, y: y*9}]; /*FIXME: box*/ } }
			//for y in 0..target.size.y { for x in 0..target.size.x { target[xy{x,y}] = source[xy{x: x*12, y: y*12}]; /*FIXME: box*/ } }
			payload_rx.send_back(payload);
			if let Some(socket) = socket.as_ref() { socket.send_to(&target.data, &to).unwrap(); /*println!("Sent");*/ }
		}*/
		/*let ctx = uvc::Context::new().unwrap();
		let dev = ctx.find_device(None, None, None).unwrap();
		//println!(dev.bus_number(), dev.device_address(), dev.description().vendor_id, description.product_id, description.product, description.manufacturer);
		let dev = dev.open().unwrap();
		let format = dev.get_preferred_format(|x, y| if x.fps >= y.fps && x.width * x.height >= y.width * y.height { x } else { y }}).unwrap();
		println!("{format:?}");
		let mut stream = dev.get_stream_handle_with_format(format).unwrap();
		//println!(dev.scanning_mode(), dev.ae_mode(), dev.ae_priority(), dev.exposure_abs(), dev.exposure_rel(), dev.focus_abs(), dev.focus_rel());
		let frame = Arc::new(Mutex::new(None));
		let _stream = stream.start_stream(|frame,_| {
			let frame = frame.to_rgb()?;
			frame.to_bytes();
			frame.width(), frame.height()
		}, frame.clone()).unwrap();*/
		use std::ptr::null_mut;
		use uvc_sys::*;
		let mut uvc = null_mut();
		assert!(unsafe{uvc_init(&mut uvc as *mut _, null_mut())} >= 0);
		let mut device = null_mut();
		assert!(unsafe{uvc_find_device(uvc, &mut device as *mut _, 0, 0, std::ptr::null())} >= 0);
		let mut device_descriptor : *mut uvc_device_descriptor_t = null_mut();
		unsafe{uvc_get_device_descriptor(device, &mut device_descriptor as &mut _)};
		println!("{:?}", unsafe{std::ffi::CStr::from_ptr(unsafe{*device_descriptor}.product)});
		let mut device_handle = null_mut();
		assert!(unsafe{uvc_open(device, &mut device_handle as *mut _)} >= 0);
		let format_desc = unsafe{uvc_get_format_descs(device_handle)};
		//println!("{}", unsafe{*format_desc}.bDescriptorSubtype); UVC_VS_FORMAT_UNCOMPRESSED
		println!("{}x{} {}", unsafe{*unsafe{*format_desc}.frame_descs}.wWidth, unsafe{*unsafe{*format_desc}.frame_descs}.wHeight, unsafe{*unsafe{*format_desc}.frame_descs}.dwDefaultFrameInterval);
		//unsafe{uvc_print_diag(device_handle, stderr)};
		let mut control = unsafe{std::mem::zeroed()};
		assert!(unsafe{uvc_get_stream_ctrl_format_size(device_handle, &mut control as *mut _, uvc_frame_format_UVC_FRAME_FORMAT_ANY, 256, 192, 25)} >= 0);
		println!("control");
		let mut stream = null_mut();
		assert!(unsafe{uvc_stream_open_ctrl(device_handle, &mut stream as *mut _, &mut control as *mut _)} >= 0);
		println!("stream");
		assert!(unsafe{uvc_stream_start(stream, None, null_mut(), 0)} >= 0);
		println!("start");
		let mut frame : *mut uvc_frame_t = null_mut();
		assert!(unsafe{uvc_stream_get_frame(stream, &mut frame as *mut _, 1000000)} >= 0);
		assert!(!frame.is_null());
		let frame = unsafe{*frame};
		println!("{}", frame.data_bytes);
	/*} else { #[cfg(feature="ui")] {
		struct View(std::net::UdpSocket);
		use ui::*;
		impl Widget for View {
			#[throws] fn paint(&mut self, target: &mut Target, _: size, _: int2) {
				#[cfg(feature="new_uninit")] let mut source = image::Image::<Box<[u8]>>::uninitialized(SIZE);
				#[cfg(not(feature="new_uninit"))] let mut source = image::Image::<Box<[u8]>>::zero(SIZE);
				let source_size = source.size;//.yx(); // Rotated
				let [num, den] = if source_size.x*target.size.y > source_size.y*target.size.x { [target.size.x, source_size.x] } else { [target.size.y, source_size.y] };
				let target_size = source_size*(num/den); // largest integer fit
				let mut target = target.slice_mut((target.size-target_size)/2, target_size);
				if true {
					let packet = bytemuck::cast_slice_mut(&mut source);
					//println!("Waiting on {:?}", self.0);
					let (len, _sender) = self.0.recv_from(packet)?;
					//println!("Received");
					assert_eq!(len, source.len()*1);
				}
				/*for y in 0..target.size.y { for x in 0..target.size.x {
					let value = x*255/(target.size.x-1);
					let p = value as u32 | (value as u32)<<8 | (value as u32)<<16;
					target[xy{x,y}] = p;
				}}*/
				let min = *source.iter().min().unwrap();
				let max = *source.iter().max().unwrap();

				/*let text = format!("{:.0} {:.0} {:.0}", (min as f32/100.-273.15), (source[source.size/2] as f32/100.-273.15), (max as f32/100.-273.15));
				{
					let mut target = image::Image::<Box<[u32]>>::zero(source_size);
					let size = target.size;
					ui::text(&text).paint_fit(&mut target.as_mut(), size, xy{x: 0, y: 0});
					for y in 0..target.size.y { for x in 0..target.size.x {
						let size = source.size;
						if target[xy{x,y}]>0 { source[xy{x: size.x-1-y, y: x}] = max; }
					}}
				}*/

				assert!(target.size.x > source_size.x, "{}>{}", target.size.x, source_size.x);
				assert!(target.size.y > source_size.y, "{}>{}", target.size.x, source_size.x);
				assert_eq!(target.size.x%source_size.x, 0, "{}%{}", target.size.x, source_size.x);
				assert_eq!(target.size.y%source_size.y, 0, "{}%{}", target.size.y, source_size.y);
				assert_eq!(target.size.x/source_size.x, target.size.y/source_size.y);
				let factor = target.size.x/source_size.x;
				let stride_factor = target.stride*factor;
				let mut row = target.as_mut_ptr();
				if min >= max { println!("{min} {max}"); return; }
				//println!("{source_size} {} {factor}", target.size);
				println!("{min} {max}");
				for y in 0..source_size.y {
					{
						let mut row = row;
						for x in 0..source_size.x {
							//let value = source[xy{x: source.size.x-1-y, y: x}];
							let value = source[xy{x,y}];
							let value = (((value - min) as u32 * ((1<<8)-1)) / (max - min) as u32) as u8;
							//let value = x*255/(source_size.x-1);
							let p = value as u32 | (value as u32)<<8 | (value as u32)<<16;
							#[cfg(feature="portable_simd")] let p4 = std::simd::u32x4::splat(p);
							#[cfg(not(feature="portable_simd"))] let p4 = [p; 4];
							{
								let mut row = row;
								for _ in 0..factor { unsafe{
									{
										let mut row = row;
										for _ in 0..factor/4 {
											#[cfg(feature="portable_simd")] (row as *mut std::simd::u32x4).write_unaligned(p4);
											#[cfg(not(feature="portable_simd"))] (row as *mut [u32; 4]).write_unaligned(p4);
											row = row.add(4);
										}
										for _ in factor/4*4..factor {
											*row = p;
											row = row.add(1);
										}
									}
									row = row.add(target.stride as usize);
								}}
							}
							row = unsafe{row.add(factor as usize)};
						}
					}
					row = unsafe{row.add(stride_factor as usize)};
				}
				/*let size = target.size;
				ui::text(&text).paint_fit(&mut target, size, xy{x: 0, y: 0});*/
			}
			fn event(&mut self, _: size, _: &mut Option<EventContext>, _: &ui::Event) -> Result<bool> { Ok(true) }
		}
		let ref address = std::env::args().skip(2).next().unwrap();
		ui::run(address, &mut View(std::net::UdpSocket::bind(address)?))
	}
	#[cfg(not(feature="ui"))] unimplemented!();
	}*/
}
