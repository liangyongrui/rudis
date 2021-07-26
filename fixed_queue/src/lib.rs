//! fixed queue

/// 一个简单的，固定大小的 循环队列
/// 队列满的时候，新元素自动覆盖旧元素
pub struct FixedQueue<E> {
    data: Vec<Option<E>>,
    /// 第一个元素的下标
    head: usize,
    /// 最后一个元素的下标
    tail: usize,
    len: usize,
    capacity: usize,
}

impl<E> FixedQueue<E> {
    pub fn new(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
            head: 0,
            tail: 0,
            len: 0,
            capacity,
        }
    }

    /// return 被挤出去的值
    pub fn push(&mut self, e: E) -> Option<E> {
        // vec 还没用完
        if self.data.len() < self.capacity {
            self.data.push(Some(e));
            self.len += 1;
            self.tail += 1;
            return None;
        }
        self.tail = (self.tail + 1) % self.capacity;
        if self.tail == self.head {
            // 队列已满，把头向后移动
            self.head = (self.head + 1) % self.capacity;
        } else {
            // 队列未满，长度+1
            self.len += 1;
        }
        self.data[self.tail].replace(e)
    }

    pub fn pop(&mut self) -> Option<E> {
        // 对列为空
        if self.head == self.tail {
            return None;
        }
        let res = self.data[self.head].take();
        self.head = (self.head + 1) % self.capacity;
        self.len -= 1;
        res
    }
}
