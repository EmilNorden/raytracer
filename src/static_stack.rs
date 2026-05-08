
pub struct StaticStack<T, const N: usize> {
    items: [T; N],
    size: usize,
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

    pub fn peek(&self) -> T {
        debug_assert!(self.size > 0);
        self.items[self.size - 1]
    }

    pub fn peek_at(&self, nth: usize) -> Option<T> {
        if self.size > nth {
            Some(self.items[self.size - 1 - nth])
        }
        else {
            None
        }

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
    pub fn has_items(&self) -> bool {
        self.size > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_peek_at() {
        let mut stack: StaticStack<i32, 4> = StaticStack::new_with_default(0);
        stack.push(1);
        stack.push(2);
        stack.push(3);

        assert_eq!(stack.peek_at(0), Some(3));
        assert_eq!(stack.peek_at(1), Some(2));
        assert_eq!(stack.peek_at(2), Some(1));
    }
}