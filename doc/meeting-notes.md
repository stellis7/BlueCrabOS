# July 12th, 2023
## Meet time
- Best to be on Wednesdays at 2100-2200
## Notes
- Can use confluence but not preferred
- More likely to use markdown
- Can use obsidian and use the /docs directory as a vault
## Todo
- **John:** needs to revert to test post progress and push to dev
- **Everyone:** get familiar with rust and start reading through the posts
# July 19th, 2023
## Notes
- Still need to finish post notes
- Realized that the test part of the "Barebones" section was not done, needs to be finished
- Still need to work on Rust knowledge
# July 26th, 2023
## Notes
- Everything is caught up to the "Testing" post as well as documentation
- Started to get better at Rust
- What are our end goals
    - Try to get interrupts done by summertime
    - Think of long term goals for the project
    - Maybe get all kernel components of a microkernel:
        - Basic IPC
        - Virtual Memory
        - Scheduling
- Figure out if we can meet up more frequently
- Try to do memory management ourselves
## Todos
- Look into interrupts:
    - CPU Exceptions
    - Double Faults
    - Hardware Interrupts
- Try to think of our own implementation of these topics
- Make sure to read the posts to see what he did
- **Proposed TODO**:
    - Split up into two teams:
        - Have two people work on implementing different synchronization primitives (spinlocks, mutex, semaphores)
        - Have the other three research and implement hardware interrupts
    - Hope to have some form of hardware interrupts and synchronization primitives by end of summer so we can tackle thread scheduling
# August 2nd, 2023
## Notes
- Seth did his assignment for reading the posts
- Teams decided:
    - John sits on both
    - Hardware Interrupts:
        - Seth
        - Joe
    - Synchronization primitives:
        - Nick
        - Michael
- Big ideas:
    - Built-in remote feature
        - Can run programs from server farms or locally
    - How to make UIs different
    - Be able to switch between graphical and just terminal
    - Implement Warp maybe?
    - Multi-sessions, where you could have multiple on the same computer
    - Maybe daisey chain resources from multiple devices with this OS
    - Focus alot on security
## Todos
- **SYNCH TEAM**:
    - Look into Async/Await blog post here: [Async/Await](https://os.phil-opp.com/async-await/)
    - Repo of spinlock we used in the [vga-buffer](barebones-notes/vga-buffer.md/###spinlocks) post: [spin-rs](https://github.com/mvdnes/spin-rs)
    - Repo from the rust-os dev: [spinning-top](https://github.com/rust-osdev/spinning_top)
    - Will meet on Mondays @ 1730
- **INTERRUPTS TEAM**: 
    - Read up on interrupts post (Seth already ahead!!!)
    - Will meet on Tuesdays @ 2100
- **JOHN**:
    - Sit on both meetings
    - Write email update to Lawrence:
        - Give him an update of how far we've gotten
        - Give him our github
        - Ask him if he recommends focusing on a microkernel structure
# August 9th, 2023
## Notes
- John finished CPU exceptions
    - Only difference is that interrupt test in main.rs instead of interrupts.rs
- Nick and Mike are working on testing the "spinning-lock" and "spin-rs" crates
## Todos
- Double Faults (Joe and Seth)
- Test spinning-lock and spin-rs (Mike and Nick)
- Need to write email to Lawrence (John)