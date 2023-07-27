# Rask - A highly performant HTTP server

This is very WIP, and a conversion of [rask-old](https://github.com/ryanseipp/rask-old) to utilize io_uring on linux rather than epoll.

As such, this is still a learning project and not currently recommended for any production use.

## TODO
- [x] Build bindgen to liburing
- [x] Replicate non-generated inline static functions of liburing
- [ ] Build idiomatic Rust wrapper to liburing
- [ ] Build connection layer and core event loop of server
