use std::time::Instant;

#[derive(Debug, Default)]
pub struct PingPong {
    counter: u32,
    active: Option<Instant>,
}

impl PingPong {
    #[inline]
    pub fn new() -> Self {
        Self {
            counter: 0,
            active: Some(Instant::now()),
        }
    }

    #[inline]
    pub const fn has_active(&self) -> bool {
        self.active.is_some()
    }

    #[inline]
    pub fn next(&mut self) -> Result<u32, u32> {
        if self.has_active() {
            return Err(self.counter);
        }

        self.counter += 1;
        self.active = Some(Instant::now());
        Ok(self.counter)
    }

    #[inline]
    pub fn finish(&mut self, c: u32) -> Result<u128, Option<u32>> {
        if let Some(instant) = self.active {
            if self.counter == c {
                let elapsed = instant.elapsed().as_millis();

                self.active = None;

                return Ok(elapsed);
            }

            return Err(Some(self.counter));
        }

        Err(None)
    }

    #[inline]
    pub fn close(&mut self) -> Result<(), (u32, u128)> {
        if let Some(instant) = self.active.take() {
            return Err((self.counter, instant.elapsed().as_millis()));
        }

        Ok(())
    }
}
