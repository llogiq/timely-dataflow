//! Handles to an operator's input and output streams.

use std::rc::Rc;
use std::cell::RefCell;
use progress::Timestamp;
use progress::count_map::CountMap;
use dataflow::channels::pullers::Counter as PullCounter;
use dataflow::channels::pushers::Counter as PushCounter;
use dataflow::channels::pushers::buffer::{Buffer, Session};
use dataflow::channels::Content;
use timely_communication::Push;

use dataflow::operators::Capability;
use dataflow::operators::capability::mint as mint_capability;

/// Handle to an operator's input stream.
pub struct InputHandle<'a, T: Timestamp, D: 'a> {
    pull_counter: &'a mut PullCounter<T, D>,
    internal: Rc<RefCell<CountMap<T>>>,
}

impl<'a, T: Timestamp, D> InputHandle<'a, T, D> {
    /// Reads the next input buffer (at some timestamp `t`) and a corresponding capability for `t`.
    /// The timestamp `t` of the input buffer can be retrieved by invoking `.time()` on the capability.
    /// Returns `None` when there's no more data available.
    #[inline]
    pub fn next(&mut self) -> Option<(Capability<T>, &mut Content<D>)> {
        let internal = &mut self.internal;
        self.pull_counter.next().map(|(&time, content)| {
            (mint_capability(time, internal.clone()), content)
        })
    }

    /// Repeatedly calls `logic` till exhaustion of the available input data.
    /// `logic` receives a capability and an input buffer.
    ///
    /// #Examples
    /// ```
    /// use timely::dataflow::operators::{ToStream, Unary};
    /// use timely::dataflow::channels::pact::Pipeline;
    ///
    /// timely::example(|scope| {
    ///     (0..10).to_stream(scope)
    ///            .unary_stream(Pipeline, "example", |input, output| {
    ///                input.for_each(|cap, data| {
    ///                    output.session(&cap).give_content(data);
    ///                });
    ///            });
    /// });
    /// ```
    #[inline]
    pub fn for_each<F: FnMut(Capability<T>, &mut Content<D>)>(&mut self, mut logic: F) {
        while let Some((cap, data)) = self.next() {
            ::logging::log(&::logging::GUARDED_MESSAGE, true);
            logic(cap, data);
            ::logging::log(&::logging::GUARDED_MESSAGE, false);
        }
    }
}

/// Constructs an input handle.
/// Declared separately so that it can be kept private when InputHandle is re-exported.
pub fn new_input_handle<'a, T: Timestamp, D: 'a>(pull_counter: &'a mut PullCounter<T, D>, internal: Rc<RefCell<CountMap<T>>>) -> InputHandle<'a, T, D> {
    InputHandle {
        pull_counter: pull_counter,
        internal: internal,
    }
}

/// Handle to an operator's output stream.
pub struct OutputHandle<'a, T: Timestamp, D: 'a, P: Push<(T, Content<D>)>+'a> {
    push_buffer: &'a mut Buffer<T, D, PushCounter<T, D, P>>,
}

impl<'a, T: Timestamp, D, P: Push<(T, Content<D>)>> OutputHandle<'a, T, D, P> {
    /// Obtains a session that can send data at the timestamp associated with capability `cap`.
    ///
    /// In order to send data at a future timestamp, obtain a capability for the new timestamp
    /// first, as show in the example.
    ///
    /// #Examples
    /// ```
    /// use timely::dataflow::operators::{ToStream, Unary};
    /// use timely::dataflow::channels::pact::Pipeline;
    ///
    /// timely::example(|scope| {
    ///     (0..10).to_stream(scope)
    ///            .unary_stream(Pipeline, "example", |input, output| {
    ///                while let Some((cap, data)) = input.next() {
    ///                    let mut time = cap.time();
    ///                    time.inner += 1;
    ///                    output.session(&cap.delayed(&time)).give_content(data);
    ///                }
    ///            });
    /// });
    /// ```
    pub fn session<'b>(&'b mut self, cap: &Capability<T>) -> Session<'b, T, D, PushCounter<T, D, P>> where 'a: 'b {
        self.push_buffer.session(cap)
    }
}

/// Constructs an output handle.
/// Declared separately so that it can be kept private when OutputHandle is re-exported.
pub fn new_output_handle<'a, T: Timestamp, D, P: Push<(T, Content<D>)>>(push_buffer: &'a mut Buffer<T, D, PushCounter<T, D, P>>) -> OutputHandle<'a, T, D, P> {
    OutputHandle {
        push_buffer: push_buffer,
    }
}


