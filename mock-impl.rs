// AXUM WEB FRAMEWORK IMPORTS
// Axum is a modern, ergonomic web framework for Rust built on top of tokio and hyper
use axum::{
    extract::{Path, Query, State}, // Extractors to get data from HTTP requests
    http::StatusCode,              // HTTP status codes (200, 404, 500, etc.)
    response::Json,                // JSON response wrapper
    routing::{get, post, put, delete}, // HTTP method routing functions
    Router,                        // Main router to define API endpoints
};
use serde::{Deserialize, Serialize}; // For JSON serialization/deserialization
use std::collections::HashMap;        // In-memory storage (replace with database later)
use std::sync::{Arc, Mutex};         // Thread-safe shared state
use tower_http::cors::CorsLayer;     // Cross-Origin Resource Sharing for web browsers

// CORE DATA STRUCTURES
// These are your existing structs with Serde traits added for JSON conversion

/// Todo represents a task that needs to be completed
/// Serde traits allow automatic conversion to/from JSON for API responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Todo {
    pub id: u32,                                    // Unique identifier - consider using UUID for production
    pub title: String,                              // Human-readable task name
    pub completed: bool,                            // Task completion status
    pub description: Option<String>,                // Detailed task description (optional)
    pub due_date: Option<String>,                   // ISO date string (consider using chrono::DateTime)
    pub personal_notes: Option<String>,             // User's private notes about progress
    pub completion_percentage: Option<u8>,          // 0-100 completion percentage
    pub location_triggers: Option<Vec<LocationTrigger>>, // Geographic triggers for notifications
}

/// LocationTrigger defines a geographic area that can trigger notifications
/// When a user enters this area, the associated Todo becomes "active"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationTrigger {
    pub id: u32,          // Unique identifier for this trigger
    pub name: String,     // Human-readable name (e.g., "Home", "Office", "Grocery Store")
    pub latitude: f64,    // GPS coordinate - North/South position (-90 to +90)
    pub longitude: f64,   // GPS coordinate - East/West position (-180 to +180)
    pub radius: f64,      // Trigger distance in meters (geofence radius)
}

// API REQUEST/RESPONSE STRUCTURES
// These structs define the shape of data sent to/from the API

/// LocationQuery represents GPS coordinates sent from the iOS app
/// Used in the /todos/nearby endpoint to find location-relevant todos
#[derive(Debug, Deserialize)]
pub struct LocationQuery {
    lat: f64,  // Current latitude from iOS device
    lng: f64,  // Current longitude from iOS device
}

/// CreateTodo represents the payload when creating a new todo
/// Similar to Todo but without ID (server generates it) and defaults completed to false
#[derive(Debug, Deserialize)]
pub struct CreateTodo {
    pub title: String,                              // Required: task name
    pub description: Option<String>,                // Optional: detailed description
    pub due_date: Option<String>,                   // Optional: deadline
    pub personal_notes: Option<String>,             // Optional: user notes
    pub completion_percentage: Option<u8>,          // Optional: initial progress
    pub location_triggers: Option<Vec<LocationTrigger>>, // Optional: geographic triggers
}

// SHARED APPLICATION STATE
// Arc<Mutex<HashMap>> provides thread-safe shared access to todo storage
// Arc = Atomic Reference Counter (multiple owners)
// Mutex = Mutual Exclusion (thread-safe access)
// HashMap = Key-value storage (todo_id -> Todo)
type TodoStore = Arc<Mutex<HashMap<u32, Todo>>>;

// SIMPLE ID GENERATION
// In production, use UUID or database auto-increment
// This is unsafe but simple for demonstration
static mut NEXT_ID: u32 = 1;

/// Generates the next available ID for new todos
/// WARNING: This is not thread-safe or production-ready
/// Consider using AtomicU32 or UUID instead
fn get_next_id() -> u32 {
    unsafe {
        let id = NEXT_ID;
        NEXT_ID += 1;
        id
    }
}

// GEOSPATIAL CALCULATIONS

