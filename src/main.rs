#![cfg_attr(feature="portable_simd", feature(portable_simd))]
fn main() {
	use vector::xy;
	let camera = std::env::args().skip(1).next().unwrap();
	// max UDP: 65,527
	let (size, _factor, port) = match camera.as_str() {
		"uvc" => (xy{x: 256, y: 192}, 0, 1024), //256x192=49152 @25fps=1M/s
		//"ids" => (xy{x: 288, y: 216}, 9, 1025), //2592x1944/9=288x216=62208 @48fps=3M/s (next 324x243=79K. @48fps=4M/s)
		"ids" => (xy{x: 144, y: 108}, 18, 1025), //2592x1944/9=288x216=62208 @48fps=3M/s (next 324x243=79K. @48fps=1M/s)
		"bsl" => (xy{x: 160, y: 100}, 12, 1026), //1920x1200/12=160x100=16000 @164fps=2M/s
		//"bsl" => (xy{x: 320, y: 200}, 1026); //1920x1200/6=320x200=64000 @164fps=6M/s (next 384x240=92K. @164fps=10M/s)
		_ => unimplemented!()
	};
	#[cfg(not(feature="ui"))] {
		let address = std::env::args().skip(2).next().unwrap();
		let socket = std::net::UdpSocket::bind((address, port)).unwrap();
		let to = (std::env::args().skip(3).next().unwrap(), port);
		match camera.as_str() {
		_camera @ ("ids"|"bsl") => {
			//std::env::set_var("GENICAM_GENTL64_PATH","/usr/lib/ids/cti");
			let mut cameras = cameleon::u3v::enumerate_cameras().unwrap();
			for camera in &cameras { println!("{:?}", camera.info()); }
			//let mut camera = cameras.swap_remove(match camera {"bsl" => 0, "ids" => 1, _ => unreachable!()});
			let ref mut camera = cameras[0]; // find(|c| c.info().contains("U3-368xXLE-NIR")).unwrap()
			camera.open().unwrap();
			camera.load_context().unwrap();
			//let mut params_ctxt = camera.params_ctxt().unwrap();
    	//let gain_node = params_ctxt.node("Gain").unwrap().as_float(&params_ctxt).unwrap();
			//if gain_node.is_writable(&mut params_ctxt).unwrap() { gain_node.set_value(&mut params_ctxt, 0.1_f64).unwrap(); }
			/*let exposure_time = params_ctxt.node("ExposureTime").unwrap().as_float(&params_ctxt).unwrap();
			println!("{}us", exposure_time.value(&mut params_ctxt).unwrap()); // 15ms=66Hz
			let acquisition_frame_rate = params_ctxt.node("AcquisitionFrameRate").unwrap().as_float(&params_ctxt).unwrap(); // 30fps
			println!("{}", acquisition_frame_rate.value(&mut params_ctxt).unwrap());*/
			let payload_rx = camera.start_streaming(3).unwrap();
			for _i in 0.. {
				let Ok(payload) = payload_rx.recv_blocking() else { continue; };
				//println!("blocking"); let Ok(payload) = payload_rx.recv_blocking() else { println!("continue"); continue; }; println!("ok");
				let &cameleon::payload::ImageInfo{width, height, ..} = payload.image_info().unwrap();
				let source = Image::new(xy{x: width as u32, y: height as u32}, payload.image().unwrap());
				use image::Image;
				#[cfg(feature="new_uninit")] let mut target = Image::uninitialized(size); 
				#[cfg(not(feature="new_uninit"))] let mut target = Image::zero(size);
				for y in 0..target.size.y { for x in 0..target.size.x { target[xy{x,y}] = source[xy{x: x*_factor, y: y*_factor}]; /*FIXME: box*/ } }
				let [min, max] = [*source.iter().min().unwrap(), *source.iter().max().unwrap()];
				println!("{_i} {min} {max}"); //println!("{min} {max} {} {}", source.size, target.size);
				payload_rx.send_back(payload);
				socket.send_to(&target.data, &to).unwrap();
				//println!("{i}");
			}
		}
		#[cfg(feature="uvc")] "uvc" => {
			use std::ptr::null_mut;
			use uvc::*;
			let mut uvc = null_mut();
			assert!(unsafe{uvc_init(&mut uvc as *mut _, null_mut())} >= 0);
			let mut device = null_mut();
			assert!(unsafe{uvc_find_device(uvc, &mut device as *mut _, 0, 0, std::ptr::null())} >= 0);
			let mut device_descriptor : *mut uvc_device_descriptor_t = null_mut();
			assert!(unsafe{uvc_get_device_descriptor(device, &mut device_descriptor as &mut _)} >= 0);
			assert!(!device_descriptor.is_null());
			let device_descriptor = unsafe{*device_descriptor};
			println!("{} {} {}", device_descriptor.idVendor, device_descriptor.idProduct, device_descriptor.bcdUVC);
			if !device_descriptor.serialNumber.is_null() { println!("{:?}", unsafe{std::ffi::CStr::from_ptr(device_descriptor.serialNumber)}); }
			if !device_descriptor.manufacturer.is_null() { println!("{:?}", unsafe{std::ffi::CStr::from_ptr(device_descriptor.manufacturer)}); }
			if !device_descriptor.product.is_null() { println!("{:?}", unsafe{std::ffi::CStr::from_ptr(device_descriptor.product)}); }
			let mut device_handle = null_mut();
			assert!(unsafe{uvc_open(device, &mut device_handle as *mut _)} >= 0);
			let mut control = unsafe{std::mem::zeroed()};
			assert!(unsafe{uvc_get_stream_ctrl_format_size(device_handle, &mut control as *mut _, uvc_frame_format_UVC_FRAME_FORMAT_ANY, 256, 192, 25)} >= 0);
			let mut stream = null_mut();
			assert!(unsafe{uvc_stream_open_ctrl(device_handle, &mut stream as *mut _, &mut control as *mut _)} >= 0);
			assert!(unsafe{uvc_stream_start(stream, None, null_mut(), 0)} >= 0);
			loop {
				let mut frame : *mut uvc_frame_t = null_mut();
				assert!(unsafe{uvc_stream_get_frame(stream, &mut frame as *mut _, 1000000)} >= 0);
				assert!(!frame.is_null());
				let frame = unsafe{*frame};
				let source = unsafe{std::slice::from_raw_parts(frame.data as *const u16, (frame.data_bytes/2) as usize)};
				let min = *source.iter().min().unwrap();
				let max = *source.iter().max().unwrap();
				if min<max {
					let target = Box::from_iter(source.iter().map(|s| (((s-min) as u32) * 0xFF / (max-min) as u32) as u8));
					assert_eq!(target.len(), (size.x*size.y) as usize);
					socket.send_to(&target, &to).unwrap();
				}
			}
		}
		_ => unimplemented!()
		}
	}
	#[cfg(feature="ui")] {
		struct View {
			socket: std::net::UdpSocket,
			size: size,
			save: bool,
		}
		use ui::*;
		impl Widget for View {
			#[throws] fn paint(&mut self, target: &mut Target, _: size, _: int2) {
				#[cfg(feature="new_uninit")] let mut source = image::Image::<Box<[u8]>>::uninitialized(self.size);
				#[cfg(not(feature="new_uninit"))] let mut source = image::Image::<Box<[u8]>>::zero(self.size);
				let [num, den] = if source.size.x*target.size.y > source.size.y*target.size.x { [target.size.x, source.size.x] } else { [target.size.y, source.size.y] };
				let target_size = source.size*(num/den); // largest integer fit
				let mut target = target.slice_mut((target.size-target_size)/2, target_size);
				let (len, _sender) = self.socket.recv_from(bytemuck::cast_slice_mut(&mut source))?;
				assert_eq!(len, source.len()*std::mem::size_of::<u8>());
				if self.save { 
					self.save = false;
					let path = format!("{}.png",chrono::Local::now().format("%m-%d_%H.%M"));
					println!("{path}");
					imagers::save_buffer(path, &source.data, source.size.x, source.size.y, imagers::ColorType::L8).unwrap();
				}

				let min = *source.iter().min().unwrap();
				let max = *source.iter().max().unwrap();
				//println!("{min} {max}");

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

				assert!(target.size.x > source.size.x, "{}>{}", target.size.x, source.size.x);
				assert!(target.size.y > source.size.y, "{}>{}", target.size.x, source.size.x);
				assert_eq!(target.size.x%source.size.x, 0, "{}%{}", target.size.x, source.size.x);
				assert_eq!(target.size.y%source.size.y, 0, "{}%{}", target.size.y, source.size.y);
				assert_eq!(target.size.x/source.size.x, target.size.y/source.size.y);
				let factor = target.size.x/source.size.x;
				let stride_factor = target.stride*factor;
				let mut row = target.as_mut_ptr();
				if min >= max { return; }
				for y in 0..source.size.y {
					{
						let mut row = row;
						for x in 0..source.size.x {
							let value = source[xy{x,y}];
							let value = (((value - min) as u32 * ((1<<8)-1)) / (max - min) as u32) as u8;
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
			fn event(&mut self, _: size, _: &mut Option<EventContext>, event: &ui::Event) -> Result<bool> { 
				if let ui::Event::Key(_key/*@' '*/) = event { /*println!("⎙");*/ self.save=true; }
				Ok(true) 
			}
		}
		let address = std::env::args().skip(2).next().unwrap();
		ui::run(&camera, &mut View{socket: std::net::UdpSocket::bind((address, port)).unwrap(), size, save: false}).unwrap();
	}
}
