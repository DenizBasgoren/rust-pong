use std::{thread, time};
use std::io::{stdout, Write};
use crossterm::{ queue, execute, cursor, style, Result};
use crossterm::terminal::{self, enable_raw_mode, disable_raw_mode};
use crossterm::event::{poll, read, Event, KeyCode};
use rand::random;

struct GameState {
	ball_r: (f64,f64), // (x,y)
	ball_v: (f64,f64),
	player_r: (f64,u16),
	computer_r: (f64,u16),
	player_score: u16,
	computer_score: u16
}

static W : u16 = 135;
static H : u16 = 37;



fn cleanup() {
	execute!( stdout(),
		cursor::Show,
		terminal::LeaveAlternateScreen,
		style::ResetColor
	).unwrap();
	disable_raw_mode().unwrap();
	std::process::exit(0);
}



fn draw( ctx: &mut std::io::Stdout, state: & GameState) -> Result<()> {

	let player_pad = "- -YOU- -";
	let computer_pad = "- -CPU- -";

	let player_x = state.player_r.0.floor() as u16;
	let computer_x = state.computer_r.0.floor() as u16;

	let ball_x = state.ball_r.0.floor() as u16;
	let ball_y = state.ball_r.1.floor() as u16;


	queue!( ctx,
		// clear screen
		style::ResetColor,
		terminal::Clear( terminal::ClearType::All ),

		// print scores
		cursor::MoveTo(0,0),
		style::Print( format!("{} You - {} Cpu", state.player_score, state.computer_score) ),

		// draw computer
		style::SetBackgroundColor( style::Color::DarkRed ),
		style::SetForegroundColor( style::Color::Red ),
		cursor::MoveTo(computer_x-4, state.computer_r.1),
		style::Print(computer_pad),

		// draw player
		style::SetBackgroundColor( style::Color::DarkBlue ),
		style::SetForegroundColor( style::Color::Blue ),
		cursor::MoveTo(player_x-4, state.player_r.1),
		style::Print(player_pad),

		// draw ball
		style::SetBackgroundColor( style::Color::Grey ),
		cursor::MoveTo(ball_x, ball_y),
		style::Print(" "),

		// move cursor back onto ball, to make it invisible
		cursor::MoveTo(ball_x, ball_y)


	)?;

	// flush buffer onto screen
	ctx.flush()?;

	Ok(())
}



fn get_center() -> (f64,f64) {
	( (W / 2) as f64,
	(H / 2) as f64 )
}



fn main() {
	match start_game() {
		Ok(_) => {}
		Err(_) => cleanup()
	}
}



fn random_v() -> (f64,f64) {
	let r = 0.3;
	let th = random::<f64>() * 6.28;
	( r * th.cos(),
	r * th.sin() )
}



fn start_game() -> Result<()> {

	// raw mode = no echoing, manual ctrl+c, no buffering
	enable_raw_mode()?;

	let mut ctx = stdout();

	// initial state
	let mut state = GameState {
		ball_r: get_center(),
		ball_v: random_v(),
		player_r: ( (W/2) as f64, H-3),
		computer_r: ( (W/2) as f64, 2),
		player_score: 0,
		computer_score: 0
	};

	// init
	queue!( ctx,
		cursor::Hide,
		terminal::EnterAlternateScreen,
		terminal::SetSize(W,H)
	)?;

	loop {
		draw(&mut ctx, & state)?;
		update(&mut state)?;
		thread::sleep( time::Duration::from_millis(20) );
	}
}



fn update( state: &mut GameState) -> Result<()> {

	// update ball position
	state.ball_r.0 += state.ball_v.0;
	state.ball_r.1 += state.ball_v.1;

	// ball gets faster
	state.ball_v.0 *= 1.001;
	state.ball_v.1 *= 1.001;

	// bounce back from left, right edges
	if state.ball_r.0 > W as f64 {
		state.ball_r.0 = W as f64;
		state.ball_v.0 = state.ball_v.0.abs() * -1.0;
	}
	else if state.ball_r.0 < 0.0 {
		state.ball_r.0 = 0.0;
		state.ball_v.0 = state.ball_v.0.abs();
	}

	// respawn if goal
	if state.ball_r.1 > H as f64 {
		state.ball_r = get_center();
		state.ball_v = random_v();
		state.computer_score += 1;
	}
	else if state.ball_r.1 < 0.0 {
		state.ball_r = get_center();
		state.ball_v = random_v();
		state.player_score += 1;
	}

	// if not goal, then bounce back if hit a pad
	else if state.ball_r.1 < 4.0 && (state.ball_r.0 - state.computer_r.0).abs() < 5.0 {
		state.ball_v.1 = state.ball_v.1.abs();
	}
	else if state.ball_r.1 > (H as f64)-4.0 && (state.ball_r.0 - state.player_r.0).abs() < 5.0 {
		state.ball_v.1 = state.ball_v.1.abs() * -1.0;
	}

	// pads' speed is constant
	let dx = 2.0;

	// computer algorithm
	if state.computer_r.0 - state.ball_r.0 > 1.0 {
		state.computer_r.0 -= dx;
	}
	else if state.computer_r.0 - state.ball_r.0 < -1.0 {
		state.computer_r.0 += dx;
	}

	// dont let computer go out of screen
	if (state.computer_r.0 as u16) > W-6 {
		state.computer_r.0 = (W-6) as f64;
	}
	else if (state.computer_r.0 as u16) < 5 {
		state.computer_r.0 = 5.0;
	}

	// event handler
	while poll(time::Duration::from_millis(0))? {

		match read()? {
			Event::Key(event) => {

				// move player's pad
				if event.code == KeyCode::Right {
					state.player_r.0 += dx;
				}
				else if event.code == KeyCode::Left {
					state.player_r.0 -= dx;
				}

				// if c or ctrl+c is hit, exit
				else if event.code == KeyCode::Char('c') {
					cleanup();
				}
			}

			// we are not interested in mouse, resize events
			_ => {}
		}

		// don't let player's pad go offscreen
		if (state.player_r.0 as u16) > W-6 {
			state.player_r.0 = (W-6) as f64;
		}
		else if (state.player_r.0 as u16) < 5 {
			state.player_r.0 = 5.0;
		}

	} // end of event handler
	
	Ok(())
}