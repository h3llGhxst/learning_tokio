# tokio-notes

Learning tokio by working through the official tutorial and building the
internals myself. Each folder is one chapter: runnable code plus a short
NOTES.md covering what clicked, what broke, and why.

Status: in progress (currently on: Hello Tokio)

## Async/await
- Calling an `async fn` doesn't run it. It returns a `Future`, a value
  representing the operation that hasn't happened yet (futures are lazy).
- `.await` on that Future actually drives it to completion and yields its result.
- A future must be polled to run; `.await` and spawning are ways to make that happen.
- `#[tokio::main]` transforms `async fn main()` into a synchronous entry point
  that creates a Tokio runtime and runs the async code.


## Accepting sockets
- `TcpListener::bind(addr).await` claims the port.
- Loop on `listener.accept().await`, which returns `(TcpStream, SocketAddr)`;
  the addr is the connecting client's IP and port.
- `process(socket, db)` reads frames and handles `GET` and `SET` commands
  using the shared database.
- Flaw: awaiting `process` inline serves one client at a time. Spawning fixes this.
- Spawn a task for each client using `tokio::spawn()` so other clients can
  progress while one waits for I/O. Spawning does not make blocking code nonblocking.


## Spawn return value
- `tokio::spawn` returns a `JoinHandle`; the task starts running immediately.
- `handle.await` gives `Result<T, JoinError>`: `Err` if the task panicked
  or was cancelled.
- Use it when you need the task's result; fire-and-forget if you don't.


## Multiple tasks, one value
- A value can be moved only once. A second `move` = E0382 (use of moved value).
- Same error in a loop: "value moved in previous iteration".
- Shared data: `Arc` + clone per task. Single-owner data (sockets): move.
- Alternative to sharing: send messages over a channel.

## Send bound
- The future passed to `tokio::spawn` must implement `Send`.
- This allows the Tokio runtime to move tasks between threads.


## Per-connection state (the bug)
- `db` created inside `process` = one HashMap per connection, so clients
  can't see each other's data.
- Verified with `mini-redis-cli set` then `get` -> (nil).
- A panic in `process` kills only that connection's task.
- Fix: shared `Arc<Mutex<HashMap>>`, clone the Arc per task.


## type Db = Arc<Mutex<HashMap<String, Bytes>>>
- `type` = alias only, no new type.
- HashMap = the data; Mutex = one task or thread accessing the protected
  data at a time (including readers); Arc = many owners.
- Clone the Arc per task (cheap), all point to the same map.
- `lock().unwrap()` gets exclusive access; guard drops = lock released.

## MutexGuard
- `mutex.lock().unwrap()` returns a `MutexGuard`, giving exclusive
  access to the protected data.
- Dropping the guard releases the lock.
- `std::sync::MutexGuard` does not implement `Send`, so it cannot
  be transferred between threads.
- Holding it across `.await` makes the future non-`Send`, which means
  that future cannot be used with `tokio::spawn`.
- Use an inner scope to drop the guard before awaiting:

```rust
use std::sync::{Mutex, MutexGuard};

async fn increment_and_do_stuff(mutex: &Mutex<i32>) {
    {
        let mut lock: MutexGuard<'_, i32> = mutex.lock().unwrap();
        *lock += 1;
    } // The guard drops here, releasing the lock.

    do_something_async().await;
}
```

Here, `do_something_async()` stands for another async function in the program.
In `main.rs`, each guard drops at the end of its `Set` or `Get` block,
before `connection.write_frame(&response).await`.
