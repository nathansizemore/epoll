# epoll [<img src="https://travis-ci.org/nathansizemore/epoll.png?branch=master">](https://travis-ci.org/nathansizemore/epoll)

Rust wrapper for Linux kernel's [epoll](http://man7.org/linux/man-pages/man7/epoll.7.html)

[Documentation](http://nathansizemore.github.io/epoll/epoll/index.html)


#### Usage

~~~rust
extern crate epoll;

use epoll;
use epoll::util::*;
use epoll::EpollEvent;

fn start_event_loop() {
    // Create an epoll instance
    let epfd = epoll::create1(0).unwrap();

    // Add fd to epoll watch list
    let some_fd = 0 as RawFd;
    let mut event = EpollEvent {
        data: some_fd as u64,
        events: (event_type::EPOLLIN | event_type::EPOLLET | event_type::EPOLLRDHUP)
    };
    match epoll::ctl(epfd, ctl_op::ADD, some_fd, &mut event) {
        Ok(()) => println!("Fd added sucessfully"),
        Err(e) => println!("Epoll CtlError during add: {}", e)
    };

    // Epoll wait
    let mut events = Vec::<EpollEvent>::with_capacity(100);
    unsafe { events.set_len(100); }
    match epoll::wait(epfd, &mut events[..], -1) {
        Ok(num_events) => {
            println!("{} epoll event(s) received", num_events);
            for x in 0..num_events {
                // Do all the stuff
            }
        }
        Err(e) => println!("Error on epoll::wait(): {}", e)
    }
}
~~~
