extern crate num_cpus;
extern crate chan;

use std::thread;

fn main() {
    let cpus = num_cpus::get();
    println!("Available CPUs: {}", cpus);

    let (send_result, recv_result) = chan::async();
    let (send_work, recv_work) = chan::sync(512);

    let generator_thread = thread::spawn(move || {
        for x in 0..10 {
            send_work.send(x);
        }
    });

    let result_thread = thread::spawn(move || {
        for (w, r) in recv_result.iter() {
            println!("got result: {} {}", w, r);
        }
    });

    let mut worker_threads = Vec::new();
    for _ in 0..cpus {
        let send_result = send_result.clone();
        let recv_work = recv_work.clone();
        worker_threads.push(thread::spawn(move || {
            while let Some(w) = recv_work.recv() {
                thread::sleep(::std::time::Duration::from_millis(1000));
                send_result.send((w, w*w));
            }
        }));
    }

    for t in worker_threads.into_iter() {
        t.join().unwrap();
    }
    std::mem::drop(send_result);

    result_thread.join().unwrap();

    generator_thread.join().unwrap();
}
