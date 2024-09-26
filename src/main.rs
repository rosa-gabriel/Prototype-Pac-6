use std::{
    collections::VecDeque,
    sync::{mpsc, Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use termion::{event::Key, input::TermRead};

#[derive(Debug)]
enum CountEvent {
    Add,
    Remove,
}

#[derive(Debug)]
struct CountStruct {
    place: String,
    time: std::time::Instant,
    event: CountEvent,
}

fn main() {
    let current_city: Arc<Mutex<String>> = Arc::new(Mutex::new("Jaraguá do sul".to_string()));
    let is_connected: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));
    let (sender, receiver) = mpsc::channel::<CountStruct>();
    let stdin = std::io::stdin();

    let thread_city = current_city.clone();
    let thread_connected = is_connected.clone();
    let handler = thread::spawn(move || {
        let mut cities: VecDeque<&str> = VecDeque::new();
        cities.push_front("Jaraguá do sul");
        cities.push_front("Joinville");
        cities.push_front("Pomerode");
        cities.push_front("Florianópolis");

        loop {
            for city in cities.iter() {
                println!("Reached keyspot {}", city);
                let mut current_city_lock = thread_city.lock().unwrap();
                *current_city_lock = city.to_string();
                drop(current_city_lock);
                thread::sleep(Duration::from_secs(10));
            }
        }
    });

    let queue_handler = thread::spawn(move || {
        for event in receiver.iter() {
            println!("Sending event {:?} to server", event);

            loop {
                let connected_lock = thread_connected.lock().unwrap();
                if *connected_lock {
                    println!("Event was successfully sent to server");
                    break;
                } else {
                    println!(
                        "Failed to send event {:?} to server, retrying in 5 seconds...",
                        event
                    );
                    drop(connected_lock);
                    thread::sleep(Duration::from_secs(5))
                }
            }
        }
    });

    let thread_connected = is_connected.clone();
    let network_handler = thread::spawn(move || {
        thread::sleep(Duration::from_secs(60));
        let mut connected_lock = thread_connected.lock().unwrap();
        *connected_lock = true;
        println!("Connected to the internet!");
    });

    for k in stdin.keys() {
        let key = k.unwrap();

        let city_lock = current_city.lock().unwrap();

        let mut new_event = CountStruct {
            place: city_lock.to_string(),
            time: Instant::now(),
            event: CountEvent::Add,
        };

        drop(city_lock);

        if key == Key::Char('+') {
            new_event.event = CountEvent::Add;
        } else if key == Key::Char('-') {
            new_event.event = CountEvent::Remove;
        } else {
            continue;
        }

        sender.send(new_event).unwrap();
    }

    handler.join().unwrap();
    queue_handler.join().unwrap();
    network_handler.join().unwrap();
}
