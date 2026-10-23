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




