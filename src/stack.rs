pub struct Stack {
    vec: Vec<usize>,
}

impl Stack {
    pub fn new() -> Stack {
        Stack { vec: Vec::new() }
    }

    pub fn push(&mut self, val: usize) {
        self.vec.push(val);
    }

    pub fn pop(&mut self) -> usize {
        if let Some(val) = self.vec.pop() {
            val
        } else {
            println!("WARNING: Attempted to pop() from empty stack");
            0
        }
    }
}
