# tokio-notes

Learning tokio by working through the official tutorial and building the
internals myself. Each folder is one chapter: runnable code plus a short
NOTES.md covering what clicked, what broke, and why.

Status: in progress (currently on: Hello Tokio)

## Async/await
- Calling an `async fn` doesn't run it. It returns a `Future`, a value
  representing the operation that hasn't happened yet (futures are lazy).
- `.await` on that Future actually drives it to completion and yields its result.
- No `.await` = the code never runs.
- `#tokio::main` is a macro that transforms async fn main() to synchronous main.


## Accepting sockets
- `TcpListener::bind(addr).await` claims the port.
- Loop on `listener.accept().await`, which returns `(TcpStream, SocketAddr)`;
  the addr is the connecting client's IP and port.
- `process(socket)` reads a frame; if there is one, it logs it and replies
  with an `"unimplemented"` error (no commands exist yet).
- Flaw: awaiting `process` inline serves one client at a time. Spawning fixes this.
- to avoid blocking the event loop , spawn a task for each client. using `tokio::spawn()`


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
- Task spawned by `tokio::spanw` must implement `Send`.
- This allows the tokio runtime to move tasks between threads.


## Per-connection state (the bug)
- `db` created inside `process` = one HashMap per connection, so clients
  can't see each other's data.
- Verified with `mini-redis-cli set` then `get` -> (nil).
- A panic in `process` kills only that connection's task.
- Fix: shared `Arc<Mutex<HashMap>>`, clone the Arc per task.


## type Db = Arc<Mutex<HashMap<String, Bytes>>>
- `type` = alias only, no new type.
- HashMap = the data; Mutex = one writer at a time; Arc = many owners.
- Clone the Arc per task (cheap), all point to the same map.
- `lock().unwrap()` gets exclusive access; guard drops = lock released.

## MutexGuard 
- `std::sync::MutexGuard` is not type `Send` , this means you cannot
- a  mutex lock to another thread , you get an error because tokio
- tokio runtime can move tasks between threads at every `.await` 
    ``` 
    sol example
        
        // This works!
        async fn increment_and_do_stuff(mutex: &Mutex<i32>) {
            {
                let mut lock: MutexGuard<i32> = mutex.lock().unwrap();
                *lock += 1;
            } // lock goes out of scope here

            do_something_async().await;
        }


    ```
