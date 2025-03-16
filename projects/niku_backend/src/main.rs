//! Backend in charge of making discovery possible on NIKU.

use std::collections::HashMap;
use std::env::VarError;
use std::sync::Arc;
use std::time::Duration;
use std::{env, io};

use axum::extract::{Json, MatchedPath, Path, Request, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post, put};
use axum::Router;
use const_format::formatcp;
use niku_core::{ObjectEntry, ObjectKeepAliveRequest, RegisteredObjectData};
use rand::seq::IndexedRandom;
use serde::Serialize;
use thiserror::Error;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio::time;
use tower_http::trace::TraceLayer;
use tracing::{error, info, trace, Level};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};
use uuid::Uuid;

const ENV_VARS_PREFIX: &str = "NIKU_BACKEND_";

const OBJECT_ID_PREFIX_ENV_VAR_NAME: &str = formatcp!("{ENV_VARS_PREFIX}ID_PREFIX");
const DEFAULT_OBJECT_ID_PREFIX: &str = "test";

const SERVE_ADDRESS: &str = "0.0.0.0:4000";

#[cfg(debug_assertions)]
const OBJECT_LIFETIME_SECONDS: u64 = 5;

#[cfg(not(debug_assertions))]
const OBJECT_LIFETIME_SECONDS: u64 = 5 * 60;

const ADJECTIVES: [&str; 200] = [
    "afraid", "all", "angry", "beige", "big", "better", "bitter", "blue", "brave", "breezy",
    "bright", "brown", "bumpy", "busy", "calm", "chatty", "chilly", "chubby", "clean", "clear",
    "clever", "cold", "crazy", "cruel", "cuddly", "curly", "curvy", "cute", "common", "cold",
    "cool", "cyan", "dark", "deep", "dirty", "dry", "dull", "eager", "early", "easy", "eight",
    "eighty", "eleven", "empty", "every", "evil", "fair", "famous", "fast", "fancy", "few", "fine",
    "fifty", "five", "flat", "fluffy", "floppy", "forty", "four", "free", "fresh", "fruity",
    "full", "funny", "fuzzy", "gentle", "giant", "gold", "good", "great", "green", "grumpy",
    "happy", "heavy", "hip", "honest", "hot", "huge", "hungry", "icy", "itchy", "khaki", "kind",
    "large", "late", "lazy", "lemon", "legal", "light", "little", "long", "loose", "loud",
    "lovely", "lucky", "major", "many", "mean", "metal", "mighty", "modern", "moody", "nasty",
    "neat", "new", "nice", "nine", "ninety", "odd", "old", "olive", "open", "orange", "pink",
    "plain", "plenty", "polite", "poor", "pretty", "proud", "public", "puny", "petite", "purple",
    "quick", "quiet", "rare", "real", "ready", "red", "rich", "ripe", "rotten", "rude", "sad",
    "salty", "seven", "shaggy", "shaky", "sharp", "shiny", "short", "shy", "silent", "silly",
    "silver", "six", "sixty", "slick", "slimy", "slow", "small", "smart", "smooth", "social",
    "soft", "solid", "some", "sour", "spicy", "spotty", "stale", "strong", "stupid", "sweet",
    "swift", "tall", "tame", "tangy", "tasty", "ten", "tender", "thick", "thin", "thirty", "three",
    "tidy", "tiny", "tired", "tough", "tricky", "true", "twelve", "twenty", "two", "upset", "vast",
    "violet", "warm", "weak", "wet", "whole", "wicked", "wide", "wild", "wise", "witty", "yellow",
    "young", "yummy",
];

