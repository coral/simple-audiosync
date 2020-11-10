extern crate anyhow;
extern crate cpal;
use aubio_rs::{Tempo, Onset};
use ableton_link::{Link, SessionState};

extern crate rosc;

use rosc::encoder;
use rosc::{OscMessage, OscPacket, OscType};
use std::net::{ UdpSocket};

use cpal::traits::{DeviceTrait, HostTrait};

fn main() {
    enm();
}

pub struct BeatDetect {
    pub onset: Onset,
    pub tempo: Tempo,
    pub socket: UdpSocket,
    pub lnk: Link,
}

impl BeatDetect {
    pub fn new(bufsize: usize, hopsize: usize, samplerate: u32) -> BeatDetect {
        BeatDetect {
            onset: Onset::new(aubio_rs::OnsetMode::SpecFlux, bufsize, hopsize, samplerate).unwrap(),
            tempo: Tempo::new(aubio_rs::OnsetMode::SpecFlux,bufsize, hopsize, samplerate).unwrap(),
            socket: UdpSocket::bind("0.0.0.0:4000").unwrap(),
            lnk: Link::new(120.0),
        }
    }

    fn process(&mut self, input: &[f32], denis: &cpal::InputCallbackInfo) {
        let m = self.tempo.do_result(input).unwrap();

        if m > 0.0 {
            println!("Tempo: {:?}", self.tempo.get_bpm());
            println!("Confidence: {:?}", self.tempo.get_confidence());

            if self.tempo.get_confidence() > 0.2 {
                self.lnk.with_app_session_state(|mut state:SessionState|{
                    state.set_tempo(self.tempo.get_bpm() as f64, 0);
                    state.commit();
                });
            }
            
            let msg_buf = encoder::encode(&OscPacket::Message(OscMessage {
                addr: "/3".to_string(),
                args: vec![OscType::Float(1.0)],
                }))
                .unwrap();
        
            self.socket.send_to(&msg_buf, "127.0.0.1:8000").unwrap();
        }

        let ostResult = self.onset.do_result(input).unwrap();

        if ostResult > 0.0 {
           // println!("Onset: {:?}", ostResult); 
        } 
    }
}


fn enm() -> Result<(), anyhow::Error>  {

    let mut prc = BeatDetect::new(512,  256, 48000);
    prc.lnk.enable(true);




    let host = cpal::default_host();
    
    let device = host
    .default_input_device()
    .expect("Failed to get default input device");
    println!("Default input device: {}", device.name()?);
    
    println!("{:?}", device.name());
    
    let config = &cpal::StreamConfig {
        channels: 1,
        buffer_size: cpal::BufferSize::Fixed(512),
        sample_rate: cpal::SampleRate(48_000)
    };
    
    let err_fn = move |err| {
        eprintln!("an error occurred on stream: {}", err);
    };
    
    let stream = device.build_input_stream(config,
        move |data, inp: &cpal::InputCallbackInfo| prc.process(data, inp),
        err_fn);
        
        std::thread::sleep(std::time::Duration::from_secs(120));
        drop(stream);
        
        Ok(())
}
    
    