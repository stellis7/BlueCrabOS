# Double Faults
## What is it?
- Special exception that occurs when the CPU fails to invoke an exception handler
- Similar to catch-all blocks in programming languages with exceptions
- To provoke one: 
```rust
unsafe {
    *(0xdeadbeef as *mut u8) = 42;
}
```
- This writes 42 to invalid address `0xdeadbeef`
- Since the virtual address is not mapped to a physical one in the page tables, a page fault occurs
- We also have no page fault handler, so a double fault occurs
- The following boot loop then happens
    1. The CPU tries to write to `0xdeadbeef` causing a page fault
    2. No handler function for a page fault, causing a double fault
    3. No handler function for a double fault, causing a triple fault
    4. Triple faults are fatal, meaning the system then resets
- Good to start with a double fault for any faults that may arise with no handler functions
## Double Fault Handler
- Add double fault handler function similar to breakpoint handler, by adding the following inside of the static IDT initialization
```rust
idt.double_fault.set_handler_fn(double_fault_handler);
```
- Then need to define the handler function as the following:
```rust
extern "x86-interrupt" fn double_fault_handler(stack_frame: InterruptStackFrame, _error_code: u64) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}
```
- This is an example of a diverging function, incidication by the return type `!`, meaning that it does not return
- This is because the x86_64 architecture does not allow returning from a double fault
- This does not catch all double faults though, so more work must be done
## Causes of Double Faults
- According to the AMD64 manual: "double fault exception can occur when a second exception occurs during the handling of a prior (first) exception handler."
- Only very specific combos of exceptions lead to a double fault:
<table>
	<tr>
		<th>First Exception</th>
		<th>Second Exception</th>
	</tr>
	<tr>
		<td>
            <a href="https://wiki.osdev.org/Exceptions#Division_Error">Divide-by-zero</a>,<br>
            <a href="https://wiki.osdev.org/Exceptions#Invalid_TSS">Invalid TSS</a>,<br>
            <a href="https://wiki.osdev.org/Exceptions#Segment_Not_Present">Segment Not Present</a>,<br>
            <a href="https://wiki.osdev.org/Exceptions#Stack-Segment_Fault">Stack-Segment Fault</a>,<br>
            <a href="https://wiki.osdev.org/Exceptions#General_Protection_Fault">General Protection Fault</a>
        </td>
		<td>
        <a href="https://wiki.osdev.org/Exceptions#Invalid_TSS">Invalid TSS</a>,<br>
            <a href="https://wiki.osdev.org/Exceptions#Segment_Not_Present">Segment Not Present</a>,<br>
            <a href="https://wiki.osdev.org/Exceptions#Stack-Segment_Fault">Stack-Segment Fault</a>,<br>
            <a href="https://wiki.osdev.org/Exceptions#General_Protection_Fault">General Protection Fault</a>
        </td>
	</tr>
	<tr>
		<td><a href="https://wiki.osdev.org/Exceptions#Page_Fault">Page Fault</a></td>
		<td>
            <a href="https://wiki.osdev.org/Exceptions#Page_Fault">Page Fault</a>,<br>
            <a href="https://wiki.osdev.org/Exceptions#Invalid_TSS">Invalid TSS</a>,<br>
            <a href="https://wiki.osdev.org/Exceptions#Segment_Not_Present">Segment Not Present</a>,<br>
            <a href="https://wiki.osdev.org/Exceptions#Stack-Segment_Fault">Stack-Segment Fault</a>,<br>
            <a href="https://wiki.osdev.org/Exceptions#General_Protection_Fault">General Protection Fault</a>
        </td>
	</tr>
</table>