const NOUNS: [&str; 300] = [
    "apes", "animals", "areas", "bars", "banks", "baths", "breads", "bushes", "cloths", "clowns",
    "clubs", "hoops", "loops", "memes", "papers", "parks", "paths", "showers", "sides", "signs",
    "sites", "streets", "teeth", "tires", "webs", "actors", "ads", "adults", "aliens", "ants",
    "apples", "baboons", "badgers", "bags", "bananas", "bats", "beans", "bears", "beds", "beers",
    "bees", "berries", "bikes", "birds", "boats", "bobcats", "books", "bottles", "boxes", "brooms",
    "buckets", "bugs", "buses", "buttons", "camels", "cases", "cameras", "candies", "candles",
    "carpets", "carrots", "carrots", "cars", "cats", "chairs", "chefs", "chicken", "clocks",
    "clouds", "coats", "cobras", "coins", "corners", "colts", "comics", "cooks", "cougars",
    "regions", "results", "cows", "crabs", "crabs", "crews", "cups", "cities", "cycles", "dancers",
    "days", "deer", "dingos", "dodos", "dogs", "dolls", "donkeys", "donuts", "doodles", "doors",
    "dots", "dragons", "drinks", "dryers", "ducks", "ducks", "eagles", "ears", "eels", "eggs",
    "ends", "mammals", "emus", "experts", "eyes", "facts", "falcons", "fans", "feet", "files",
    "flies", "flowers", "forks", "foxes", "friends", "frogs", "games", "garlics", "geckos",
    "geese", "ghosts", "ghosts", "gifts", "glasses", "goats", "grapes", "groups", "guests",
    "hairs", "hands", "hats", "heads", "hornets", "horses", "hotels", "hounds", "houses", "humans",
    "icons", "ideas", "impalas", "insects", "islands", "items", "jars", "jeans", "jobs", "jokes",
    "keys", "kids", "kings", "kiwis", "knives", "lamps", "lands", "laws", "lemons", "lies",
    "lights", "lines", "lions", "lizards", "llamas", "mails", "mangos", "maps", "masks", "meals",
    "melons", "mice", "mirrors", "moments", "moles", "monkeys", "months", "moons", "moose", "mugs",
    "nails", "needles", "news", "nights", "numbers", "olives", "onions", "oranges", "otters",
    "owls", "pandas", "pans", "pants", "papayas", "parents", "parts", "parrots", "paws", "peaches",
    "pears", "peas", "pens", "pets", "phones", "pianos", "pigs", "pillows", "places", "planes",
    "planets", "plants", "plums", "poems", "poets", "points", "pots", "pugs", "pumas", "queens",
    "rabbits", "radios", "rats", "ravens", "readers", "rice", "rings", "rivers", "rockets",
    "rocks", "rooms", "roses", "rules", "schools", "bats", "seals", "seas", "sheep", "shirts",
    "shoes", "shrimps", "singers", "sloths", "snails", "snakes", "socks", "spiders", "spies",
    "spoons", "squids", "stamps", "stars", "states", "steaks", "suits", "suns", "swans", "symbols",
    "tables", "taxes", "taxis", "teams", "terms", "things", "ties", "tigers", "times", "tips",
    "toes", "towns", "tools", "toys", "trains", "trams", "trees", "turkeys", "turtles", "vans",
    "views", "walls", "walls", "wasps", "waves", "ways", "weeks", "windows", "wings", "wolves",
    "wombats", "words", "worlds", "worms", "yaks", "years", "zebras", "zoos",
];

