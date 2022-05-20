// use crate::os::trace_fionread;
use crate::runtime::*;
#[cfg(feature = "verify")]
use crate::tcb::verifier::external_specs::result::*;
#[cfg(feature = "verify")]
use crate::tcb::verifier::*;
use crate::types::*;
use crate::wrappers::wasi_clock_time_get; // TODO: remove this circular reference
use prusti_contracts::*;
use std::convert::{TryFrom, TryInto};
use wave_macros::{external_calls, external_methods, with_ghost_var};
use RuntimeError::*;
use crate::os::trace_fionread;

#[with_ghost_var(trace: &mut Trace)]
#[requires(ctx_safe(ctx))]
#[requires(trace_safe(trace, ctx))]
#[ensures(ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx))]
#[external_methods(push, checked_sub, try_into, subscription_clock_abstime)]
#[external_calls(Some)]
pub fn poll_parse_clock(
    ctx: &VmCtx,
    sub_clock: SubscriptionClock,
    precision: u64,
    min_timeout: &mut Option<Timestamp>,
    timeouts: &mut Vec<(u64, Timestamp)>,
    userdata: u64,
) -> RuntimeResult<()> {
    // if the subscription is a clock, check if it is the shortest timeout.
    // let clock = subscription_clock.id;
    match sub_clock.id.try_into()? {
        // TODO: what clock source does posix poll use for timeouts? Will a relative
        //       realtime be significantly different than monotonic?
        ClockId::Monotonic | ClockId::Realtime => {
            let now = wasi_clock_time_get(ctx, sub_clock.id, precision)?;
            let timeout: Timestamp = if sub_clock.flags.subscription_clock_abstime() {
                // if this is an absolute timeout, we need to wait the difference
                // between now and the timeout
                // This will also perform a checked cast to an i32, which will
                sub_clock.timeout.checked_sub(now).ok_or(Eoverflow)?
            } else {
                sub_clock.timeout
            };

            if let Some(m_timeout) = min_timeout {
                if timeout < *m_timeout {
                    *min_timeout = Some(timeout);
                }
            } else {
                *min_timeout = Some(timeout);
            }

            timeouts.push((userdata, timeout));
            Ok(())
        }
        // we don't support timeouts on other clock types
        _ => {
            return Err(Einval);
        }
    }
}

#[with_ghost_var(trace: &mut Trace)]
#[requires(ctx_safe(ctx))]
#[requires(trace_safe(trace, ctx))]
#[ensures(ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx))]
#[external_methods(push, to_posix)]
pub fn poll_parse_fds(
    ctx: &VmCtx,
    pollfds: &mut Vec<libc::pollfd>,
    fd_data: &mut Vec<(u64, SubscriptionFdType)>,
    userdata: u64,
    subscription_readwrite: SubscriptionFdReadWrite,
) -> RuntimeResult<()> {
    let fd = ctx.fdmap.fd_to_native(subscription_readwrite.v_fd)?;
    let os_fd: usize = fd.to_raw();
    // let event = match subscription_readwrite.typ {
    //     SubscriptionFdType::Read => libc::POLLIN,
    //     SubscriptionFdType::Write => libc::POLLOUT,
    // };
    let event = subscription_readwrite.typ.to_posix();
    // convert FD subscriptions to their libc versions
    let pollfd = libc::pollfd {
        fd: os_fd as i32,
        events: event,
        revents: 0,
    };
    pollfds.push(pollfd);
    fd_data.push((userdata, subscription_readwrite.typ));
    Ok(())
}

#[with_ghost_var(trace: &mut Trace)]
#[requires(ctx_safe(ctx))]
#[requires(trace_safe(trace, ctx))]
#[ensures(ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx))]
// #[external_calls(poll_handle_fds, poll_handle_clock)]
pub fn parse_subscriptions(
    ctx: &VmCtx,
    in_ptr: u32,
    nsubscriptions: u32,
    precision: u64,
    min_timeout: &mut Option<Timestamp>,
    timeouts: &mut Vec<(u64, Timestamp)>,
    pollfds: &mut Vec<libc::pollfd>,
    fd_data: &mut Vec<(u64, SubscriptionFdType)>,
) -> RuntimeResult<()> {
    let mut i = 0;
    while i < nsubscriptions {
        body_invariant!(ctx_safe(ctx));
        body_invariant!(trace_safe(trace, ctx));

        let sub_offset = i * Subscription::WASI_SIZE;

        if !ctx.fits_in_lin_mem_usize(
            (in_ptr + sub_offset) as usize,
            Subscription::WASI_SIZE as usize,
        ) {
            return Err(Eoverflow);
        }

        let subscription = Subscription::read(ctx, in_ptr + sub_offset)?;

        match subscription.subscription_u {
            SubscriptionInner::Clock(subscription_clock) => {
                poll_parse_clock(
                    ctx,
                    subscription_clock,
                    precision,
                    min_timeout,
                    timeouts,
                    subscription.userdata,
                )?;
            }
            SubscriptionInner::Fd(subscription_readwrite) => {
                poll_parse_fds(
                    ctx,
                    pollfds,
                    fd_data,
                    subscription.userdata,
                    subscription_readwrite,
                )?;
            }
        }

        i += 1;
    }
    Ok(())
}

