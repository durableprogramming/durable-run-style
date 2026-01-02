use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use tokio::sync::Notify;

/// An asynchronous job queue for managing command execution tasks.
///
/// This queue provides thread-safe enqueue and dequeue operations,
/// with async notification when items become available.
///
/// # Type Parameters
///
/// * `T` - The type of items stored in the queue. Must be `Send + Sync + 'static`.
#[derive(Clone)]
pub struct CommandQueue<T> {
    queue: Arc<Mutex<VecDeque<T>>>,
    notify: Arc<Notify>,
    finished: Arc<Mutex<bool>>,
}

impl<T> CommandQueue<T> {
    /// Creates a new empty command queue.
    pub fn new() -> Self {
        Self {
            queue: Arc::new(Mutex::new(VecDeque::new())),
            notify: Arc::new(Notify::new()),
            finished: Arc::new(Mutex::new(false)),
        }
    }

    /// Adds an item to the end of the queue and notifies waiting consumers.
    pub fn enqueue(&self, item: T) {
        let mut queue = self.queue.lock().unwrap();
        queue.push_back(item);
        drop(queue);
        self.notify.notify_one();
    }

    /// Asynchronously removes and returns the next item from the queue.
    ///
    /// If the queue is empty and not finished, this method will wait until an item is available.
    /// If the queue is empty and finished, this method returns None.
    pub async fn dequeue(&self) -> Option<T> {
        loop {
            let item = {
                let mut queue = self.queue.lock().unwrap();
                queue.pop_front()
            };

            if let Some(item) = item {
                return Some(item);
            }

            let finished = *self.finished.lock().unwrap();
            if finished {
                return None;
            }

            self.notify.notified().await;
        }
    }

    /// Returns the number of commands currently in the queue.
    pub fn len(&self) -> usize {
        self.queue.lock().unwrap().len()
    }

    /// Returns true if the queue is empty.
    pub fn is_empty(&self) -> bool {
        self.queue.lock().unwrap().is_empty()
    }

    /// Returns a clone of the front of the queue without removing it.
    ///
    /// Returns `None` if the queue is empty.
    /// This method requires `T: Clone`.
    pub fn peek(&self) -> Option<T>
    where
        T: Clone,
    {
        let queue = self.queue.lock().unwrap();
        queue.front().cloned()
    }

    /// Removes all items from the queue.
    pub fn clear(&self) {
        let mut queue = self.queue.lock().unwrap();
        queue.clear();
    }

    /// Marks the queue as finished, indicating no more items will be enqueued.
    ///
    /// After calling this method, dequeue operations will return None when the queue is empty.
    pub fn finish(&self) {
        let mut finished = self.finished.lock().unwrap();
        *finished = true;
        self.notify.notify_waiters();
    }

    /// Returns true if the queue has been marked as finished.
    pub fn is_finished(&self) -> bool {
        *self.finished.lock().unwrap()
    }



    /// Drains all items from the queue and returns them as a vector.
    pub fn drain(&self) -> Vec<T> {
        let mut queue = self.queue.lock().unwrap();
        queue.drain(..).collect()
    }

    /// Attempts to dequeue an item with a timeout.
    ///
    /// Returns `None` if no item becomes available within the timeout or if the queue is finished and empty.
    pub async fn dequeue_timeout(&self, duration: std::time::Duration) -> Option<T> {
        tokio::time::timeout(duration, self.dequeue()).await.ok().flatten()
    }
}