const VERBS: [&str; 250] = [
    "accept", "act", "add", "admire", "agree", "allow", "appear", "argue", "arrive", "ask",
    "attack", "attend", "bake", "bathe", "battle", "beam", "beg", "begin", "behave", "bet", "boil",
    "bow", "brake", "brush", "build", "burn", "buy", "call", "camp", "care", "carry", "change",
    "cheat", "check", "cheer", "chew", "clap", "clean", "cough", "count", "cover", "crash",
    "create", "cross", "cry", "cut", "dance", "decide", "deny", "design", "dig", "divide", "do",
    "double", "doubt", "draw", "dream", "dress", "drive", "drop", "drum", "eat", "end", "enter",
    "enjoy", "exist", "fail", "fall", "feel", "fetch", "film", "find", "fix", "flash", "float",
    "flow", "fly", "fold", "follow", "fry", "give", "glow", "go", "grab", "greet", "grin", "grow",
    "guess", "hammer", "hang", "happen", "heal", "hear", "help", "hide", "hope", "hug", "hunt",
    "invent", "invite", "itch", "jam", "jog", "join", "joke", "judge", "juggle", "jump", "kick",
    "kiss", "kneel", "knock", "know", "laugh", "lay", "lead", "learn", "leave", "lick", "like",
    "lie", "listen", "live", "look", "lose", "love", "make", "march", "marry", "mate", "matter",
    "melt", "mix", "move", "nail", "notice", "obey", "occur", "open", "own", "pay", "peel", "play",
    "poke", "post", "press", "prove", "pull", "pump", "pick", "punch", "push", "raise", "read",
    "refuse", "relate", "relax", "remain", "repair", "repeat", "reply", "report", "rescue", "rest",
    "retire", "return", "rhyme", "ring", "roll", "rule", "run", "rush", "say", "scream", "see",
    "search", "sell", "send", "serve", "shake", "share", "shave", "shine", "show", "shop", "shout",
    "sin", "sink", "sing", "sip", "sit", "sleep", "slide", "smash", "smell", "smile", "smoke",
    "sneeze", "sniff", "sort", "speak", "spend", "stand", "start", "stay", "stick", "stop",
    "stare", "study", "strive", "swim", "switch", "take", "talk", "tan", "tap", "taste", "teach",
    "tease", "tell", "thank", "think", "throw", "tickle", "tie", "trade", "train", "travel", "try",
    "turn", "type", "unite", "vanish", "visit", "wait", "walk", "warn", "wash", "watch", "wave",
    "wear", "win", "wink", "wish", "wonder", "work", "worry", "write", "yawn", "yell",
];

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        // Config the logging with env vars and set the default level to "info"
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{}=info", env!("CARGO_CRATE_NAME")).into()),
        )
        .with_target(false)
        .compact()
        .init();

    match run().await {
        Ok(_) => (),
        Err(err) => error!("{err}"),
    }
}

#[derive(Debug)]
struct KeepAliveEntry {
    ticket_id: String,
    delete_task: JoinHandle<()>,
}

struct SharedData {
    objects: HashMap<String, ObjectEntry>,
    keep_alive_entries: HashMap<String, KeepAliveEntry>,
}

#[derive(Error, Debug)]
enum RunError {
    #[error("Binding to the TCP listening port failed: {0}")]
    BingTcpListenerFailed(#[source] io::Error),

    #[error("Unable to start serving the server with Axum: {0}")]
    ServeFailed(#[source] io::Error),

    #[error("The environment variable {0} is not formatted as Unicode")]
    EnvVarNotUnicode(String),
}

enum ServerError {
    UnknownObject,
    UnknownKeepAliveKey,
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        // How we want errors responses to be serialized
        #[derive(Serialize)]
        struct ErrorResponse {
            code: String,
            message: String,
        }

        let (status, code, message) = match self {
            ServerError::UnknownObject => (
                StatusCode::NOT_FOUND,
                String::from("NKBE:1"),
                String::from("The requested object is not available"),
            ),

            ServerError::UnknownKeepAliveKey => (
                StatusCode::NOT_FOUND,
                String::from("NKBE:2"),
                String::from("The given keep alive key doesn't match for any registered object"),
            ),
        };

        (status, Json(ErrorResponse { code, message })).into_response()
    }
}

async fn run() -> Result<(), RunError> {
    let object_id_prefix =
        env::var(OBJECT_ID_PREFIX_ENV_VAR_NAME).unwrap_or(String::from(DEFAULT_OBJECT_ID_PREFIX));

    info!("Starting NIKU backend server...");
    info!("Object lifetime: {OBJECT_LIFETIME_SECONDS}s");
    info!("Object ID prefix: {object_id_prefix}");
    info!("Serving at http://{SERVE_ADDRESS}/");

    let state = Arc::new(Mutex::new(SharedData {
        objects: HashMap::new(),
        keep_alive_entries: HashMap::new(),
    }));

    let app = Router::new()
        .route("/objects", put(put_objects))
        .route("/objects/{id}", get(get_objects_id))
        .route("/objects/{id}/keep-alive", post(post_objects_id_keep_alive))
        .with_state(state)
        .layer(TraceLayer::new_for_http().make_span_with(|req: &Request| {
            let method = req.method();
            let uri = req.uri();

            let matched_path = req
                .extensions()
                .get::<MatchedPath>()
                .map(|matched_path| matched_path.as_str());

            tracing::debug_span!("request", %method, %uri, matched_path)
        }));

    let listener = tokio::net::TcpListener::bind(SERVE_ADDRESS)
        .await
        .map_err(RunError::BingTcpListenerFailed)?;

    axum::serve(listener, app)
        .await
        .map_err(RunError::ServeFailed)?;

    Ok(())
}

fn create_object_delete_task(
    locked_state: Arc<Mutex<SharedData>>,
    id: &str,
    keep_alive_key: &str,
) -> JoinHandle<()> {
    // Only on debug mode for privacy reasons
    if cfg!(debug_assertions) {
        trace!(%id, %keep_alive_key, "Creating an object scheduled delete task");
    }

    let locked_state = locked_state.clone();
    let object_id = String::from(id);
    let object_keep_alive_key = String::from(keep_alive_key);

    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(5));
        // The first tick is immediate, skip it
        interval.tick().await;
        interval.tick().await;

