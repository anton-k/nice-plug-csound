use std::ops::ControlFlow;

use quick_xml::Reader;
use quick_xml::events::Event;

pub struct CsdSettings {
    pub out_size: usize,
    pub in_size: usize,
}

impl Default for CsdSettings {
    fn default() -> Self {
        CsdSettings {
            out_size: 2,
            in_size: 2,
        }
    }
}

pub fn parse_csd_settings(csd_file_content: &str) -> Option<CsdSettings> {
    let instruments_str = get_csd_instruments_text(csd_file_content)?;
    let mut out_size_var = None;
    let mut in_size_var = None;
    let _ = instruments_str.trim_start().lines().try_for_each(|line| {
        let trimmed_line = line.replace([' ', '\t'], "");
        let parts: Vec<&str> = trimmed_line.split("=").collect();
        if trimmed_line.starts_with("instr") && parts.len() == 1 {
            ControlFlow::Break(())
        } else {
            if parts.len() == 2 {
                let name = parts[0];
                let value = parts[1];

                if name == "nchnls"
                    && let Ok(out_val) = value.parse::<usize>()
                {
                    out_size_var = Some(out_val);
                }

                if name == "nchnls_i"
                    && let Ok(in_val) = value.parse::<usize>()
                {
                    in_size_var = Some(in_val);
                }
            }
            ControlFlow::Continue(())
        }
    });

    let out_size = out_size_var?;
    Some(CsdSettings {
        out_size,
        in_size: in_size_var.unwrap_or(0),
    })
}

fn get_csd_instruments_text(csd_file_content: &str) -> Option<String> {
    let mut txt = Vec::new();
    let mut reader = Reader::from_str(csd_file_content);
    let mut buf = Vec::new();
    let mut is_instruments_tag = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) if e.name().as_ref() == b"CsInstruments" => {
                is_instruments_tag = true;
            }
            Ok(Event::Text(e)) if is_instruments_tag => {
                txt.push(e.decode().unwrap().into_owned());
            }
            Ok(Event::End(_e)) if is_instruments_tag => break,
            Ok(Event::Eof) => break,
            _ => {}
        }
        buf.clear();
    }
    if txt.is_empty() {
        None
    } else {
        Some(txt[0].clone())
    }
}
