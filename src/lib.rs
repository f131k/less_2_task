pub struct Stack<T> {
    pub stack: Vec<T>,
}

impl<T> Stack<T> {
    pub fn new() -> Self {
        Stack { stack: Vec::new() }
    }

    pub fn pop(&mut self) -> Option<T> {
        self.stack.pop()
    }

    pub fn push(&mut self, item: T) {
        self.stack.push(item)
    }

    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }

    pub fn peek(&self) -> Option<&T> {
        self.stack.last()
    }

    pub fn size(&self) -> usize {
        self.stack.len()
    }
}

pub struct Queue<T> {
    pub queue: Vec<T>,
}

impl<T> Queue<T> {
    pub fn new() -> Self {
        Queue { queue: Vec::new() }
    }

    pub fn enqueue(&mut self, item: T) {
        self.queue.push(item)
    }

    pub fn dequeue(&mut self) -> T {
        self.queue.remove(0)
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
}
