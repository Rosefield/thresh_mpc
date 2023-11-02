pub struct MultiBuf {
    bufs: Box<[Box<[u8]>]>,
    pos: usize,
}

impl MultiBuf {
    pub fn new(bufs: Box<[Box<[u8]>]>) -> Self {
        MultiBuf { bufs, pos: 0 }
    }

    pub fn total_size(&self) -> usize {
        self.bufs.iter().map(|b| b.len()).sum()
    }

    pub fn next_size(&self) -> Option<usize> {
        self.bufs.get(self.pos).map(|b| b.len())
    }

    pub fn iter(&self) -> impl Iterator<Item = &[u8]> {
        self.bufs.iter().map(|b| b.as_ref())
    }

    pub fn next_buf_mut(&mut self) -> Option<&mut [u8]> {
        let buf = self.bufs.get_mut(self.pos).map(|b| b.as_mut());
        self.pos += 1;
        buf
    }

    pub fn reset_pos(&mut self) {
        self.pos = 0;
    }
}
