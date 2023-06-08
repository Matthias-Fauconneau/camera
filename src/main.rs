//#![feature(array_methods,portable_simd)]#![allow(non_snake_case)]
fn main() {
    //tracing_subscriber::fmt().init();
    //tracing_subscriber::fmt().with_max_level(tracing::Level::TRACE).init();
    let mut camera = cameleon::u3v::enumerate_cameras().unwrap().pop().unwrap();
    println!("{:?}", camera.info());
    camera.open().unwrap();
    camera.load_context().unwrap();
    let payload_rx = camera.start_streaming(3).unwrap();
    //const SIZE : (u32, u32) = (160, 120);
    if !std::env::args().any(|arg| arg.contains("display")) {
        let socket = std::env::args().skip(2).next().map(|address| std::net::UdpSocket::bind(address).unwrap()); // 192.168.0.105:8888
        let to = std::env::args().skip(3).next().unwrap();
        //let mut index = 0;
        loop {
            //println!("{to} {}", (tv_sec as u64)*1_000_000+(tv_nsec as u64)/1000-timestamp);
            let payload = payload_rx.recv_blocking().unwrap();
            let &cameleon::payload::ImageInfo{width, height, ..} = payload.image_info().unwrap();
            use {vector::xy, image::Image};
            let source = Image::new(xy{x: width as u32, y: height as u32}, payload.image().unwrap());
            #[cfg(feature="new_uninit")] let mut target = Image::uninitialized(xy{x: 320, y: 240}); // max UDP:65,527. 1920x1200/6=320x200=64000 (next 384x240=92K. @164fps=10M/s)
            #[cfg(not(feature="new_uninit"))] let mut target = Image::zero(xy{x: 320, y: 240}); // max UDP:65,527. 1920x1200/6=320x200=64000 (next 384x240=92K. @164fps=10M/s)
            for y in 0..target.size.y { for x in 0..target.size.x {
                target[xy{x,y}] = source[xy{x: x*6, y: y*6}]; // FIXME: box
            }}
            payload_rx.send_back(payload);
            /*let data : &[u16] = bytemuck::cast_slice(&data);
            assert_eq!(data.len(), (image.width * image.height) as usize);
            let min = *data.iter().min().unwrap();
            let max = *data.iter().max().unwrap();
            //println!("{min} {max} {} {}", max-min, data.iter().scan(data[0], |predictor, &value| { let residual = value as i16-*predictor as i16; *predictor=value; Some(residual) }).map(|r| r.abs()).max().unwrap());
            let data = Box::<[_]>::from(data);
            let mut image = image::Image::new(SIZE.into(), data);
            let text = format!("{:.0} {:.0} {:.0}", (min as f32/100.-273.15), (image[image.size/2] as f32/100.-273.15), (max as f32/100.-273.15));
            {
                let image_size = image.size.yx(); // Rotated
                let mut target = image::Image::<Box<[u32]>>::zero(image_size);
                let size = target.size;
                use vector::xy;
                ui::text(&text).paint_fit(&mut target.as_mut(), size, xy{x: 0, y: 0});
                for y in 0..target.size.y { for x in 0..target.size.x {
                    let size = image.size;
                    if target[xy{x,y}]>0 { image[xy{x: size.x-1-y, y: x}] = max; }
                }}
            }*/
            //println!("{}", image.map(|row| row.iter().scan(row[0], |predictor, &value| { let residual = value as i16-*predictor as i16; *predictor=value; Some(residual) }).map(|r| r.abs()).max().unwrap()).max().unwrap()); // 1+9bit
            //println!("{}", image.map(|row| row.iter().scan(row[0], |predictor, &value| { let residual = value as i16-*predictor as i16; *predictor=value; Some(residual) }).map(|r| 1+r.abs() as usize).sum::<usize>()).sum::<usize>()/(SIZE.0*SIZE.1) as usize); // 31
            /*fn ceil_log2(x: usize) -> u8 { ((x-1)<<1).ilog2() as u8 }
            println!("{}", image.map(|row| row.iter().scan(row[0], |predictor, &value| { let residual = value as i16-*predictor as i16; *predictor=value; Some(residual) }).map(|r:i16| {
                let u = if r > 0 { ((r as u16)<<1)-1 } else { ((-r) as u16)<<1 };
                const k : usize = 4;
                let u = u >> k;
                //let u2 = ((r as u16)<<1) ^ ((r as u16)>>15);
                //assert_eq!(u, u2, "{r} {u} {u2}");
                let e : u16 = u + 1;
                assert!(e > 0, "{r} {u}");
                let b = 16 - e.leading_zeros(); // ceil_ilog2
                //println!("{e} {b}");
                (b-1+b) as usize + k
            }).sum::<usize>()).sum::<usize>() as f32/(SIZE.0*SIZE.1) as usize as f32); // 8*/
            //data[0..8].copy_from_slice(&timestamp.to_ne_bytes());
            //println!("send {}", data.len());
            if let Some(socket) = socket.as_ref() {
                socket.send_to(/*bytemuck::cast_slice(&image.data)*/&target.data, &to).unwrap(); // 192.168.0.104:6666
                //println!("sent");
            }
            //println!("queue");
            //index = (index+1)%count;
        }
    } else { #[cfg(feature="display")] {
        println!("{}", local_ip_address::local_ip()?);
        struct View(std::net::UdpSocket);
        use ui::*;
        impl Widget for View {
            #[throws] fn paint(&mut self, target: &mut Target, _: size, _: int2) {
                let mut source = image::Image::<Box<[u16]>>::zero(SIZE.into());
                let source_size = source.size.yx(); // Rotated
                let [num, den] = if source_size.x*target.size.y > source_size.y*target.size.x { [target.size.x, source_size.x] } else { [target.size.y, source_size.y] };
                //let target_size = source.size*num/den;
                let target_size = source_size*(num/den); // largest integer fit
                let mut target = target.slice_mut((target.size-target_size)/2, target_size);
                if true {
                    let packet = bytemuck::cast_slice_mut(&mut source);
                    let (len, _sender) = self.0.recv_from(packet)?;
                    assert_eq!(len, source.len()*2);
                    //let timestamp = u64::from_ne_bytes(packet[0..8].try_into().unwrap());
                    //let linux::general::__kernel_timespec{tv_sec,tv_nsec} = rustix::time::clock_gettime(rustix::time::ClockId::Monotonic);
                    //println!("{}", (tv_sec as u64)*1_000_000+(tv_nsec as u64)/1000-timestamp);
                }
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

                assert_eq!(target.size.x%source_size.x, 0, "{}%{}", target.size.x, source_size.x);
                assert_eq!(target.size.y%source_size.y, 0, "{}%{}", target.size.y, source_size.y);
                assert_eq!(target.size.x/source_size.x, target.size.y/source_size.y);
                let factor = target.size.x/source_size.x;
                let stride_factor = target.stride*factor;
                let mut row = target.as_mut_ptr();
                if min < max { match factor {
                    N@15 => {
                        for y in 0..source_size.y {
                            {
                                let mut row = row;
                                for x in 0..source_size.x {
                                    let value = source[xy{x: source.size.x-1-y, y: x}];
                                    let value = (((value - min) as u32 * ((1<<8)-1)) / (max - min) as u32) as u8;
                                    let p = value as u32 | (value as u32)<<8 | (value as u32)<<16;
                                    let p8 =  std::simd::u32x8::splat(p);
                                    let p4 =  std::simd::u32x4::splat(p);
                                    let p2 =  std::simd::u32x2::splat(p);
                                    {
                                        let mut row = row;
                                        for _ in 0..N { unsafe{
                                            (row as *mut std::simd::u32x8).write_unaligned(p8);
                                            (row.add(8) as *mut std::simd::u32x4).write_unaligned(p4);
                                            (row.add(8+4) as *mut std::simd::u32x2).write_unaligned(p2);
                                            *row.add(8+4+2) = p;
                                            row = row.add(target.stride as usize);
                                        }}
                                    }
                                    row = unsafe{row.add(factor as usize)};
                                }
                            }
                            row = unsafe{row.add(stride_factor as usize)};
                        }
                    },
                    N@10 => {
                        for y in 0..source_size.y {
                            {
                                let mut row = row;
                                for x in 0..source_size.x {
                                    let value = source[xy{x: source.size.x-1-y, y: x}];
                                    let value = (((value - min) as u32 * ((1<<8)-1)) / (max - min) as u32) as u8;
                                    let p = value as u32 | (value as u32)<<8 | (value as u32)<<16;
                                    let p8 =  std::simd::u32x8::splat(p);
                                    let p2 =  std::simd::u32x2::splat(p);
                                    {
                                        let mut row = row;
                                        for _ in 0..N { unsafe{
                                            (row as *mut std::simd::u32x8).write_unaligned(p8);
                                            (row.add(8) as *mut std::simd::u32x2).write_unaligned(p2);
                                            row = row.add(target.stride as usize);
                                        }}
                                    }
                                    row = unsafe{row.add(factor as usize)};
                                }
                            }
                            row = unsafe{row.add(stride_factor as usize)};
                        }
                    },
                    N => {
                        println!("{N}");
                        for y in 0..source_size.y {
                            {
                                let mut row = row;
                                for x in 0..source_size.x {
                                    let value = source[xy{x: source.size.x-1-y, y: x}];
                                    let value = (((value - min) as u32 * ((1<<8)-1)) / (max - min) as u32) as u8;
                                    let p = value as u32 | (value as u32)<<8 | (value as u32)<<16;
                                    let p4 =  std::simd::u32x4::splat(p);
                                    {
                                        let mut row = row;
                                        for _ in 0..N { unsafe{
                                            {
                                                let mut row = row;
                                                for _ in 0..N/4 {
                                                    (row as *mut std::simd::u32x4).write_unaligned(p4);
                                                    row = row.add(4);
                                                }
                                                for _ in N/4*4..N {
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
                    }
                }}
                /*let size = target.size;
                ui::text(&text).paint_fit(&mut target, size, xy{x: 0, y: 0});*/
            }
            fn event(&mut self, _: size, _: &mut Option<EventContext>, _: &ui::Event) -> Result<bool> { Ok(true) }
        }
        let ref address = std::env::args().skip(2).next().unwrap();
        ui::run(address, &mut View(std::net::UdpSocket::bind(address)?))
    }
    #[cfg(not(feature="display"))] unimplemented!();
    }
}
