use ::rand::distributions::Distribution;
use ::rand::Rng;
use macroquad::prelude::*;
use nalgebra::Rotation2;
use statrs::distribution::Continuous;
use std::collections::VecDeque;

const VERTEBRAE_PER_SECOND: f64 = 1.0;
const GLUCOSE_LEVEL_AT_START: i16 = 10;
const AVERAGE_APPLES_PER_SECOND: f64 = 0.4;
const APPLE_LIFE_TIME: f64 = 40.0;
const SIZE_OF_VERTEBRA_COEFFICIENT: f32 = 0.8;

type Vector = nalgebra::Vector2<f32>;

struct Snake {
    spine: VecDeque<Vector>,
    time_of_last_redraw: f64,
    head: Vector,
    glucose_level: i16,
    velocity: Vector,
}

struct Apple {
    time_of_creation: f64,
    size: i16,
    position: Vector,
}

#[macroquad::main("Snakes")]
async fn main() {
    show_mouse(false);
    let mut random = ::rand::thread_rng();
    let mut snake = Snake {
        spine: VecDeque::new(),
        time_of_last_redraw: 0.0,
        head: Vector::new(screen_width() / 2.0, screen_height() / 2.0),
        glucose_level: GLUCOSE_LEVEL_AT_START,
        velocity: Vector::new(20.0, 0.0),
    };
    snake.spine.push_front(snake.head);
    let mut apples = VecDeque::new();
    apples.push_back(create_apple_at_time(0.0, &mut random));

    loop {
        let current_time = get_time();
        let frame_time = get_frame_time();
        let mouse;
        match mouse_position() {
            (x, y) => {
                mouse = Vector::new(x, y);
            }
        }

        snake.velocity =
            Rotation2::new(compute_acceleration(snake.head, snake.velocity, mouse) * frame_time)
                * snake.velocity;
        snake.head = snake.head + snake.velocity * frame_time;
        if current_time > snake.time_of_last_redraw + 1.0 / VERTEBRAE_PER_SECOND {
            snake.spine.push_front(snake.head);
            snake.time_of_last_redraw = snake.time_of_last_redraw
                + std::primitive::f64::round(
                    current_time - snake.time_of_last_redraw * VERTEBRAE_PER_SECOND,
                ) / VERTEBRAE_PER_SECOND;
            if snake.glucose_level > 0 {
                snake.glucose_level -= 1;
            } else {
                snake.spine.pop_back();
            }
        }

        if ::rand::distributions::Bernoulli::new(std::primitive::f64::min(
            frame_time as f64 * AVERAGE_APPLES_PER_SECOND,
            1.0,
        ))
        .unwrap()
        .sample(&mut random)
        {
            let apple = create_apple_at_time(current_time, &mut random);
            if does_apple_collide_with_snake(&apple, &snake) {
            } else {
                apples.push_back(apple);
            }
        }

        apples.retain(|apple| {
            if is_apple_eaten_by_snake(&apple, &snake) {
                snake.glucose_level += apple.size;
                false
            } else {
                if current_time - apple.time_of_creation < APPLE_LIFE_TIME {
                    true
                } else {
                    false
                }
            }
        });

        clear_background(BLACK);
        draw_apples(&apples, current_time);
        draw_snake(&snake);
        draw_mouse(mouse);

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

fn draw_apples(apples: &VecDeque<Apple>, current_time: f64) {
    for apple in apples {
        draw_circle(
            apple.position.x,
            apple.position.y,
            apple.size as f32,
            if current_time - apple.time_of_creation < APPLE_LIFE_TIME / 2.0 {
                RED
            } else {
                BROWN
            },
        );
    }
}

fn create_apple_at_time(current_time: f64, random: &mut ::rand::rngs::ThreadRng) -> Apple {
    Apple {
        time_of_creation: current_time,
        size: std::primitive::f64::round(
            statrs::distribution::Beta::new(2.0, 10.0)
                .unwrap()
                .pdf(random.gen())
                * 2.0,
        ) as i16,
        position: Vector::new(
            random.gen_range(1.0..screen_width()),
            random.gen_range(1.0..screen_height()),
        ),
    }
}

fn compute_acceleration(position: Vector, velocity: Vector, target: Vector) -> f32 {
    let target_in_first_person =
        Rotation2::new(num::Float::atan2(velocity.x, velocity.y)) * (target - position);
    let angle_to_target = -num::Float::atan2(target_in_first_person.x, target_in_first_person.y);
    num::clamp(angle_to_target, -1.0, 1.0)
}

fn size_of_vertebra(index: usize, total: usize) -> f32 {
    (statrs::distribution::Normal::new(0.0, 1.0)
        .unwrap()
        .pdf(((index + 1) as f64 / total as f64) * 4.0 - 2.0) as f32
        * total as f32).powf(1.0/3.0) * SIZE_OF_VERTEBRA_COEFFICIENT
}

fn does_apple_collide_with_snake(apple: &Apple, snake: &Snake) -> bool {
    (&snake.spine)
        .iter()
        .enumerate()
        .map(|(index, vertebra)| {
            if (apple.position - vertebra).norm()
                < apple.size as f32 + size_of_vertebra(index, (&snake.spine).len())
            {
                true
            } else {
                false
            }
        })
        .fold(false, |x, y| x || y)
}

fn is_apple_eaten_by_snake(apple: &Apple, snake: &Snake) -> bool {
    if (apple.position - snake.head).norm()
        < apple.size as f32
            + size_of_vertebra(
                ((&snake.spine).len() as f32 / 2.0) as usize,
                (&snake.spine).len(),
            )
    {
        true
    } else {
        false
    }
}
