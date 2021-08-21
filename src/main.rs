use macroquad::prelude::*;
use nalgebra::*;
use statrs::distribution::Continuous;
use std::collections::VecDeque;

type Vector = nalgebra::Vector2<f32>;

struct Snake {
    spine: VecDeque<Vector>,
    time_of_last_redraw: f64,
    head: Vector,
    glucose_level: i16,
    velocity: Vector,
}

#[macroquad::main("Snakes")]
async fn main() {
    show_mouse(false);
    let mut snake = Snake {
        spine: VecDeque::new(),
        time_of_last_redraw: 0.0,
        head: Vector::new(screen_width() / 2.0, screen_height() / 2.0),
        glucose_level: 7,
        velocity: Vector::new(10.0, 0.0),
    };
    snake.spine.push_front(snake.head);
    loop {
        let current_time = get_time();
        let mouse;
        match mouse_position() {
            (x, y) => {
                mouse = Vector::new(x, y);
            }
        }

        snake.velocity = Rotation2::new(
            compute_acceleration(snake.head, snake.velocity, mouse) * get_frame_time(),
        ) * snake.velocity;
        snake.head = snake.head + snake.velocity * get_frame_time();
        if current_time > snake.time_of_last_redraw + 1.0 {
            snake.spine.push_front(snake.head);
            snake.time_of_last_redraw = snake.time_of_last_redraw
                + std::primitive::f64::round(current_time - snake.time_of_last_redraw);
            if snake.glucose_level > 0 {
                snake.glucose_level -= 1;
            } else {
                snake.spine.pop_back();
            }
        }

        clear_background(BLACK);
        draw_mouse(mouse);
        draw_snake(&snake);

        next_frame().await
    }
}

fn draw_mouse(mouse: Vector) {
    draw_circle_lines(mouse.x, mouse.y, 10.0, 1.0, GREEN);
}

fn draw_snake(snake: &Snake) {
    for (index, vertebra) in (&snake.spine).iter().enumerate() {
        draw_circle(
            vertebra.x,
            vertebra.y,
            size_of_vertebra(index, (&snake.spine).len()),
            GREEN,
        );
    }
    draw_circle_lines(snake.head.x, snake.head.y, 3.0, 1.0, GREEN);
    draw_line(
        snake.head.x,
        snake.head.y,
        (snake.head + snake.velocity).x,
        (snake.head + snake.velocity).y,
        1.0,
        RED,
    );
}

fn compute_acceleration(position: Vector, velocity: Vector, target: Vector) -> f32 {
    let target_in_first_person =
        Rotation2::new(num::Float::atan2(velocity.x, velocity.y)) * (target - position);
    let angle_to_target = -num::Float::atan2(target_in_first_person.x, target_in_first_person.y);
    num::clamp(angle_to_target, -1.0, 1.0)
}

fn size_of_vertebra(index: usize, total: usize) -> f32 {
    statrs::distribution::Normal::new(0.0, 1.0)
        .unwrap()
        .pdf(((index + 1) as f64 / total as f64) * 2.0 - 1.0) as f32
        * total as f32
}
