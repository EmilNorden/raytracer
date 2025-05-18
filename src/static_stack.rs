
pub struct StaticStack<T, const N: usize> {
    items: [T; N],
    size: usize,
}

impl<T, const N: usize> StaticStack<T, N>
where T: Default + Copy
{
    pub fn new() -> Self {
        Self {
            items: [T::default(); N],
            size: 0,
        }
    }
}

impl<T, const N: usize> StaticStack<T, N>
where T: Copy
{
    pub fn new_with_default(default: T) -> Self {
        Self {
            items: [default; N],
            size: 1,
        }
    }
    
    pub fn pop(&mut self) -> T {
        debug_assert!(self.size > 0);
        self.size -= 1;
        self.items[self.size]
    }
}

impl<T, const N: usize> StaticStack<T, N> {
    pub fn push(&mut self, item: T) {
        debug_assert!(self.size < N);
        self.items[self.size] = item;
        self.size += 1;
    }
    
    pub fn is_empty(&self) -> bool {
        self.size == 0
    }
}