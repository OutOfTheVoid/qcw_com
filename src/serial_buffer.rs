pub struct SerialBuffer<const N: usize> {
    data: [u8; N],
    count: usize,
    i_write: usize,
}

impl<const N: usize> SerialBuffer<N> {
    pub fn new() -> Self {
        Self {
            data: [0u8; N],
            count: 0,
            i_write: 0,
        }
    }

    pub fn push(&mut self, b: u8) {
        self.data[self.i_write] = b;
        self.i_write += 1;
        self.i_write %= N;
        self.count += 1;
    }

    pub fn count(&self) -> usize {
        self.count
    }

    pub fn free_space(&self) -> usize {
        N - self.count
    }

    pub fn pop(&mut self) -> Option<u8> {
        if self.count != 0 {
            let index = if self.i_write < self.count {
                self.i_write + N - self.count
            } else {
                self.i_write - self.count
            };
            self.count -= 1;
            Some(self.data[index])
        } else {
            None
        }
    }

    pub fn peek(&mut self) -> Option<u8> {
        if self.count != 0 {
            let index = if self.i_write < self.count {
                self.i_write + N - self.count
            } else {
                self.i_write - self.count
            };
            Some(self.data[index])
        } else {
            None
        }
    }
}