#[with_ghost_var(trace: &mut Trace)]
#[requires(ctx_safe(ctx))]
#[requires(trace_safe(trace, ctx))]
#[ensures(ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx))]
pub fn writeback_timeouts(
    ctx: &mut VmCtx,
    out_ptr: u32,
    timeouts: &Vec<(u64, Timestamp)>,
    min_timeout: &Option<Timestamp>,
) -> RuntimeResult<u32> {
    let mut num_events_written = 0;
    let mut event_idx = 0;
    while event_idx < timeouts.len() {
        body_invariant!(ctx_safe(ctx));
        body_invariant!(trace_safe(trace, ctx));

        let (userdata, timeout) = timeouts[event_idx];
        let event_offset = (num_events_written * Event::WASI_SIZE) as usize;
        if !ctx.fits_in_lin_mem_usize(out_ptr as usize + event_offset, Event::WASI_SIZE as usize) {
            return Err(Eoverflow);
        }
        // Technically we know there must be a min_timeout, but use if let to be safe
        if let Some(m_timeout) = min_timeout {
            if timeout == *m_timeout {
                let event = Event {
                    userdata,
                    error: RuntimeError::Success,
                    typ: EventType::Clock,
                    fd_readwrite: None,
                };
                event.write(ctx, out_ptr + event_offset as u32);
                num_events_written += 1;
            }
        }
        event_idx += 1;
    }

    return Ok(num_events_written);
}

#[with_ghost_var(trace: &mut Trace)]
#[requires(ctx_safe(ctx))]
#[requires(trace_safe(trace, ctx))]
#[ensures(ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx))]
#[external_calls(from_posix, from_poll_revents, Some)]
#[external_methods(to_event_type)]
pub fn writeback_fds(
    ctx: &mut VmCtx,
    out_ptr: u32,
    pollfds: &Vec<libc::pollfd>,
    fd_data: &Vec<(u64, SubscriptionFdType)>,
) -> RuntimeResult<u32> {
    let mut num_events_written = 0;
    let mut event_idx = 0;
    while event_idx < fd_data.len() {
        body_invariant!(ctx_safe(ctx));
        body_invariant!(trace_safe(trace, ctx));

        let (userdata, sub_type) = fd_data[event_idx];
        // let typ = match sub_type {
        //     SubscriptionFdType::Read => EventType::FdRead,
        //     SubscriptionFdType::Write => EventType::FdWrite,
        // };
        let typ = sub_type.to_event_type();

        let pollfd = pollfds[event_idx];

        // if no event ocurred, continue
        if pollfd.revents == 0 {
            continue;
        }

        let event_offset = (num_events_written * Event::WASI_SIZE) as usize;
        if !ctx.fits_in_lin_mem_usize(out_ptr as usize + event_offset, Event::WASI_SIZE as usize) {
            return Err(Eoverflow);
        }

        // get the number of bytes for reading...
        // TODO: If we want, we can remove this, and always return 1 for reads
        let nbytes = match typ {
            EventType::FdRead => trace_fionread(ctx, HostFd::from_raw(pollfd.fd as usize))? as u64,
            // no way to check number of bytes for writing
            _ => 0,
        };

        let fd_readwrite = EventFdReadWrite {
            nbytes,
            flags: EventRwFlags::from_posix(pollfd.revents),
        };

        let error = RuntimeError::from_poll_revents(pollfd.revents);

        let event = Event {
            userdata,
            error,
            typ,
            fd_readwrite: Some(fd_readwrite),
        };
        event.write(ctx, out_ptr + event_offset as u32);
        num_events_written += 1;

        event_idx += 1;
    }
    return Ok(num_events_written);
}
