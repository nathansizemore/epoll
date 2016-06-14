# epoll [<img src="https://travis-ci.org/nathansizemore/epoll.svg?branch=master">][travis-badge]

Safe wrapper around the Linux kernel's [epoll][epoll-man-page] API.

[Documentation][docs]

---

### Example Usage

``` rust
extern crate epoll;

use epoll::*;


fn main() {
	// Magic area of new fds to add
	let regsiter_buf: Vec<Interest>;

	// Magic area of Interests needing updated event flags and/or data
	let modify_buf: Vec<Interest>;

	// Magic area of Interests needing removed.
	let remove_buf: Vec<Interest>;

	// Out super awesome epoll instance.
	let mut epoll = EpollInstance::new().unwrap();

	loop {
		event_loop();
	}
}

fn event_loop(epoll: &mut EpollInstance,
	          register: &mut Vec<Interest>,
			  modify: &mut Vec<Interest>,
			  remove: &mut Vec<Interest>)
{
	// Insert new
	for ref mut interest in regsister.iter_mut() {
		epoll.add_interest(interest).unwrap();
	}

	// Modify existing
	for ref mut interest in modify.iter_mut() {
		epoll.mod_interest(interest).unwrap();
	}

	// Remove stale
	for ref mut interest in remove.iter_mut() {
		epoll.del_interest(interest).unwrap();
	}

	// Wait for new events
	let indefinite: i32 = -1;
	let max_returned: usize = 100;
	let mut event_buf = epoll.wait(indefinite, max_returned).unwrap();

	// Process stuff
	for ref mut interest in event_buf {
		handle_interest(interest);
	}
}

fn handle_interest(interest: &mut Interest) {
	// Awesome stuff...
}
```

### Author

Nathan Sizemore, nathanrsizemore@gmail.com

### License

hydrogen is available under the MPL-2.0 license. See the LICENSE file for more info.



[travis-badge]: https://travis-ci.org/nathansizemore/epoll
[docs]: https://nathansizemore.github.io/epoll/epoll/index.html
[epoll-man-page]: http://man7.org/linux/man-pages/man7/epoll.7.html
