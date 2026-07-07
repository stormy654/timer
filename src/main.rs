// TODO 
// - pause / unpause
// - flags
// - rodio
// - display
//     - stopwatch
//     - timer
//     - *alarm*
use ratatui::layout::{Constraint,Alignment};

use ratatui::style::{Color};
use ratatui::widgets::{Paragraph};
use ratatui::DefaultTerminal;
use ratatui::Frame;

use crossterm;
use crossterm::event;
use crossterm::event::{ KeyCode, Event };

use std::time::{Duration,Instant};
use std::fmt::{Display,Formatter};
use std::env;

use rodio::{ Decoder,MixerDeviceSink , Player,DeviceSinkBuilder};

use figlet_rs::FIGlet;

struct Cli {
    command:Command
}

impl Cli{
    fn parse() -> Cli {
        let mut args = env::args();
        let _program = args.next().expect("the program should have a name");
        let args:Vec<String> = args.collect();
        
        match args.len() {
            2 => {
                let arg0:&str  = &args[0]; 
                let cmd: Command = match arg0 {
                    "a" => {
                        Command::a{time:args[1].clone()}
                    },
                    "t" => {
                        Command::t{duration:args[1].clone()}
                    },
                    "s" => {
                        Command::s
                    },
                    a @ _   => {panic!("couldnt parse the first input {a} not in [a,s,t]")}
                };
                Cli { 
                    command:cmd
                }
            },
            1 => {
                if matches!(&args[0][0..],"help") {
                    println!(" 
                    This is a utility for a timer
                    USAGE:
                    timer [a,s,t]
                        t : timer t %TIME% : starts a timer 
                            timer %TIME% also works :D
                        s : timer s : starts a stopwatch
                        a : timer a %TODO% : starts an alarm (TODwO)

                    %TIME% FORMAT:
                    hours.minutes.seconds
                    example: 1.2.3 : would be 1h ,2min and 3s 

                    ADDITIONAL FORMATTING :
                    dd would me just minutes (important for me :D)
                    dd. would be minutes
                    dd.ss would be seconds
                    .ss would also be seconds
                    ");
                    std::process::exit(0);
                }else

                if matches!(&args[0][0..],"s") {
                Cli {
                    command: Command::s
                }
                }else{
                Cli {
                    command: Command::t{duration:args[0].clone()}
                }
                }
            }
            a @ _ =>  {
                panic!("{a} is the wrong amount of arguments")
            }

        }
    }
}
#[allow(non_camel_case_types,unused)]
enum Command {
    t { duration:String },
    a{ time:String },
    s,
}
#[allow(unused)]
enum Mode { 
    Stopwatch{paused:bool,sum:f64 },
    Timer{paused:bool, overflow:bool,sum:f64},
    Alarm,
}
pub struct Clock { 
    mode :  Mode,
    last_instant: Instant,
}
impl Clock { 
    fn new() -> Self { 

        let cli = Cli::parse();

        let m:Mode = match cli.command {
                    Command::a{time:_} =>todo!() ,
                    Command::s => {
                        Mode::Stopwatch {
                            paused : false,
                            sum : 0.,
                        }
                    },
                    Command::t{duration} => {
                        let Some(seconds) = parse_time(&duration) else{ 
                            panic!("Couldnt parse duration")
                        };

                        Mode::Timer {
                            paused : false,
                            overflow:seconds <0,
                            sum : seconds as f64,
                        }
                    }
                };
        Clock {
            mode:m,
            last_instant: Instant::now(),
        }
    }
    fn get_paused(&self) -> Option<bool> { 
        match self.mode{
            Mode::Timer{paused,overflow:_,sum:_} | Mode::Stopwatch{paused, sum:_}  =>{
                Some(paused)
            },
            Mode::Alarm =>{ 
                None
            }
        }
    }
    fn advance(&mut self,player:&Player){ 
        match &mut self.mode { 
            Mode::Stopwatch{paused, sum}  => {
                if *paused {
                    self.last_instant = Instant::now();
                }
                let tmp = Instant::now();
                let diff  = (tmp - self.last_instant).as_secs_f64();
                self.last_instant = tmp;
                *sum += diff;
            },
            Mode::Timer{paused, overflow,sum}  => {
                if *paused {
                    self.last_instant = Instant::now();
                }

                if !*overflow && *sum < 0.{
                    *overflow = true;
                    player.play();
                }
                let tmp = Instant::now();
                let diff  = (tmp - self.last_instant).as_secs_f64();
                self.last_instant = tmp;
                *sum -= diff;
            },
            Mode::Alarm => todo!(),
        }
    }
    fn toggle_pause(&mut self) -> Option<()>{
        match  &mut self.mode { 
            Mode::Stopwatch{ paused, sum: _ } | Mode::Timer{paused, overflow:_,sum:_} => {
                if !*paused { self.last_instant = Instant::now(); }

                *paused = !*paused;
                Some(())
            },
            Mode::Alarm => None,
        }
    }

}

impl Display for Clock { 
    fn fmt(&self,f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        let timer_minus_str = if matches!(self.mode, Mode::Timer{paused: _, overflow:true , sum:_}) {
            "-"
        } else {""};

        match &self.mode { 
            Mode::Timer{paused:_ , overflow : _ , sum} 
            | Mode::Stopwatch{paused: _,  sum} =>{
                let sum = (*sum as i64).abs();
                let hours:i64 = sum / 3600;
                let sum = sum - hours * 3600;
                let minutes  = sum / 60;
                let sum = sum - minutes * 60;
                let seconds = sum;
                
                let hours_str = if hours > 0 { &format!("{hours}:") }else {""};
                let minutes_str = if minutes > 0 || hours>0 { &format!("{minutes}:") }else {""};
                let seconds_str = &format!("{seconds}");

                write!(f,"{}",format!("{}{}{}{}",timer_minus_str,hours_str , minutes_str, seconds_str))?;
                Ok(())
            }
            Mode::Alarm => { 
                todo!();
            }
        }
    }
}

fn parse_time(s : &str)-> Option<i64> {
    let ndots = s.chars().filter(|&c| c== '.').count();
    match ndots  {
         0 => {
            let minutes = if s.is_empty() { 0 } else { s.parse().ok()? };
             return Some(minutes * 60);
        },
        1 => { 
            let mut v = s.split('.');
            let s = v.next().expect("this always has 2 elements");
            let minutes = if s.is_empty() { 0 } else { s.parse().ok()? };

            let s = v.next().expect("this always has 2 elements");
            let seconds = if s.is_empty() { 0 } else { s.parse().ok()? };

            return Some(minutes * 60 + seconds)
        },
        2 => {
            let mut v = s.split('.');

            let s = v.next().expect("this always has 3 elements");
            let hours = if s.is_empty() { 0 } else { s.parse().ok()? };

            let s = v.next().expect("this always has 3 elements");
            let minutes = if s.is_empty() { 0 } else { s.parse().ok()? };

            let s = v.next().expect("this always has 3 elements");
            let seconds = if s.is_empty() { 0 } else { s.parse().ok()? };

            
             return Some(hours * 3600 + minutes * 60 + seconds)
        },
        _ => {
            None
        }
    }
}

fn app(terminal: &mut DefaultTerminal) -> std::io::Result<()> {
    let bytes = include_bytes!("ANSI Shadow.flf");
    let font = FIGlet::from_content(std::str::from_utf8(bytes).expect("coudlnt parse default font")).expect("ERROR: COULDNT FIND FONT");

    let mut clock = Clock::new();

    let handle:MixerDeviceSink = DeviceSinkBuilder::open_default_sink()
        .expect("open default audio stream");
    let bytes = include_bytes!("ringtone.mp3");
    let source = Decoder::builder().
        with_data(std::io::Cursor::new(bytes))
        .with_hint("mp3")
        .with_gapless(true)
        .build()
        .expect("sollte compile time sein");
    let player = Player::connect_new(handle.mixer());
    player.append(source);
    player.pause();

// MAIN LOOP
    loop {
    clock.advance(&player);
        terminal.draw(|frame | render(frame,&font, &clock))?;
        
// KEY INPUTS
        if event::poll(Duration::from_millis(16))? && let Event::Key(key) = event::read()? && key.kind == event::KeyEventKind::Press {
            match key.code  {
                KeyCode::Char('q') => {
                    break Ok(());},
                KeyCode::Char(' ') => {
                    let _ = clock.toggle_pause();
                        
                    if  !player.empty() && matches!(clock.mode,Mode::Timer{paused:_,overflow:true,sum:_}) {
                        if  player.is_paused() {
                            player.play();
                        }else {
                            player.pause();
                        }
                    }
                },
                _ => {/*Dont do anything when this key isnt implemented yet*/},
            }
        }
    }
}

fn figlet(message : &str , font : &FIGlet) -> String{ 
    font.convert(message).unwrap().as_str()
}


fn render(frame: &mut Frame , font :&FIGlet, clock : & Clock) {
    let text:&str = &format!("{}",clock);
    let s : &str = &figlet(text, font);
    
    let text_color = if matches!(clock.mode,Mode::Timer{paused:_,overflow:true,sum:_}) {
        Color::Red
    } else { if let Some(paused) = clock.get_paused() && paused { 
        Color::Blue
    } else {
        Color::White
    }};



    

    frame.render_widget(
            Paragraph::new(s).
                alignment(Alignment::Center)
                .style(text_color), 
        frame.area()
        .centered(
            Constraint::Ratio(1, 2),
            Constraint::Ratio(1, 3)
        ));
}

fn main() -> std::io::Result<()> {
    ratatui::run(app)?;
    Ok(())
}
