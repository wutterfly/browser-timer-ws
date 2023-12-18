use std::{
    io::{BufWriter, Write},
    path::PathBuf,
};

use crate::message::EventTyp;

#[derive(Debug)]
pub struct DataHolder {
    buffer: Vec<Data>,
    out_dir: PathBuf,
    counter: usize,
    file_counter: usize,
    write_on: Option<WriteOn>,
    out_prefix: &'static str,
    rtt: u128,
}

impl DataHolder {
    pub fn new(
        write_on: Option<WriteOn>,
        out_dir: PathBuf,
        out_prefix: &'static str,
    ) -> std::io::Result<Self> {
        std::fs::create_dir_all(&out_dir)?;

        Ok(Self {
            buffer: Vec::with_capacity(10_000),
            counter: 0,
            out_dir,
            file_counter: 0,
            write_on,
            out_prefix,
            rtt: 0,
        })
    }

    pub fn update_rtt(&mut self, rtt: u128) {
        self.rtt = rtt;
    }

    pub fn push(&mut self, value: Data) -> std::io::Result<()> {
        log::trace!("value: {value:?}");
        // if write out
        if let Some(on) = &self.write_on {
            match on {
                WriteOn::Count(count) if *count != self.counter => {}
                WriteOn::Filter(filter) if !filter(&value) => {}
                _ => {
                    log::info!("Writing data to file");
                    self.buffer.push(value);

                    return self.flush();
                }
            }
        }

        self.buffer.push(value);
        self.counter += 1;

        Ok(())
    }

    pub fn flush(&mut self) -> std::io::Result<()> {
        let file_name = format!("{}_{}.csv", self.out_prefix, self.file_counter);
        let path = PathBuf::from(&self.out_dir).join(file_name);

        let f = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)?;

        let mut out = BufWriter::new(f);

        out.write_fmt(format_args!("key,timestamp,key_code,typ,rtt\n"))?;

        for data in &self.buffer {
            out.write_fmt(format_args!(
                "{},{},{},{},{}\n",
                data.key,
                data.timestamp,
                data.key_code,
                data.typ.as_str(),
                self.rtt
            ))?;
        }

        out.flush()?;
        self.buffer.clear();
        self.file_counter += 1;

        return Ok(());
    }
}

#[derive(Debug)]
pub struct Data {
    pub key: String,
    pub timestamp: u128,
    pub typ: EventTyp,
    pub key_code: u32,
}

pub enum WriteOn {
    Count(usize),
    Filter(Box<dyn (Fn(&Data) -> bool) + Send>),
}

impl std::fmt::Debug for WriteOn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Count(arg0) => f.debug_tuple("Count").field(arg0).finish(),
            Self::Filter(_) => f.debug_tuple("Filter").finish(),
        }
    }
}