        let mut state = locked_state.lock().await;

        // Only on debug mode for privacy reasons
        if cfg!(debug_assertions) {
            info!(
                keep_alive_key = object_keep_alive_key,
                "Object '{object_id}' timed out! Deleting it..."
            );
        }

        state.objects.remove(&object_id);
        state.keep_alive_entries.remove(&object_keep_alive_key);
    })
}

/// Get a random value from a `&[&str]`
///
/// # Safety
/// The given slice must not be empty.
unsafe fn get_random_value_from_str(values: &[&str]) -> String {
    #[allow(clippy::expect_used)]
    values
        .choose(&mut rand::rng())
        .expect("The vector should never be empty")
        .to_string()
}

fn get_random_word() -> String {
    unsafe {
        let adjective = get_random_value_from_str(&ADJECTIVES);
        let noun = get_random_value_from_str(&NOUNS);
        let verb = get_random_value_from_str(&VERBS);

        format!("{adjective} {noun} {verb}")
    }
}

async fn put_objects(
    State(locked_state): State<Arc<Mutex<SharedData>>>,
    Json(upload_ticket): Json<ObjectEntry>,
) -> Json<RegisteredObjectData> {
    let state = &mut locked_state.lock().await;

    // Iterate over until a unique ID is found, given the number of combinations
    // this should not happen more than one or two times at most
    let id = loop {
        let new_id = get_random_word();

        if !state.objects.contains_key(&new_id) {
            break new_id;
        }
    };

    let keep_alive_key = Uuid::new_v4().to_string();

    state.objects.insert(id.clone(), upload_ticket);

    state.keep_alive_entries.insert(
        keep_alive_key.clone(),
        KeepAliveEntry {
            ticket_id: id.clone(),
            delete_task: create_object_delete_task(locked_state.clone(), &id, &keep_alive_key),
        },
    );

    if cfg!(debug_assertions) {
        info!(%id, %keep_alive_key, "Created new object");
    }

    Json(RegisteredObjectData { id, keep_alive_key })
}

async fn get_objects_id(
    State(state): State<Arc<Mutex<SharedData>>>,
    Path(id): Path<String>,
) -> Result<Json<ObjectEntry>, ServerError> {
    let objects = &mut state.lock().await.objects;
    let entry = objects.get(&id).ok_or(ServerError::UnknownObject)?.clone();

    if cfg!(debug_assertions) {
        info!(?entry, "Requested object entry");
    }

    Ok(Json(entry))
}

async fn post_objects_id_keep_alive(
    State(locked_state): State<Arc<Mutex<SharedData>>>,
    Path(id): Path<String>,
    Json(keep_alive_request): Json<ObjectKeepAliveRequest>,
) -> Result<(), ServerError> {
    let mut state = locked_state.lock().await;

    let keep_alive_entry = state
        .keep_alive_entries
        .get(&keep_alive_request.keep_alive_key)
        .ok_or(ServerError::UnknownKeepAliveKey)?;

    keep_alive_entry.delete_task.abort();
    let ticket_id = keep_alive_entry.ticket_id.clone();

    // Drop the reference
    let _ = keep_alive_entry;

    let delete_task = create_object_delete_task(
        locked_state.clone(),
        &id,
        &keep_alive_request.keep_alive_key,
    );

    state.keep_alive_entries.insert(
        keep_alive_request.keep_alive_key,
        KeepAliveEntry {
            ticket_id,
            delete_task,
        },
    );

    Ok(())
}
