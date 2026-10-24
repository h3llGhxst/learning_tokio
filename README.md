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
