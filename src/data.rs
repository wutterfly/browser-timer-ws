use std::{
    io::{BufWriter, Write},
    path::PathBuf,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

use crate::{message::EventTyp, time::now};

static CONN_COUNTER: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug)]
pub struct Distributer {
    out_dir: PathBuf,
    write_on: Option<WriteOn>,
    out_prefix: &'static str,
}

impl Distributer {
    pub fn new(
        write_on: Option<WriteOn>,
        out_dir: PathBuf,
        out_prefix: &'static str,
    ) -> std::io::Result<Self> {
        std::fs::create_dir_all(&out_dir)?;

        Ok(Self {
            out_dir,
            write_on,
            out_prefix,
        })
    }

    pub fn new_connection(&self) -> DataHolder {
        DataHolder::new(self.write_on.clone(), self.out_dir.clone(), self.out_prefix)
    }
}

#[derive(Debug)]
pub struct DataHolder {
    buffer: Vec<Data>,
    out_dir: PathBuf,
    counter: usize,
    file_counter: usize,
    connection_counter: usize,
    write_on: Option<WriteOn>,
    out_prefix: &'static str,
    rtt: u128,
    first_timestamp: u128,
}

impl DataHolder {
    pub fn new(write_on: Option<WriteOn>, out_dir: PathBuf, out_prefix: &'static str) -> Self {
        let cc = CONN_COUNTER.fetch_add(1, Ordering::Relaxed);
        log::trace!("New Connection Data Holder: {cc}");

        let first_timestamp = now();
        Self {
            buffer: Vec::with_capacity(10_000),
            counter: 0,
            out_dir,
            file_counter: 0,
            connection_counter: cc,
            write_on,
            out_prefix,
            rtt: 0,
            first_timestamp,
        }
    }

    #[inline]
    pub const fn first_timestamp(&self) -> u128 {
        self.first_timestamp
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
        let file_name = format!(
            "{}_{}_{}.csv",
            self.out_prefix, self.connection_counter, self.file_counter
        );
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

        Ok(())
    }
}

#[derive(Debug)]
pub struct Data {
    pub key: Box<str>,
    pub timestamp: u64,
    pub typ: EventTyp,
    pub key_code: u16,
}

#[derive(Clone)]
pub enum WriteOn {
    Count(usize),
    Filter(Arc<dyn (Fn(&Data) -> bool) + Send + Sync>),
}

impl std::fmt::Debug for WriteOn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Count(arg0) => f.debug_tuple("Count").field(arg0).finish(),
            Self::Filter(_) => f.debug_tuple("Filter").finish(),
        }
    }
}
