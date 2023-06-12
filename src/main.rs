//#![feature(generic_arg_infer)]
#![cfg_attr(feature="portable_simd", feature(portable_simd))]
use std::ptr::{null,null_mut};//{vector::xy, image::Image};
#[repr(C)] pub struct CameraDescriptor {
	id: u64,
	r#type: u32,
	model: [u8; 64],
	serial: [u8; 64],
	name: [u8; 64],
	reserved: [u8; 448]
}
//impl Default for CameraDescriptor { fn default() -> Self { Self{id:0, r#type:0, model:[0;_], serial:[0;_], name: [0;_], reserved: [0:_]}}}
impl Default for CameraDescriptor { fn default() -> Self { Self{id:0, r#type:0, model:[0;64], serial:[0;64], name: [0;64], reserved: [0;448]}}}
// C:\Program Files\IDS\ids_peak\comfort_sdk\api\{include\ids_peak_comfort_c/ids_peak_comfort_c.h, lib/x86_64/ids_peak_comfort_c.dll}
#[repr(C)] pub struct Camera { _data: [u8; 0], _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>}
#[repr(C)] pub struct Frame { _data: [u8; 0], _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>}
//#[link(name="ids_peak_comfort_c", kind="raw-dylib")] extern "C" { 
#[link(name="ids_peak_comfort_c")] extern "C" { 
	#[link_name="peak_Library_Init"] pub fn Library_init() -> u32;
	#[link_name="peak_CameraList_Update"] pub fn CameraList_update(len: *const usize) -> u32; 
	#[link_name="peak_CameraList_Get"] pub fn CameraList_get(buffer: *mut CameraDescriptor, len: *mut usize) -> u32; 
	#[link_name="peak_Camera_Open"] pub fn Camera_open(id: u64, camera: *mut *const Camera) -> u32;
	#[link_name="peak_Acquisition_Start"] pub fn start(camera: *const Camera, number_of_frames: u32) -> u32;
	#[link_name="peak_Acquisition_WaitForFrame"] pub fn wait_for_frame(camera: *const Camera, timeout_ms: u32, frame: *mut *const Frame) -> u32;
}

fn main() {//-> Result<(), Box<dyn std::error::Error>> {
  assert_eq!(unsafe{Library_init()}, 0);
	assert_eq!(unsafe{CameraList_update(null_mut())}, 0);
  let mut cameras_len = 0;
  assert_eq!(unsafe{CameraList_get(null_mut(), &mut cameras_len as *mut _)}, 0);
	assert!(cameras_len >= 1, "{cameras_len}");
  let mut cameras = Box::from_iter(std::iter::from_fn(|| Default::default()).take(cameras_len));
  unsafe{CameraList_get(cameras.as_mut_ptr(), &mut cameras_len as *mut _)};
	let mut camera = null();
	unsafe{Camera_open(cameras[0].id, &mut camera as *mut _)};
  //peak_Camera_ResetToDefaultSettings(hCam);
  //double currentFramerate = 0.0;  // Used in Acquisition to calculate timeout
  //status = peak_FrameRate_GetRange(hCam, &minFramerate, &maxFramerate, &incFramerate);
  //status = peak_FrameRate_Set(hCam, maxFramerate);
  //status = peak_FrameRate_Get(hCam, &currentFramerate);
  unsafe{start(camera, 1)};
	let mut frame = null();
	unsafe{wait_for_frame(camera, 1000, &mut frame as *mut _)};
	//peak_Frame_Release(hCam, hFrame);
	/*//const SIZE : xy<u32> = xy{x: 320, y: 200};
	const SIZE : xy<u32> = xy{x: 160, y: 100};
	if !std::env::args().any(|arg| arg.contains("ui")) {
		let socket = std::env::args().skip(2).next().map(|address| std::net::UdpSocket::bind(address).unwrap()); // 192.168.0.105:8888
		let to = std::env::args().skip(3).next().unwrap();
		let mut camera = cameleon::u3v::enumerate_cameras().unwrap().pop().unwrap();
		camera.open().unwrap();
		camera.load_context().unwrap();
		//let params_ctxt = camera.params_ctxt().unwrap();
		//dbg!(params_ctxt.ctxt);
		let payload_rx = camera.start_streaming(1).unwrap();
		loop {
			let payload = payload_rx.recv_blocking().unwrap();
			//let min = *payload.image().unwrap().iter().min().unwrap();
			//let max = *payload.image().unwrap().iter().max().unwrap();
			//println!("{min} {max}");
			let &cameleon::payload::ImageInfo{width, height, ..} = payload.image_info().unwrap();
			let source = Image::new(xy{x: width as u32, y: height as u32}, payload.image().unwrap());
			#[cfg(feature="new_uninit")] let mut target = Image::uninitialized(SIZE); // max UDP:65,527. 1920x1200/6=320x200=64000 (next 384x240=92K. @164fps=10M/s)
			#[cfg(not(feature="new_uninit"))] let mut target = Image::zero(SIZE); // max UDP:65,527. 1920x1200/6=320x200=64000 (next 384x240=92K. @164fps=10M/s)
			//for y in 0..target.size.y { for x in 0..target.size.x { target[xy{x,y}] = source[xy{x: x*6, y: y*6}]; /*FIXME: box*/ } }
			for y in 0..target.size.y { for x in 0..target.size.x { target[xy{x,y}] = source[xy{x: x*12, y: y*12}]; /*FIXME: box*/ } }
			payload_rx.send_back(payload);
			if let Some(socket) = socket.as_ref() { socket.send_to(&target.data, &to).unwrap(); /*println!("Sent");*/ }
		}
	} else { #[cfg(feature="ui")] {
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
