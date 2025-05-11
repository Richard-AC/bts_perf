use std::arch::asm;
use std::time::Instant;

/// If true, use the memory operand variant of bts.
/// If false, use the manual implementation.
const USE_MEMORY_OPERAND_BTS: bool = true;

/// Number of worker threads to spawn
const THREAD_COUNT: usize = 20;

/// Whether to randomize the index of the checked bit.
/// The performance issue is more obvious if all threads always check the same
/// bit.
const RANDOMIZE_INDEX: bool = false;

/// Size of the bitmap in bytes
const BITMAP_SIZE: usize = 512 * 1024 * 1024;

struct BitMap {
    /// Memory for the bitmap
    backing: Box<[u8; BITMAP_SIZE]>
}

impl BitMap {
    fn new() -> Self {
        let backing: Box<[u8; BITMAP_SIZE]> =
            vec![0u8; BITMAP_SIZE].into_boxed_slice().try_into().unwrap();
        Self { backing }
    }

    /// Return a pointer to the start of the bitmap
    unsafe fn base(&mut self) -> *mut u8 {
        self.backing.as_mut_ptr()
    }
}

#[inline(always)]
pub fn rdtsc() -> u64 {
    unsafe { std::arch::x86_64::_rdtsc() }
}

/// Simple rng
fn xorshift(prev: u64) -> u64 {
    let mut next = prev;
    next ^= next << 13;
    next ^= next >> 17;
    next ^= next << 43;
    next
}

/// Loop forever and perform a bit test and set either using the memory operand
/// variant of bts or a "manual" implementation which loads the bit, use the
/// register variant of bts and conditionally store the set bit back.
unsafe fn worker(bitmap_base: *mut u8, id: usize) {
    let mut iter: usize = 0;
    let start = Instant::now();
    let mut max_observed = 0;
    let mut prev = start;

    let mut rng = rdtsc();

    loop {
        rng = if RANDOMIZE_INDEX {
            xorshift(rng)
        } else {
            123123
        };

        let random_bit = rng % (BITMAP_SIZE as u64 * 8);

        if USE_MEMORY_OPERAND_BTS {
            asm!(r#"
                bts [rdi], rcx
            "#,
            in("rdi") bitmap_base,
            in("rcx") random_bit);
        } else {
            asm!(r#"
                mov rax, rcx
                shr rax, 3

                // Load the u64 containing the desired bit
                mov rdx, [rdi + rax]

                // Note: BT takes the mod 64 of the second operand
                bts rdx, rcx

                jc 2f

                // Store the new value with the bit set
                mov [rdi + rax], rdx

                2:

            "#,
            in("rdi") bitmap_base,
            in("rcx") random_bit,
            out("rax") _,
            out("rdx") _);
        }

        iter += 1;

        if id == 0 && iter % 50_000_000 == 0 {
            let elapsed = prev.elapsed().as_secs_f64();
            let iter_per_sec = ((50_000_000 / 1_000_000) as f64 / elapsed) as u64;
            max_observed = std::cmp::max(max_observed, iter_per_sec);
            println!("{iter_per_sec} M iter per sec per thread (max observed: {max_observed} M)");
            prev = Instant::now();
        }
    }
}

fn main() {
    let mut bitmap = BitMap::new();
    let bitmap_base = unsafe { bitmap.base() as usize };


    let mut thread_handles = Vec::new();

    for id in 0..THREAD_COUNT {
        let handle = std::thread::spawn(move || {
            unsafe { worker(bitmap_base as _, id); }
        });
        thread_handles.push(handle);
    }

    for h in thread_handles { h.join().unwrap() }
}
