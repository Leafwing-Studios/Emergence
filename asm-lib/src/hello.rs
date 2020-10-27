use bevy::prelude::*;

struct Person;
struct Name(String);
fn add_people(mut commands: Commands) {
	commands
		.spawn((Person, Name("Elaina Proctor".to_string())))
		.spawn((Person, Name("Renzo Hume".to_string())))
		.spawn((Person, Name("Zayna Nieves".to_string())));
}
struct GreetTimer(Timer);
fn greet_people(
	time: Res<Time>,
	mut timer: ResMut<GreetTimer>,
	mut query: Query<(&Person, &Name)>,
) {
	timer.0.tick(time.delta_seconds);
	if timer.0.finished {
		for (_person, name) in &mut query.iter() {
			println!("hello {}!", name.0);
		}
	}
}

pub struct HelloPlugin;
impl Plugin for HelloPlugin {
	fn build(&self, app: &mut AppBuilder) {
		// the reason we call from_seconds with the true flag is to make the timer repeat itself
		app.add_resource(GreetTimer(Timer::from_seconds(2.0, true)))
			.add_startup_system(add_people.system())
			.add_system(greet_people.system());
	}
}
