# epoll [<img src="https://travis-ci.org/nathansizemore/epoll.svg?branch=master">][travis-badge]

Safe wrapper around the Linux kernel's [epoll][epoll-man-page] API.

[Documentation][docs]

---


### Example Usage

``` rust
extern crate epoll;

use std::sync::{Arc, Mutex};
use epoll::*;


type InterestList = Arc<Mutex<Vec<Interest>>>;


fn event_loop(mut epoll: EpollInstance,
	      register: InterestList,
	      modify: InterestList,
	      remove: InterestList)
{
    loop {
        // Insert new interests
        {
            let mut list = register.lock().unwrap();
            for interest in list.drain(..) {
                epoll.add_interest(interest).unwrap();
            }
        }

        // Modify existing interests
        {
            let list = modify.lock().unwrap();
            for ref interest in list.iter() {
                epoll.mod_interest(interest).unwrap();
            }
        }

        // Remove existing interests
        {
            let list = remove.lock().unwrap();
            for ref interest in list.iter() {
                epoll.del_interest(interest).unwrap();
            }
        }

        // Wait for new events
        let indefinite: i32 = -1;
        let max_returned: usize = 100;
        let event_buf = epoll.wait(indefinite, max_returned).unwrap();
        handle_events(event_buf);
    }
}

fn handle_events(events: Vec<Interest>) {
    // Awesome handling of events here...
}
```

### Author

Nathan Sizemore, nathanrsizemore@gmail.com

### License

epoll is available under the MPL-2.0 license. See the LICENSE file for more info.



[travis-badge]: https://travis-ci.org/nathansizemore/epoll
[docs]: https://nathansizemore.github.io/epoll/epoll/index.html
[epoll-man-page]: http://man7.org/linux/man-pages/man7/epoll.7.html