- This means that a divide-by-zero followed by a page fault is fine, but a divide-by-zero followed by a general-protection fault causes a double fault
- This would also follow when there is handler function for an exception, which lands as a general protection fault. There is no entry for a general protection fault, so this causes a double fault.
### Kernel Stack Overflow
- What happens if our kernel overflows its stack and the guard page is hit?
- Guard page is a special memory page at the bottom of a stack that makes it possible to detect stack overflows
- It's not mapped to any physical frame, so it causes a page fault when accessing it
- This means a stack overflow causes a page fault
## Switching Stacks
- The x86_64 architecture allows us to solve this by allowing us to switch to a predefined, known-good stack when an exception occurs
- Switch happens at hardware level, so it can occur before the CPU pushes the exception stack frame
- The switching mechanism is implemented as an Interrupt Stack Table (IST)
- The IST is a table of 7 pointers to known-good stacks
- Can choose a stack for each exception handler from the IST with the `stack_pointers` field in the IDT.
### The IST and TSS
- The IST is part of an old legacy structure called [Task State Segment](https://en.wikipedia.org/wiki/Task_state_segment) (TSS)
- TSS holds various pieces of info about a task in 32-bit mode
- Used for [hardware context switching](https://wiki.osdev.org/Context_Switching#Hardware_Context_Switching)
- This is not supported in 64-bit mode though
- On x86_64, the TSS now only holds two stack tables (including the IST)
- Only common field is the pointer to the I/O port permissions bitmap
- 64-bit TSS has the following format:
<table>
	<tr>
		<th>Field</th>
		<th>Type</th>
	</tr>
	<tr>
		<td>(reserved)</td>
		<td>u32</td>
	</tr>
	<tr>
		<td>Privilege Stack Table</td>
		<td>[u64; 3]</td>
	</tr>
    <tr>
		<td>(reserved)</td>
		<td>[u64; 3]</td>
	</tr>
    <tr>
		<td>Privelege Stack Table</td>
		<td>u64</td>
	</tr>
    <tr>
		<td>Interrupt Stack Table</td>
		<td>[u64; 7]</td>
	</tr>
    <tr>
		<td>(reserved)</td>
		<td>u64</td>
	</tr>
    <tr>
		<td>(reserved)</td>
		<td>u16</td>
	</tr>
    <tr>
		<td>I/O Map Base Address</td>
		<td>u16</td>
	</tr>
</table>

- The *Privilege Stack Table* is used by the CPU when the privilege changes
### Creating a TSS
- Create a new TSS that contains a separate double fault stack in its IST
- Need a TSS struct, which we can use the `x86_64` crate's TSS
- Create new gdt module, adding `pub mod gdt;` in lib.rs and add new `gdt.rs` file
```rust
use x86_64::VirtAddr;
use x86_64::structures::tss::TaskStateSegment;
use lazy_static::lazy_static;

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 4096 * 5;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
            let stack_end = stack_start + STACK_SIZE;
            stack_end
        };
        tss
    };
}
```
- Use lazy_static so that it can be initialized at compile time
- Define the 0th IST entry is the double fault stack
- Then write the top address of a double fault stack to the 0th entry
- Use `static mut` array as stack storage since we don't have memory management yet
- Need `unsafe` because the compiler can't guarantee race freedom when mutable statics are accessed
- Needs to be mutable because otherwise the bootloader will map it to a read-only page
- **NOTE:** Need to replace this later on with proper memory management
- Need to add a segment descriptor to the GDT instead of loading the TSS directly because it uses the segmentation system
- Then load the TSS by invoking the `ltr` instruction with the respective GDT index
### The Global Descriptor Table
- It is a relic used for [memory segmentation](https://en.wikipedia.org/wiki/X86_memory_segmentation) begore paging became the de facto standard
- It is still needed in 64-bit mode for things like kernel/user mode config or TSS loading
- GDT is a structure that contains the segments of the program
- Create a static GDT that includes a segment for our TSS static (not included since changed later for [final steps](#the-final-steps)):
```rust
use x86_64::structures::gdt::{GlobalDescriptorTable, Descriptor};

lazy_static! {
    static ref GDT: GlobalDescriptorTable = {
        let mut gdt = GlobalDescriptorTable::new();
        gdt.add_entry(Descriptor::kernel_code_segment());
        gdt.add_entry(Descriptor::tss_segment(&TSS));
        gdt
    };
}
```
- Then create a new `init()` function for the GDT and call it in `lib.rs`
```rust
// in src/gdt.rs

pub fn init() {
    GDT.load();
}

// in src/lib.rs

pub fn init() {
    gdt::init();
    interrupts::init_idt();
}
```
- Now it is loaded, but still boot loops on stack overflow
### The Final Steps
- This still happens because the GDT segments are not yet active because the segment and TSS registers still contain the values from the old GDT
- Also need to modify the double fault IDT entry so that it uses the new stack
- Summary:
    1. Reload code segment register: changed the GDT so need to reload `cs` since the old segment selector could now point to a different GDT descriptor
    2. Load the TSS: Still need to tell the CPU that it should use the GDT's TSS
    3. Update the IDT entry: When TSS is loaded, CPU has access to a valid IST. Then we can tell the CPU that it should use our new double fault stack by modifying our double fault IDT entry
- For first two steps, need to access `code_selector` and `tss_selector` variables in our gdt's init function
- Can do this by making them part of the static through a new `Selectors` struct:
```rust
use x86_64::structures::gdt::SegmentSelector;

lazy_static! {
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();
        let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
        let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));
        (gdt, Selectors { code_selector, tss_selector })
    };
}

struct Selectors {
    code_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}
```
- Then use the selectors to reload the `cs` register and load our `TSS`:
```rust
pub fn init() {
    use x86_64::instructions::tables::load_tss;
    use x86_64::instructions::segmentation::{CS, Segment};
    
    GDT.0.load();
    unsafe {
        CS::set_reg(GDT.1.code_selector);
        load_tss(GDT.1.tss_selector);
    }
}
```
- Reload using `CS::set_reg` and load the TSS with `load_tss`
- Need to use `unsafe for them`
- Now can set the stack index for our double fault handler in the IDT:
```rust
lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
            idt.double_fault.set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX); // new
        }

        idt
    };
}
```
- Method `set_stack_index` unsafe because the caller must ensure that the used index is valid and not already used for another exception
- Now able to catch all double faults, even stack overflows
## Stack Overflow Test
- First create new minimal skeleton and add it to the Cargo.toml file as a test
- Need to add the tst its own IDT with a custom double fault handler:
```rust
use lazy_static::lazy_static;
use x86_64::structures::idt::InterruptDescriptorTable;

lazy_static! {
    static ref TEST_IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        unsafe {
            idt.double_fault
                .set_handler_fn(test_double_fault_handler)
                .set_stack_index(blue_crab_os::gdt::DOUBLE_FAULT_IST_INDEX);
        }

        idt
    };
}

pub fn init_test_idt() {
    TEST_IDT.load();
}
```
- Need to test a stack overflow occurring, so add a dummy volatile read statement at the end of a function, allowing endless recursion:
```rust
use blue_crab::serial_print;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    serial_print!("stack_overflow::stack_overflow...\t");

    blue_crab::gdt::init();
    init_test_idt();

    // trigger a stack overflow
    stack_overflow();

    panic!("Execution continued after stack overflow");
}

#[allow(unconditional_recursion)]
fn stack_overflow() {
    stack_overflow(); // for each recursion, the return address is pushed
    volatile::Volatile::new(0).read(); // prevent tail recursion optimizations
}
```