/// Calculates the distance between two GPS coordinates using the Haversine formula
/// This accounts for the Earth's curvature and provides accurate distances
/// 
/// Parameters:
/// - lat1, lng1: First GPS coordinate (e.g., user's current location)
/// - lat2, lng2: Second GPS coordinate (e.g., todo's location trigger)
/// 
/// Returns: Distance in meters between the two points
fn calculate_distance(lat1: f64, lng1: f64, lat2: f64, lng2: f64) -> f64 {
    const EARTH_RADIUS_METERS: f64 = 6371000.0; // Earth's radius in meters
    
    // Convert degrees to radians for trigonometric functions
    let lat1_rad = lat1.to_radians();
    let lat2_rad = lat2.to_radians();
    let delta_lat = (lat2 - lat1).to_radians();
    let delta_lng = (lng2 - lng1).to_radians();

    // Haversine formula for great-circle distance
    // This calculates the shortest distance between two points on a sphere
    let a = (delta_lat / 2.0).sin().powi(2) + 
            lat1_rad.cos() * lat2_rad.cos() * (delta_lng / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

    EARTH_RADIUS_METERS * c
}

// REST API ENDPOINT HANDLERS
// Each function handles a specific HTTP endpoint and operation

/// GET /todos - Returns all todos in the system
/// iOS app can use this to sync all todos for offline access
async fn get_todos(State(store): State<TodoStore>) -> Json<Vec<Todo>> {
    let todos = store.lock().unwrap(); // Get exclusive access to the todo store
    let todo_list: Vec<Todo> = todos.values().cloned().collect(); // Convert HashMap values to Vec
    Json(todo_list) // Automatically serializes to JSON response
}

/// GET /todos/{id} - Returns a specific todo by ID
/// Useful for getting detailed information about a single todo
async fn get_todo(
    Path(id): Path<u32>,           // Extract ID from URL path
    State(store): State<TodoStore>, // Get access to shared todo storage
) -> Result<Json<Todo>, StatusCode> {
    let todos = store.lock().unwrap();
    match todos.get(&id) {
        Some(todo) => Ok(Json(todo.clone())), // Found: return the todo
        None => Err(StatusCode::NOT_FOUND),   // Not found: return 404
    }
}

/// POST /todos - Creates a new todo
/// iOS app uses this to add new todos with location triggers
async fn create_todo(
    State(store): State<TodoStore>,    // Access to todo storage
    Json(payload): Json<CreateTodo>,   // Extract JSON payload from request body
) -> Json<Todo> {
    // Create new todo with generated ID and default values
    let todo = Todo {
        id: get_next_id(),                      // Generate unique ID
        title: payload.title,                   // Use provided title
        completed: false,                       // New todos start incomplete
        description: payload.description,       // Optional description
        due_date: payload.due_date,            // Optional due date
        personal_notes: payload.personal_notes, // Optional notes
        completion_percentage: payload.completion_percentage, // Optional progress
        location_triggers: payload.location_triggers, // Optional location triggers
    };

    // Store the new todo and return it
    let mut todos = store.lock().unwrap();
    todos.insert(todo.id, todo.clone());
    Json(todo) // Return the created todo with its assigned ID
}

/// PUT /todos/{id} - Updates an existing todo
/// iOS app uses this to mark todos complete, update progress, etc.
async fn update_todo(
    Path(id): Path<u32>,               // Todo ID from URL
    State(store): State<TodoStore>,    // Access to storage
    Json(payload): Json<Todo>,         // New todo data from request body
) -> Result<Json<Todo>, StatusCode> {
    let mut todos = store.lock().unwrap();
    if todos.contains_key(&id) {
        todos.insert(id, payload.clone()); // Replace existing todo
        Ok(Json(payload))                  // Return updated todo
    } else {
        Err(StatusCode::NOT_FOUND)         // Todo doesn't exist
    }
}

/// DELETE /todos/{id} - Removes a todo
/// iOS app can use this to delete completed or unwanted todos
async fn delete_todo(
    Path(id): Path<u32>,            // Todo ID to delete
    State(store): State<TodoStore>, // Access to storage
) -> Result<StatusCode, StatusCode> {
    let mut todos = store.lock().unwrap();
    if todos.remove(&id).is_some() {
        Ok(StatusCode::NO_CONTENT)  // Successfully deleted (204)
    } else {
        Err(StatusCode::NOT_FOUND)  // Todo didn't exist (404)
    }
}

/// GET /todos/nearby?lat=37.7749&lng=-122.4194
/// CRITICAL ENDPOINT: This is the core of your location-based notification system
/// 
/// When the iOS app detects a location change, it calls this endpoint with the
/// current GPS coordinates. The server responds with todos that have location
/// triggers within range of the current position.
/// 
/// The iOS app can then display local notifications for these nearby todos.
async fn get_nearby_todos(
    Query(location): Query<LocationQuery>, // Extract lat/lng from query parameters
    State(store): State<TodoStore>,        // Access to todo storage
) -> Json<Vec<Todo>> {
    let todos = store.lock().unwrap();
    
    // Filter todos to find those with location triggers near the current position
    let nearby_todos: Vec<Todo> = todos
        .values()
        .filter(|todo| {
            // Only check todos that have location triggers defined
            if let Some(ref triggers) = todo.location_triggers {
                // Check if ANY trigger is within range of current location
                triggers.iter().any(|trigger| {
                    // Calculate distance between current location and trigger
                    let distance = calculate_distance(
                        location.lat,      // Current latitude from iOS
                        location.lng,      // Current longitude from iOS
                        trigger.latitude,  // Trigger's latitude
                        trigger.longitude, // Trigger's longitude
                    );
                    
                    // Todo is "nearby" if:
                    // 1. Distance is within the trigger's radius AND
                    // 2. Todo is not already completed
                    distance <= trigger.radius && !todo.completed
                })
            } else {
                false // No location triggers = not location-based
            }
        })
        .cloned() // Create owned copies of the todos
        .collect();

    Json(nearby_todos) // Return nearby todos as JSON
}

// MAIN APPLICATION SETUP

#[tokio::main] // Enables async main function with tokio runtime
async fn main() {
    // Initialize shared application state (todo storage)
    let store: TodoStore = Arc::new(Mutex::new(HashMap::new()));

    // ADD SAMPLE DATA FOR TESTING
    // In production, this would be loaded from a database
    {
        let mut todos = store.lock().unwrap();
        let sample_todo = Todo {
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
                    latitude: 37.7749,    // San Francisco coordinates
                    longitude: -122.4194,
                    radius: 100.0,        // 100 meter radius
                },
            ]),
        };
        todos.insert(1, sample_todo);
    }

    // BUILD THE API ROUTER
    // This defines all available endpoints and their HTTP methods
    let app = Router::new()
        // Todo CRUD operations
        .route("/todos", get(get_todos).post(create_todo))              // GET /todos, POST /todos
        .route("/todos/:id", get(get_todo).put(update_todo).delete(delete_todo)) // GET/PUT/DELETE /todos/{id}
        
        // Location-based endpoint (MOST IMPORTANT for iOS integration)
        .route("/todos/nearby", get(get_nearby_todos))                 // GET /todos/nearby?lat=X&lng=Y
        
        // Enable CORS for web browser access (if building a web interface later)
        .layer(CorsLayer::permissive())
        
        // Inject shared state into all handlers
        .with_state(store);

    // START THE SERVER
    // Bind to all network interfaces (0.0.0.0) so iOS devices on the same network can connect
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("🚀 Todo Location Server running on http://0.0.0.0:3000");
    println!("📱 iOS app can connect to: http://[YOUR_COMPUTER_IP]:3000");
    println!("🔍 Test nearby todos: http://localhost:3000/todos/nearby?lat=37.7749&lng=-122.4194");
    
    // Run the server indefinitely
    axum::serve(listener, app).await.unwrap();
}

// IMPLEMENTATION ROADMAP FOR YOUR iOS APP:
//
// 1. LOCATION MONITORING:
//    - Request location permissions in iOS
//    - Use CLLocationManager to track significant location changes
//    - Implement background location updates for notifications
//
// 2. API INTEGRATION:
//    - Create Swift structs matching your Rust structs
//    - Implement HTTP client to call these endpoints
//    - Handle JSON encoding/decoding
//
// 3. NOTIFICATION SYSTEM:
//    - When location changes significantly, call /todos/nearby
//    - If nearby todos are found, schedule local notifications
//    - Handle notification taps to open relevant todos
//
// 4. OFFLINE SUPPORT:
//    - Periodically call /todos to sync all todos
//    - Store todos locally using Core Data or SQLite
//    - Show cached todos when offline
//
// 5. BACKGROUND PROCESSING:
//    - Use background app refresh to check for nearby todos
//    - Implement efficient location monitoring to preserve battery
//    - Cache location-based queries to reduce server load