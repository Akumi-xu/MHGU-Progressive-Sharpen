#![no_std]
#![allow(unsafe_op_in_unsafe_fn, static_mut_refs)]
mod session;
use core::{
    mem::transmute,
    ptr::{read_volatile as rd, write_volatile as wr},
};
use forge::log::prelude::*;
use session::{Session, restore_amount};

static mut BASE: u32 = 0;
static mut ACTOR: u32 = 0;
static mut ROUND: Option<Session> = None;
static mut ITEM: u32 = 0;
static mut IN_NATIVE: bool = false;
static mut COST: u32 = 0;
static mut ROW: [u32; 6] = [0; 6];
static mut CHILDREN: [u32; 2] = [0; 2];
unsafe fn r(p: u32, o: u32) -> u32 {
    rd((p + o) as *const u32)
}
unsafe fn h(p: u32, o: u32) -> u32 {
    rd((p + o) as *const u16) as u32
}
unsafe fn b(p: u32, o: u32) -> u32 {
    rd((p + o) as *const u8) as u32
}
unsafe fn f(p: u32, o: u32) -> f32 {
    rd((p + o) as *const f32)
}
unsafe fn call(off: u32, p: u32) -> u32 {
    let fun: unsafe extern "C" fn(u32) -> u32 = transmute(BASE + off);
    fun(p)
}
unsafe fn skill(p: u32, id: u32) -> bool {
    let fun: unsafe extern "C" fn(u32, u32) -> u32 = transmute(BASE + 0x2864c4);
    fun(p, id) != 0
}
unsafe fn allowed() -> bool {
    let slot = r(BASE, 0x1838d28);
    if slot == 0 {
        return false;
    }
    let fun: unsafe extern "C" fn(u32, u32, u32) -> u32 = transmute(BASE + 0x3a5ed0);
    fun(r(slot, 0), 5, 1) == 0
}
#[forge::hook(offset = 0x2ad288)]
fn sharpness(p: u32, delta: i32, flags: u32) {
    unsafe {
        let remaining = if IN_NATIVE && p == ACTOR && delta > 0 {
            ROUND
                .as_ref()
                .map_or(delta, |s| delta.saturating_sub(s.issued as i32))
        } else {
            delta
        };
        if remaining != 0 || delta == 0 {
            original!(p, remaining, flags);
        }
    }
}
#[forge::hook(offset = 0x3015e0)]
fn consume(p: u32, item: u32, amount: i32, flags: u32, extra: u32) -> u32 {
    unsafe {
        if IN_NATIVE
            && p == ACTOR
            && item == ITEM
            && amount == -1
            && ROUND.as_ref().is_some_and(|s| s.consumed)
        {
            return COST;
        }
    }
    original!(p, item, amount, flags, extra)
}

// Keep the borrowed command row alive across the native action handoff.
unsafe fn dodge(p: u32) -> bool {
    let pad = r(BASE, 0x2126888);
    if pad == 0 {
        return false;
    }
    let index = r(pad, 0x6b60);
    if index > 8 || r(pad + index * 0xb50, 0x1d0) & 0x4000 == 0 {
        return false;
    }
    let res = call(0x285f1c, p);
    if res == 0 || r(res, 0x68) == 0 {
        return false;
    }
    let rows = r(res, 0x74);
    if rows == 0 {
        return false;
    }
    let source = r(rows, 0);
    if source == 0 {
        return false;
    }
    let n = r(source, 8);
    let items = r(source, 20);
    if n > 48 || items == 0 {
        return false;
    }
    let mut count = 0;
    for i in 0..n {
        let child = r(items, i * 4);
        if child == 0 {
            continue;
        }
        let prop = r(child, 8);
        if prop == 0 {
            continue;
        }
        if h(prop, 0x64) == 1 && [6, 166].contains(&h(prop, 0x66)) && h(prop, 0x6a) == 0x8007 {
            if count == 2 {
                return false;
            }
            CHILDREN[count] = child;
            count += 1;
        }
    }
    if count == 0 {
        warn!("[psharp] no native dodge command");
        return false;
    }
    for i in 0..6 {
        ROW[i] = r(source, i as u32 * 4);
    }
    ROW[2] = count as u32;
    ROW[3] = count as u32;
    ROW[5] = CHILDREN.as_ptr() as u32;
    let mut saved = [0u32; 17];
    for i in 0..17 {
        saved[i] = r(p, 0x24a8 + i as u32 * 4);
    }
    let state = h(p, 0x2506);
    let action = h(p, 0x250a);
    wr((p + 0x24a8) as *mut u32, ROW.as_ptr() as u32);
    for o in (0x24b0..=0x24e8).step_by(4) {
        wr((p + o) as *mut u32, 0);
    }
    // Item actions suppress processed button flags. Forward this physical B press
    // only to the borrowed native dodge commands, then restore the input words.
    let mut input = [0u32; 4];
    for i in 0..4 {
        input[i] = r(p, 0x1414 + i as u32 * 4);
    }
    for i in 0..count {
        let key = r(r(CHILDREN[i], 8), 0x6c);
        if key < 128 {
            let at = p + 0x1414 + (key / 32) * 4;
            wr(at as *mut u32, r(at, 0) | (1 << (key % 32)));
        }
    }
    let stick = h(p, 0x1410);
    if stick <= 49 {
        wr((p + 0x1410) as *mut u16, 100);
        wr((p + 0x14a0) as *mut u32, r(p, 0xfec));
    } else {
        call(0x27f678, p);
    }
    call(0x2e9d54, p);
    wr((p + 0x1410) as *mut u16, stick as u16);
    for i in 0..4 {
        wr((p + 0x1414 + i as u32 * 4) as *mut u32, input[i]);
    }
    let changed = state != h(p, 0x2506) || action != h(p, 0x250a);
    if !changed {
        for i in 0..17 {
            wr((p + 0x24a8 + i as u32 * 4) as *mut u32, saved[i]);
        }
    } else if r(p, 0x24a8) == ROW.as_ptr() as u32 {
        wr((p + 0x24a8) as *mut u32, saved[0]);
    }
    if changed {
        info!("[psharp] cancel -> {}/{}", h(p, 0x2506), h(p, 0x250a));
    }
    changed
}

