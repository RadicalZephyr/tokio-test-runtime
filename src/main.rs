use futures::future;

use tokio_current_thread::{self as current_thread, CurrentThread, Entered};
use tokio_reactor::Reactor;
use tokio_timer::clock::{self, Clock};
use tokio_timer::{timer, Timer};

pub struct Rt {
    clock: Clock,
    reactor_handle: tokio_reactor::Handle,
    timer_handle: timer::Handle,
    executor: CurrentThread<Timer<Reactor>>,
}

impl Rt {
    pub fn new() -> Rt {
        let clock = Clock::new();
        let reactor = Reactor::new().expect("creating reactor failed");
        let reactor_handle = reactor.handle();
        let timer = Timer::new_with_now(reactor, clock.clone());
        let timer_handle = timer.handle();
        let executor = CurrentThread::new_with_park(timer);

        Rt {
            clock,
            reactor_handle,
            timer_handle,
            executor,
        }
    }

    pub fn run<F>(&mut self, f: F)
    where
        F: FnOnce(&mut Entered<'_, Timer<Reactor>>),
    {
        let Rt {
            ref clock,
            ref reactor_handle,
            ref timer_handle,
            ref mut executor,
            ..
        } = *self;

        let mut enter = tokio_executor::enter().expect("Multiple executors at once");

        tokio_reactor::with_default(&reactor_handle, &mut enter, |enter| {
            clock::with_default(clock, enter, |enter| {
                timer::with_default(&timer_handle, enter, |enter| {
                    let mut default_executor = current_thread::TaskExecutor::current();
                    tokio_executor::with_default(&mut default_executor, enter, |enter| {
                        let mut entered = executor.enter(enter);
                        f(&mut entered);
                        entered.run().expect("exited main executor");
                    })
                })
            })
        });
    }
}

fn spawn_with_default() {
    let res: Result<(), ()> = Ok(());
    current_thread::spawn(future::result(res));
}

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_it() {
        spawn_with_default();
    }

    #[test]
    fn test_it_with_rt() {
        let mut rt = Rt::new();
        rt.run(|_| spawn_with_default());
    }
}
