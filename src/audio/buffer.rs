use bit_mask_ring_buf::BitMaskRB;

pub struct ProcessorBuffers<T> {
    input: BitMaskRB<T>,
    output: BitMaskRB<T>,
    in_write: isize,
    in_read: isize,
    out_write: isize,
    out_read: isize,
    init_value: T,
}

impl<T: Clone + Copy> ProcessorBuffers<T> {
    pub fn new(capacity: usize, value: T) -> Self {
        let cap = capacity.next_power_of_two();
        Self {
            input: BitMaskRB::new(cap, value),
            output: BitMaskRB::new(cap, value),
            in_write: 0,
            in_read: 0,
            out_write: 0,
            out_read: 0,
            init_value: value,
        }
    }

    #[inline(always)]
    pub fn input_size(&self) -> usize {
        (self.in_write - self.in_read) as usize
    }

    #[inline(always)]
    pub fn output_size(&self) -> usize {
        (self.out_write - self.out_read) as usize
    }

    // writes samples to input buffer by index without advancing the cursor
    #[inline(always)]
    pub fn write_input(&mut self, sample: T, index: usize) {
        self.input[self.in_write + (index as isize)] = sample;
    }

    // reads samples from input buffer by index without advancing the cursor
    #[inline(always)]
    pub fn read_input_into(&mut self, slice: &mut [T]) {
        self.input.read_into(slice, self.in_read);
        self.in_read += slice.len() as isize;
    }

    // advance the input buffer write cursor
    #[inline(always)]
    pub fn advance_input_write_cursor(&mut self, size: usize) {
        self.in_write += size as isize;
    }

    // reads samples from output buffer by index without advancing the cursor
    #[inline(always)]
    pub fn read_output(&mut self, index: usize) -> T {
        self.output[self.out_read + (index as isize)]
    }

    // writes samples to output buffer by index without advancing the cursor
    #[inline(always)]
    pub fn write_output_latest(&mut self, slice: &[T]) {
        self.output.write_latest(slice, self.out_write);
        self.out_write += slice.len() as isize;
    }

    // advance the output buffer read cursor
    #[inline(always)]
    pub fn advance_output_read_cursor(&mut self, size: usize) {
        self.out_read += size as isize;
    }

    // advance the output buffer read cursor
    #[inline(always)]
    pub fn advance_output_write_cursor(&mut self, size: usize) {
        self.out_write += size as isize;
    }

    pub fn reset(&mut self) {
        let cap = self.input.len().get();
        for i in 0..cap {
            self.input[i as isize] = self.init_value;
            self.output[i as isize] = self.init_value;
        }
        // Reset all cursors to the start.
        self.in_write = 0;
        self.in_read = 0;
        self.out_write = 0;
        self.out_read = 0;
    }
}