#[forge::hook(offset = 0x2d6634)]
fn sharpen(p: u32) {
    unsafe {
        let m = r(BASE, 0x1885b58);
        if p == 0 || m == 0 || call(0x276278, m) != p {
            original!(p);
            return;
        }
        if p != ACTOR || b(p, 5) == 0 {
            ACTOR = p;
            ROUND = None;
        }
        let before = f(p, 0x500);
        IN_NATIVE = true;
        original!(p);
        IN_NATIVE = false;
        if h(p, 0x250a) != 0x5a {
            ROUND = None;
            return;
        }
        if ROUND.is_none() {
            ITEM = h(p, 0x2520);
            let fast = skill(p, 0x51) || skill(p, 0x115) || skill(p, 0x127);
            let mode = b(p, 7);
            let count = if fast {
                1
            } else if mode != 0 {
                mode
            } else {
                (if b(p, 0x2b20) != 0 { 3 } else { 4 }) + skill(p, 0x52) as u32
            };
            ROUND = Session::begin(
                count,
                if ITEM == 0xfc || ITEM == 0x6b8 {
                    100
                } else {
                    150
                },
            );
            info!(
                "[psharp] begin actor={:08X} item={} strokes={}",
                p, ITEM, count
            );
        }
        if let Some(s) = ROUND.as_mut() {
            let events = s.observe(before, f(p, 0x500));
            for _ in 0..events {
                if !allowed() {
                    break;
                }
                if !s.consumed {
                    let cost: unsafe extern "C" fn(u32, u32, i32, u32, u32) -> u32 =
                        transmute(BASE + 0x3015e0);
                    COST = cost(p, ITEM, -1, 0, 0);
                    if COST > 2 {
                        warn!("[psharp] consumption rejected {}", COST);
                        break;
                    }
                }
                let delta = s.commit();
                // Native positive restores announce "maximum" even when already full.
                let applied = restore_amount(delta, r(p, 0x2774), r(p, 0x2778));
                let restore: unsafe extern "C" fn(u32, i32, u32) = transmute(BASE + 0x2ad288);
                if applied != 0 {
                    restore(p, applied as i32, 0);
                }
                info!(
                    "[psharp] stroke={}/{} delta={} applied={} sharp={}/{} cost={}",
                    s.completed,
                    s.count,
                    delta,
                    applied,
                    r(p, 0x2774),
                    r(p, 0x2778),
                    COST
                );
            }
        }
        if dodge(p) {
            ROUND = None;
        }
    }
}

#[forge::entry]
fn main() {
    unsafe {
        BASE = forge::mem::text_addr();
        for (off, head) in [
            (0x2d6634, 0xe92d4830),
            (0x2ad288, 0xe92d48f0),
            (0x3015e0, 0xe92d41f0),
        ] {
            if r(BASE, off) != head {
                error!(
                    "[psharp] incompatible function {:x}: {:08X}",
                    off,
                    r(BASE, off)
                );
                return;
            }
        }
        forge::install_hook!(BASE, sharpness);
        forge::install_hook!(BASE, consume);
        forge::install_hook!(BASE, sharpen);
        info!("[psharp] loaded v0.1.1");
    }
}
