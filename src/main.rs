extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;

extern crate rand;

use glutin_window::GlutinWindow;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::*;
use piston::input::*;
use piston::window::WindowSettings;

use std::collections::LinkedList;
use std::iter::FromIterator;

//
// Game Board
//

pub struct Game {
    gl: GlGraphics,
    rows: u32,
    cols: u32,
    snake: Snake,
    eaten: bool,
    square_width: u32,
    food: Food,
    score: u32,
}

impl Game {
    fn render(&mut self, args: &RenderArgs) {
        use graphics;

        const BACKGROUND_COLOR: [f32; 4] = [1.0, 1.0, 1.0, 1.0];

        self.gl.draw(args.viewport(), |_, gl| {
            graphics::clear(BACKGROUND_COLOR, gl);
        });

        self.snake.render(args);
        self.food.render(&mut self.gl, args, self.square_width)
    }

    fn update(&mut self, args: &UpdateArgs) -> bool {
        if !self.snake.update(self.eaten, self.cols, self.rows) {
            return false;
        }

        if self.eaten {
            self.score += 1;
            self.eaten = false;
        }

        self.eaten = self.food.update(&self.snake);

        // Generate a new food block.
        if self.eaten {
            use rand::thread_rng;
            use rand::Rng;

            let mut random = thread_rng();
            loop {
                let new_x = random.gen_range(0, self.cols);
                let new_y = random.gen_range(0, self.rows);

                if !self.snake.is_collide(new_x, new_y) {
                    self.food = Food { x: new_x, y: new_y };
                    break;
                }
            }
        }
        true
    }

    fn pressed(&mut self, btn: &Button) {
        let last_direction = self.snake.direction.clone();

        self.snake.direction = match btn {
            &Button::Keyboard(Key::Up) if last_direction != Direction::Down => Direction::Up,
            &Button::Keyboard(Key::Down) if last_direction != Direction::Up => Direction::Down,
            &Button::Keyboard(Key::Left) if last_direction != Direction::Right => Direction::Left,
            &Button::Keyboard(Key::Right) if last_direction != Direction::Left => Direction::Right,
            _ => last_direction,
        };
    }
}

//
// Snake
//

#[derive(Clone, PartialEq)]
enum Direction {
    Right,
    Left,
    Up,
    Down,
}

#[derive(Clone)]
pub struct SnakePiece(u32, u32);

pub struct Snake {
    gl: GlGraphics,
    snake_parts: LinkedList<SnakePiece>,
    width: u32,
    direction: Direction,
}

impl Snake {
    pub fn render(&mut self, args: &RenderArgs) {
        use graphics;

        const COLOR_SNAKE: [f32; 4] = [0.819, 0.2, 0.545, 1.0];

        let squares: Vec<graphics::types::Rectangle> = self
            .snake_parts
            .iter()
            .map(|p| SnakePiece(p.0 * self.width, p.1 * self.width))
            .map(|p| graphics::rectangle::square(p.0 as f64, p.1 as f64, self.width as f64))
            .collect();

        self.gl.draw(args.viewport(), |c, gl| {
            let transform = c.transform;
            squares
                .into_iter()
                .for_each(|square| graphics::rectangle(COLOR_SNAKE, square, transform, gl));
        })
    }

    /// Move the snake if valid, otherwise return false.
    pub fn update(&mut self, eaten: bool, cols: u32, rows: u32) -> bool {
        let mut new_head: SnakePiece =
            (*self.snake_parts.front().expect("Snake has no body.")).clone();

        if (self.direction == Direction::Up && new_head.1 == 0)
            || (self.direction == Direction::Left && new_head.0 == 0)
            || (self.direction == Direction::Down && new_head.1 == rows - 1)
            || (self.direction == Direction::Right && new_head.0 == cols - 1)
        {
            return false;
        }

        match self.direction {
            Direction::Up => new_head.1 -= 1,
            Direction::Down => new_head.1 += 1,
            Direction::Left => new_head.0 -= 1,
            Direction::Right => new_head.0 += 1,
        }

        if !eaten {
            self.snake_parts.pop_back();
        }

        // Checks self collision.
        if self.is_collide(new_head.0, new_head.1) {
            return false;
        }

        self.snake_parts.push_front(new_head);
        true
    }

    fn is_collide(&self, x: u32, y: u32) -> bool {
        self.snake_parts.iter().any(|p| x == p.0 && y == p.1)
    }
}

//
// Food
//

pub struct Food {
    x: u32,
    y: u32,
}

impl Food {
    // Return true if snake ate food this update.
    fn update(&mut self, s: &Snake) -> bool {
        let front = s.snake_parts.front().unwrap();
        if front.0 == self.x && front.1 == self.y {
            true
        } else {
            false
        }
    }

    fn render(&mut self, gl: &mut GlGraphics, args: &RenderArgs, width: u32) {
        use graphics;

        const COLOR_FOOD: [f32; 4] = [0.878, 0.439, 0.0, 1.0];

        let x = self.x * width;
        let y = self.y * width;

        let square = graphics::rectangle::square(x as f64, y as f64, width as f64);

        gl.draw(args.viewport(), |c, gl| {
            let transform = c.transform;

            graphics::rectangle(COLOR_FOOD, square, transform, gl)
        });
    }
}

fn main() {
    let opengl = OpenGL::V3_2; // OpenGL::V2_1

    const COLUMNS: u32 = 30;
    const ROWS: u32 = 20;
    const SQUARE_WIDTH: u32 = 20;

    let width = COLUMNS * SQUARE_WIDTH;
    let height = ROWS * SQUARE_WIDTH;

    let mut window: GlutinWindow = WindowSettings::new("Snake Game", [width, height])
        .opengl(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    // Initialize Game struct.
    let mut game = Game {
        gl: GlGraphics::new(opengl),
        rows: ROWS,
        cols: COLUMNS,
        eaten: false,
        square_width: SQUARE_WIDTH,
        food: Food { x: 1, y: 1 },
        score: 0,
        snake: Snake {
            gl: GlGraphics::new(opengl),
            snake_parts: LinkedList::from_iter(
                (vec![SnakePiece(COLUMNS / 2, ROWS / 2)]).into_iter(),
            ),
            width: SQUARE_WIDTH,
            direction: Direction::Down,
        },
    };

    // Event Loop
    let mut events = Events::new(EventSettings::new()).ups(8);
    while let Some(e) = events.next(&mut window) {
        if let Some(r) = e.render_args() {
            game.render(&r);
        }

        if let Some(u) = e.update_args() {
            if !game.update(&u) {
                break;
            }
        }

        if let Some(k) = e.button_args() {
            if k.state == ButtonState::Press {
                game.pressed(&k.button);
            }
        }
    }

    println!("Your score was: {:?}!", game.score);
}
