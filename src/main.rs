use std::io;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::io::Read;
use std::path::Path;
use regex::Regex;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

#[derive(Deserialize, Serialize)]
struct Game{
    id: u8,
    sub_name: String,
    last_save: DateTime<Utc>,
    oxygen: u8,
    health: u8,
    player_direction: Direction,
    player_position: (u8, u8, u8),
}

enum Action{
    Move(Direction),
    Scan,
    Shoot,
    Capture,
    Save,
    Help,
    Quit,
}


#[derive(Deserialize, Serialize)]
enum Direction {
    North,
    South,
    East,
    West,
    Up,
    Down,
}

fn print_movement(dir: &Direction) -> std::io::Result<()>{
    match dir {
        Direction::North => println!("O submarino se move na direção norte"),
        Direction::South => println!("O submarino se move na direção sul"),
        Direction::East => println!("O submarino se move na direção leste"),
        Direction::West => println!("O submarino se move na direção oeste"),
        Direction::Up => println!("O submarino ascende"),
        Direction::Down => println!("O submarino desce"),
    }

    Ok(())
}

fn create_save_file(game: &Game, game_file: &str) -> std::io::Result<()> {

    let save_name = format!("{}-{}.json",game.id, game.sub_name);

    let filepath = format!("saves/{}", save_name);

    let mut file = File::create(filepath)?;
    let data = serde_json::to_string(game)?;
    file.write_all(data.as_bytes())?;

    println!("Jogo salvo como: {}", save_name);
    Ok(())
}
//fn  update_save_file(game: &Game)

fn get_save_files() -> std::io::Result<Vec<String>> {
    let entries = fs::read_dir("saves")?;

    let reg = Regex::new(r"^\d+-").unwrap();

    let mut match_files = Vec::new();

    for en in entries {
        let en = en?;
        let path = en.path();

        if path.is_file(){
            if let Some(filename) = path.file_name(){
                if let Some(filename_str) = filename.to_str(){
                    if reg.is_match(filename_str){
                        match_files.push(filename_str.to_string());
                    }
                }
            }
        }
    }

    //if match_files.is_empty(){
    //    Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Nenhum jogo salvo encontrado"))
    Ok(match_files)
    
}

fn load_save_menu(save_id: &str) ->std::io::Result<Game>{
    let save_files = get_save_files()?;

    for sf in &save_files {
        println!("{}", sf);
    }

    println!("Entre o nome do jogo que deseja carregar:");

    let input = get_player_input();
    let mut game = load_save_file(&input);

    match load_save_file(&input){
        Ok(game) => Ok(game),
        Err(e) => Err(e),
    }
}

fn load_save_file(save_name: &str) -> std::io::Result<Game> {

    let filepath = format!("saves/{}.json", save_name);

    let mut file = File::open(&filepath)?;

    let mut data = String::new();
    file.read_to_string(&mut data)?;

    let loaded_game: Game = serde_json::from_str(&data)?;
    
    println!("Jogo {} carregado com sucesso!", save_name);
    
    Ok(loaded_game)
}

fn match_player_input(input: &str) -> Option<Action> {
    match input.trim().to_lowercase().as_str() {
        "move north" => Some(Action::Move(Direction::North)),
        "move south" => Some(Action::Move(Direction::South)),
        "move east" => Some(Action::Move(Direction::East)),
        "move west" => Some(Action::Move(Direction::West)),
        "move up" => Some(Action::Move(Direction::Up)),
        "move down" => Some(Action::Move(Direction::Down)),
        "scan" => Some(Action::Scan),
        "shoot" => Some(Action::Shoot),
        "capture" => Some(Action::Capture),
        "save" => Some(Action::Save),
        "help" => Some(Action::Help),
        "quit" => Some(Action::Quit),
        _ => None,
    }
}

fn get_player_input() -> String {
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Falha ao ler entrada");
    input
}

fn main() {
    println!("Insira o nome de seu submarino:");
    let sub_name = get_player_input();

    if !Path::new("saves").exists() {
        fs::create_dir("saves");
    }

    let mut cur_game = Game{
        id: get_save_files().expect("Erro ao carregar arquivos salvos.").len() as u8 + 1,
        sub_name: String::from(sub_name),
        last_save: Utc::now(),
        oxygen: 200,
        health: 100,
        player_direction: Direction::North,
        player_position: (0, 0, 0),
    };

    println!("Jogo começado, por favor insira um comando:");

    loop {
        let input = get_player_input();
        match match_player_input(&input) {
            Some(Action::Move(dir)) => {
                print_movement(&dir);
                cur_game.player_direction = dir;

            }
            Some(Action::Scan) => {
                println!("O submarino escaneia seus arredores...");
            }
            Some(Action::Shoot) =>{
                println!("O submarino dispara um míssel!");
            }
            Some(Action::Capture) =>{
                println!("O submarino tenta pegar um item próximo de si...");
                println!("Mas nada ocorre!");
            }
            Some(Action::Quit) =>{
                println!("Encerrando o jogo...");
                break;
            }
            Some(Action::Save) =>{
                println!("Encerrando o jogo...");
                break;
            }
            Some(Action::Help) =>{
                println!("Encerrando o jogo...");
                break;
            }
            None => {
                println!("Comando inválido ou ainda não implementado");
            }
        }

    }
}
