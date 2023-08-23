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
- 