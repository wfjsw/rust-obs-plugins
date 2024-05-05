use std::{borrow::Borrow, ptr::null_mut};

use obs_sys::{audio_output_get_channels, audio_output_get_sample_rate, audio_t, obs_audio_data, MAX_AV_PLANES};

pub struct AudioDataContext {
    pointer: *mut obs_audio_data,
}

impl AudioDataContext {
    pub fn from_raw(pointer: *mut obs_audio_data) -> Self {
        Self { pointer }
    }

    pub fn frames(&self) -> usize {
        unsafe {
            self.pointer
                .as_ref()
                .expect("Audio pointer was null!")
                .frames as usize
        }
    }

    pub fn channels(&self) -> usize {
        unsafe {
            self.pointer
                .as_ref()
                .expect("Audio pointer was null!")
                .data
                .len()
        }
    }

    pub fn timestamp(&self) -> u64 {
        unsafe {
            self.pointer
                .as_ref()
                .expect("Audio pointer was null!")
                .timestamp
        }
    }

    pub fn set_timestamp(&mut self, timestamp: u64) {
        unsafe {
            self.pointer
                .as_mut()
                .expect("Audio pointer was null!")
                .timestamp = timestamp;
        }
    }

    pub fn get_channel_as_mut_slice(&self, channel: usize) -> Option<&'_ mut [f32]> {
        unsafe {
            let data = self.pointer.as_ref()?.data;

            if channel >= data.len() {
                return None;
            }

            let frames = self.pointer.as_ref()?.frames;

            Some(core::slice::from_raw_parts_mut(
                data[channel] as *mut f32,
                frames as usize,
            ))
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioInfo {
    pub sample_rate: usize,
    pub channels: usize,
}

pub struct AudioRef {
    pub pointer: *mut audio_t,
}

impl AudioRef {
    pub fn from_raw(pointer: *mut audio_t) -> Self {
        Self { pointer }
    }

    pub fn info(&self) -> AudioInfo {
        AudioInfo {
            sample_rate: self.sample_rate(),
            channels: self.channels(),
        }
    }

    pub fn sample_rate(&self) -> usize {
        unsafe { audio_output_get_sample_rate(self.pointer) as usize }
    }

    pub fn channels(&self) -> usize {
        unsafe { audio_output_get_channels(self.pointer) }
    }
}


pub struct AudioData {
    pub data: Vec<Vec<f32>>,
    pub frames: usize,
    pub timestamp: u64,
}

impl AudioData {
    pub fn new(channels: usize, frames: usize, timestamp: u64) -> Self {
        let mut data = Vec::with_capacity(channels);
        for _ in 0..channels {
            data.push(vec![0.0; frames]);
        }

        Self {
            data,
            frames,
            timestamp,
        }
    }

    pub fn new_like(data: &AudioDataContext) -> Self {
        let channels = data.channels();
        let frames = data.frames();
        let mut data = Vec::with_capacity(8);

        for _ in 0..channels {
            data.push(vec![0.0; frames]);
        }

        Self {
            data,
            frames,
            timestamp: 0,
        }
    }

    pub(crate) fn to_raw(&self) -> obs_audio_data {
        let mut data: [*mut u8; MAX_AV_PLANES as usize] = [null_mut(); MAX_AV_PLANES as usize];
        for (i, channel) in self.data.iter().enumerate() {
            data[i] = channel.as_ptr() as *mut u8;
        }

        obs_audio_data {
            data,
            frames: self.frames as u32,
            timestamp: self.timestamp,
        }
    }
}
