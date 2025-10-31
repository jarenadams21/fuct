// What is a TODO?
// A TODO is a activity that needs to be done.
// The traits include:
// - id: A unique identifier for the TODO item.
// - title: A brief title or summary of the TODO item.
// - completed: A boolean indicating whether the TODO item has been completed.
// - description: An optional detailed description of the TODO item. Could be an assignment description or steps to complete the task for example.
// - due_date: An optional due date for the TODO item.
// - personal_notes: Optional personal notes to use against AI semantics to gauge if the TODO is complete enough.
//   This is where you would list what you have done so far to complete the TODO item.
// - completion_percentage: An optional percentage indicating how much of the TODO item has been completed.
// - location_triggers: Optional location-based triggers that can be associated with the TODO item.
pub struct Todo {
    pub id: u32,
    pub title: String,
    pub completed: bool,
    pub description: Option<String>,
    pub due_date: Option<String>,
    pub personal_notes: Option<String>,
    pub completion_percentage: Option<u8>,
    pub location_triggers: Option<Vec<LocationTrigger>>,
}

// What is a LocationTrigger?
// A LocationTrigger is a geographical point that can be associated with a TODO item.
// The traits include:
// - id: A unique identifier for the LocationTrigger.
// - name: A brief name or description of the location trigger.
// - latitude: The latitude coordinate of the location trigger.
// - longitude: The longitude coordinate of the location trigger.
// - radius: The radius (in meters) around the location that defines the trigger area.
// Location triggers will be constructed in spherical geometry.
pub struct LocationTrigger {
    pub id: u32,
    pub name: String,
    pub latitude: f64,
    pub longitude: f64,
    pub radius: f64, // in meters
}

fn main() {
    let todo = Todo {
        id: 1,
        title: String::from("Finish Rust project"),
        completed: false,
        description: Some(String::from("Complete the Rust project for the client.")),
        due_date: Some(String::from("2024-07-01")),
        personal_notes: Some(String::from("Have completed the initial setup and basic functionality.")),
        completion_percentage: Some(50),
        location_triggers: Some(vec![
            LocationTrigger {
                id: 1,
                name: String::from("Home"),
                latitude: 37.7749,
                longitude: -122.4194,
                radius: 100.0,
            },
        ]),
    };

    println!("TODO Item: {}", todo.title);
    if let Some(ref location_triggers) = todo.location_triggers {
        for trigger in location_triggers {
            println!("Location Trigger: {} at ({}, {}) with radius {} meters", trigger.name, trigger.latitude, trigger.longitude, trigger.radius);
        }
    }
}
