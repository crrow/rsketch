use std::net::TcpListener;
use std::os::fd::AsRawFd;
use epoll::{Event, Events};
use epoll::ControlOptions::EPOLL_CTL_ADD;

fn main() {
    let mut listener = TcpListener::bind(":38888").unwrap();
    listener.set_nonblocking(true).unwrap();
    let epoll = epoll::create(false).unwrap(); // 👈
    let event = Event::new(Events::EPOLLIN, listener.as_raw_fd() as _);
    epoll::ctl(epoll, EPOLL_CTL_ADD, listener.as_raw_fd(), event).unwrap();
}
