use std::sync::atomic::{AtomicBool, Ordering::Relaxed};
use crate::tetra::{
    Context, 
    Result,
};
use storage::load;
use pokedex::serialize::{SerializedDex, SerializedPokemon};
use crate::audio::{
    sound::{Sound, add_sound},
    serialized::SerializedSoundData,
};
use crate::config::{Configuration, CONFIGURATION};

pub use firecore_text::init as text;

pub static LOADING_FINISHED: AtomicBool = AtomicBool::new(false);

pub fn seed_random(seed: u64) {
    deps::random::seed_global(seed);
}

pub fn logger() {

    use simple_logger::SimpleLogger;
    use deps::log::LevelFilter;

    // Initialize logger

    let logger = SimpleLogger::new();

    #[cfg(debug_assertions)]
    let logger = logger.with_level(LevelFilter::Debug);
    #[cfg(not(debug_assertions))]
    let logger = logger.with_level(LevelFilter::Info);

    logger.init().unwrap_or_else(|err| panic!("Could not initialize logger with error {}", err));

}

pub fn configuration() -> Result {
    let config = load::<Configuration>();
    // store::<PlayerSaves>().await;

    {

        crate::input::keyboard::load(config.controls.clone());
        crate::input::controller::load(crate::input::controller::default_button_map());

        // if config.touchscreen {
        //     crate::input::touchscreen::touchscreen(true);
        // }

    }

    unsafe { CONFIGURATION = Some(config) };

    Ok(())

}

pub fn pokedex(ctx: &mut Context, dex: SerializedDex) -> Result {
    let callback = |pokemon: &mut SerializedPokemon| {
        if !pokemon.cry_ogg.is_empty() {
            if let Err(_) = add_sound(SerializedSoundData {
                bytes: std::mem::take(&mut pokemon.cry_ogg),
                sound: Sound::variant(crate::CRY_ID, Some(pokemon.pokemon.id)),
            }) {
                // warn!("Error adding pokemon cry: {}", err);
            }
        }
    };
    pokedex::init(ctx, dex, #[cfg(feature = "audio")] callback)
}


#[cfg(feature = "audio")]
pub fn audio(audio: crate::audio::serialized::SerializedAudio) {
    use crate::log::error;    

    if let Err(err) = crate::audio::create() {
        error!("{}", err);
    } else {
        std::thread::spawn( || {
            if let Err(err) = crate::audio::load(audio) {
                error!("Could not load audio files with error {}", err);
            }
        });
    }    
}

pub fn finished_loading() {
    LOADING_FINISHED.store(true, Relaxed);
}