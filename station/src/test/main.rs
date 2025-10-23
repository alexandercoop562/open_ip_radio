mod tests {
    #[test]
    fn test_get_lock() {
        use crate::{Arc, Mutex, Queue, get_lock};

        let queue = Arc::new(Mutex::new(Queue::new()));
        let result = get_lock(&queue);
        assert!(result.is_ok());
    }

    #[test]
    fn test_log_error() {
        use crate::{LogError, MainErr};

        // Test Ok passes through
        let result: Result<i32, MainErr> = Ok(42);
        let logged = result.log_error("test");
        assert!(logged.is_ok());
        if let Ok(val) = logged {
            assert_eq!(val, 42);
        }

        // Test Err logs and returns wrapped error
        let result: Result<i32, MainErr> = Err("test error".into());
        let logged = result.log_error("test_function");
        assert!(logged.is_err());
        if let Err(e) = logged {
            assert_eq!(e.to_string(), "An error was logged.");
        }
    }
}
