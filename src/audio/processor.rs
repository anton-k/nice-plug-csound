use crate::audio::buffer::ProcessorBuffers;
use crate::params::CsoundParams;
use std::sync::Arc;

use csound_dyn::{ChannelName, Csound};
use nice_plug::{params::Param, prelude::*};
use std::path::Path;

pub struct CsoundAudioProcessor {
    csound: Csound,
    channel_names: Vec<ChannelName>,
    buffers: ProcessorBuffers<f64>,
    csound_frame_size: usize,
    zero_dbfs: f32,
    inverse_zero_dbfs: f32,
    frame_size: usize,
}

impl CsoundAudioProcessor {
    pub fn new<P: CsoundParams>(
        params: &Arc<P>,
        csd: &str,
        sample_rate: usize,
        channel_names: &[ChannelName],
    ) -> Self {
        let mut csound = init_csound(csd, sample_rate);
        let mut buffers = ProcessorBuffers::new(8000, 0.0);
        let ksmps = csound.get_ksmps_unsafe();
        let frame_size = 2 * ksmps as usize;
        // can safely advance cursor as buffers are initialized with zeroes.
        buffers.advance_output_write_cursor(frame_size);
        let zero_dbfs = csound.get_0dbfs_unsafe() as f32;
        let inverse_zero_dbfs = zero_dbfs.recip();
        let channel_names = channel_names.to_vec();
        update_csound_params(params, &mut csound, &channel_names);
        Self {
            csound,
            buffers,
            csound_frame_size: ksmps as usize,
            channel_names,
            zero_dbfs,
            inverse_zero_dbfs,
            frame_size,
        }
    }

    pub fn process<P: CsoundParams, Plug: Plugin>(
        &mut self,
        audio: &mut Buffer,
        params: &Arc<P>,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Plug>,
    ) -> ProcessStatus {
        // TODO: why without this line parameters are not updated in the Csound?
        self.update_csound_params(params);
        let num_samples = audio.samples();

        let mut block_start = 0;
        let mut block_end = num_samples;
        let mut next_event = context.next_event();

        while block_start < num_samples {
            'events: loop {
                match &next_event {
                    Some(event) if (event.timing() as usize) <= block_start => {
                        self.handle_event(event);
                        next_event = context.next_event();
                    }
                    Some(event) if (event.timing() as usize) < block_end => {
                        block_end = event.timing() as usize;
                        break 'events;
                    }
                    _ => break 'events,
                }
            }
            self.process_batch(audio, block_start, block_end);
            block_start = block_end;
            block_end = num_samples;
        }

        ProcessStatus::Normal
    }

    pub fn reset(&mut self) {
        self.buffers.reset();
        self.buffers.advance_output_write_cursor(self.frame_size);
    }

    // TODO: complete event handlers
    fn handle_event<P>(&mut self, event: &NoteEvent<P>) {
        match event {
            NoteEvent::NoteOn {
                timing,
                voice_id,
                channel,
                note,
                velocity,
            } => {}
            NoteEvent::NoteOff {
                timing,
                voice_id,
                channel,
                note,
                velocity,
            } => {}
            NoteEvent::Choke {
                timing,
                voice_id,
                channel,
                note,
            } => {}
            NoteEvent::VoiceTerminated {
                timing,
                voice_id,
                channel,
                note,
            } => {}
            NoteEvent::PolyModulation {
                timing,
                voice_id,
                poly_modulation_id,
                normalized_offset,
            } => {}
            NoteEvent::MonoAutomation {
                timing,
                poly_modulation_id,
                normalized_value,
            } => {}
            _ => {}
        }
    }

    fn process_batch(&mut self, audio: &mut Buffer, block_start: usize, block_end: usize) {
        self.write_daw_input_to_buffer(audio, block_start, block_end);
        self.csound_audio_processing();
        self.read_daw_output_from_buffer(audio, block_start, block_end);
    }

    pub fn update_csound_params<P: CsoundParams>(&mut self, params: &Arc<P>) {
        update_csound_params(params, &mut self.csound, &self.channel_names);
    }

    fn write_daw_input_to_buffer(
        &mut self,
        audio: &mut Buffer,
        block_start: usize,
        block_end: usize,
    ) {
        let batch_size = block_end - block_start;
        let outs = audio.as_slice();
        for index in 0..batch_size {
            outs.iter().enumerate().for_each(|(channel, out)| {
                self.buffers.write_input(
                    (self.zero_dbfs * out[block_start + index]) as f64,
                    index * 2 + channel,
                );
            });
        }
        let total_samples_size = batch_size * 2;
        self.buffers.advance_input_write_cursor(total_samples_size);
    }

    fn csound_audio_processing(&mut self) {
        let input_capacity = self.buffers.input_size() / 2;
        let csound_cycle_size = input_capacity / self.csound_frame_size;

        for _ in 0..csound_cycle_size {
            // fill csound inputs from buffer
            let spin = self.csound.get_spin_unsafe();
            self.buffers.read_input_into(spin);

            // perform csound
            self.csound.perform_ksmps_unsafe();

            // read csound outputs to buffer
            let spout = &self.csound.get_spout_unsafe();
            self.buffers.write_output_latest(spout);
        }
    }

    fn read_daw_output_from_buffer(
        &mut self,
        audio: &mut Buffer,
        block_start: usize,
        block_end: usize,
    ) {
        let batch_size = block_end - block_start;
        let outs = audio.as_slice();
        for index in 0..batch_size {
            outs.iter_mut().enumerate().for_each(|(channel, out)| {
                out[block_start + index] =
                    self.inverse_zero_dbfs * self.buffers.read_output(index * 2 + channel) as f32
            });
        }
        let total_samples_size = batch_size * 2;
        self.buffers.advance_output_read_cursor(total_samples_size);
    }
}

pub fn update_csound_params<P: CsoundParams>(
    params: &Arc<P>,
    csound: &mut Csound,
    channel_names: &[ChannelName],
) {
    channel_names.iter().for_each(|channel_name| {
        if let Some(param) = params.get(channel_name.to_str()) {
            csound
                .set_control_channel(channel_name, param.modulated_normalized_value() as f64)
                .ok();
        }
    });
}

pub fn init_csound(csd: &str, sample_rate: usize) -> Csound {
    let mut csound =
        Csound::new(Path::new("/usr/local/lib/libcsound64.so")).expect("Failed to init Csound");
    csound
        .set_option(format!("-n -d -+rtmidi=NULL -M0 -sr {}", sample_rate).as_str())
        .expect("Failed to set sample rate");
    csound.compile_csd_from_str(csd, true).unwrap();
    csound.start().expect("Failed to start Csound");
    csound
}