impl<T> Default for CommandQueue<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{timeout, Duration};

    #[test]
    fn test_command_queue_new() {
        let queue: CommandQueue<String> = CommandQueue::new();
        assert_eq!(queue.len(), 0);
        assert!(queue.is_empty());
    }

    #[test]
    fn test_command_queue_enqueue() {
        let queue: CommandQueue<String> = CommandQueue::new();
        queue.enqueue("test command".to_string());
        assert_eq!(queue.len(), 1);
        assert!(!queue.is_empty());
    }

    #[tokio::test]
    async fn test_command_queue_dequeue() {
        let queue: CommandQueue<String> = CommandQueue::new();
        queue.enqueue("test command".to_string());

        let command = timeout(Duration::from_secs(1), queue.dequeue())
            .await
            .unwrap()
            .unwrap();

        assert_eq!(command, "test command");
        assert_eq!(queue.len(), 0);
        assert!(queue.is_empty());
    }

    #[tokio::test]
    async fn test_command_queue_multiple_operations() {
        let queue: CommandQueue<String> = CommandQueue::new();

        // Enqueue multiple commands
        queue.enqueue("cmd1".to_string());
        queue.enqueue("cmd2".to_string());
        queue.enqueue("cmd3".to_string());

        assert_eq!(queue.len(), 3);

        // Dequeue in order
        let cmd1 = timeout(Duration::from_secs(1), queue.dequeue()).await.unwrap().unwrap();
        assert_eq!(cmd1, "cmd1");
        assert_eq!(queue.len(), 2);

        let cmd2 = timeout(Duration::from_secs(1), queue.dequeue()).await.unwrap().unwrap();
        assert_eq!(cmd2, "cmd2");
        assert_eq!(queue.len(), 1);

        let cmd3 = timeout(Duration::from_secs(1), queue.dequeue()).await.unwrap().unwrap();
        assert_eq!(cmd3, "cmd3");
        assert_eq!(queue.len(), 0);
    }

    #[test]
    fn test_command_queue_peek() {
        let queue: CommandQueue<String> = CommandQueue::new();
        queue.enqueue("test".to_string());
        assert_eq!(queue.peek(), Some("test".to_string()));
        assert_eq!(queue.len(), 1); // Peek doesn't remove
    }

    #[test]
    fn test_command_queue_clear() {
        let queue: CommandQueue<String> = CommandQueue::new();
        queue.enqueue("test1".to_string());
        queue.enqueue("test2".to_string());
        assert_eq!(queue.len(), 2);
        queue.clear();
        assert_eq!(queue.len(), 0);
        assert!(queue.is_empty());
    }

    #[test]
    fn test_command_queue_drain() {
        let queue: CommandQueue<String> = CommandQueue::new();
        queue.enqueue("test1".to_string());
        queue.enqueue("test2".to_string());
        let drained = queue.drain();
        assert_eq!(drained, vec!["test1", "test2"]);
        assert_eq!(queue.len(), 0);
    }

    #[tokio::test]
    async fn test_command_queue_dequeue_timeout() {
        let queue: CommandQueue<String> = CommandQueue::new();

        // Test timeout when empty
        let result = queue.dequeue_timeout(Duration::from_millis(10)).await;
        assert_eq!(result, None);

        // Test successful dequeue
        queue.enqueue("test".to_string());
        let result = queue.dequeue_timeout(Duration::from_millis(10)).await;
        assert_eq!(result, Some("test".to_string()));
    }

    #[tokio::test]
    async fn test_command_queue_finish() {
        let queue: CommandQueue<String> = CommandQueue::new();

        // Initially not finished
        assert!(!queue.is_finished());

        // Finish the queue
        queue.finish();
        assert!(queue.is_finished());

        // Dequeue from finished empty queue should return None
        let result = queue.dequeue().await;
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_command_queue_dequeue_after_finish() {
        let queue: CommandQueue<String> = CommandQueue::new();

        // Enqueue items
        queue.enqueue("item1".to_string());
        queue.enqueue("item2".to_string());

        // Dequeue one item
        let result1 = queue.dequeue().await;
        assert_eq!(result1, Some("item1".to_string()));

        // Finish the queue
        queue.finish();

        // Dequeue remaining item
        let result2 = queue.dequeue().await;
        assert_eq!(result2, Some("item2".to_string()));

        // Dequeue from empty finished queue
        let result3 = queue.dequeue().await;
        assert_eq!(result3, None);
    }

    #[test]
    fn test_command_queue_default() {
        let queue: CommandQueue<String> = Default::default();
        assert_eq!(queue.len(), 0);
        assert!(queue.is_empty());
    }
}