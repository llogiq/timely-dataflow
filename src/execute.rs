//! Starts a timely dataflow execution from configuration information and per-worker logic.

use timely_communication::{initialize, Configuration, Allocator, WorkerGuards};
use dataflow::scopes::{Root, Child, Scope};

/// Executes a single-threaded timely dataflow computation.
///
/// The `example` method takes a closure on a `Scope` which it executes to initialize and run a
/// timely dataflow computation on a single thread. This method is intended for use in examples,
/// rather than programs that may need to run across multiple workers.
///
/// #Examples
/// ```
/// use timely::dataflow::operators::{ToStream, Inspect};
///
/// timely::example(|scope| {
///     (0..10).to_stream(scope)
///            .inspect(|x| println!("seen: {:?}", x));
/// });
/// ```
pub fn example<F>(func: F) 
where F: Fn(&mut Child<Root<Allocator>, u64>)+Send+Sync+'static {
    initialize(Configuration::Thread, move |allocator| {
        let mut root = Root::new(allocator);
        root.scoped::<u64,_,_>(|x| func(x));
        while root.step() { }
    }).unwrap();
}

/// Executes a timely dataflow from a configuration and per-communicator logic.
///
/// The `execute` method takes a `Configuration` and spins up some number of
/// workers threads, each of which execute the supplied closure to construct
/// and run a timely dataflow computation.
///
/// The closure may return a `T: Send+'static`, and `execute` returns a result
/// containing a `WorkerGuards<T>` (or error information), which can be joined
/// to recover the result `T` values from the local workers.
///
/// #Examples
/// ```
/// use timely::dataflow::Scope;
/// use timely::dataflow::operators::{ToStream, Inspect};
///
/// // execute a timely dataflow using three worker threads.
/// timely::execute(timely::Configuration::Process(3), |root| {
///     root.scoped::<u64,_,_>(|scope| {
///         (0..10).to_stream(scope)
///                .inspect(|x| println!("seen: {:?}", x));
///     })
/// }).unwrap();
/// ```
pub fn execute<T:Send+'static, F>(config: Configuration, func: F) -> Result<WorkerGuards<T>,String> 
where F: Fn(&mut Root<Allocator>)->T+Send+Sync+'static {
    initialize(config, move |allocator| {
        let mut root = Root::new(allocator);
        let result = func(&mut root);
        while root.step() { }
        result
    })
}


/// Executes a timely dataflow from supplied arguments and per-communicator logic.
///
/// The `execute` method takes arguments (typically `std::env::args()`) and spins up some number of
/// workers threads, each of which execute the supplied closure to construct and run a timely
/// dataflow computation.
///
/// The closure may return a `T: Send+'static`, and `execute` returns a result
/// containing a `WorkerGuards<T>` (or error information), which can be joined
/// to recover the result `T` values from the local workers.
///
/// The arguments `execute` currently understands are:
///
/// `-w, --workers`: number of per-process worker threads.
///
/// `-n, --processes`: number of processes involved in the computation.
///
/// `-p, --process`: identity of this process; from 0 to n-1.
///
/// `-h, --hostfile`: a text file whose lines are "hostname:port" in order of process identity.
/// If not specified, `localhost` will be used, with port numbers increasing from 2101 (chosen
/// arbitrarily).
///
/// #Examples
/// ```
/// use timely::dataflow::*;
/// use timely::dataflow::operators::{ToStream, Inspect};
///
/// // execute a timely dataflow using command line parameters
/// timely::execute_from_args(std::env::args(), |root| {
///     root.scoped::<u64,_,_>(|scope| {
///         (0..10).to_stream(scope)
///                .inspect(|x| println!("seen: {:?}", x));
///     })
/// }).unwrap();
/// ```
/// ```ignore
/// host0% cargo run -- -w 2 -n 4 -h hosts.txt -p 0
/// host1% cargo run -- -w 2 -n 4 -h hosts.txt -p 1
/// host2% cargo run -- -w 2 -n 4 -h hosts.txt -p 2
/// host3% cargo run -- -w 2 -n 4 -h hosts.txt -p 3
/// ```
/// ```ignore
/// % cat hosts.txt
/// host0:port
/// host1:port
/// host2:port
/// host3:port
/// ```
pub fn execute_from_args<I, T:Send+'static, F>(iter: I, func: F) -> Result<WorkerGuards<T>,String> 
    where I: Iterator<Item=String>, 
          F: Fn(&mut Root<Allocator>)->T+Send+Sync+'static, {
    execute(try!(Configuration::from_args(iter)), func)
 }
