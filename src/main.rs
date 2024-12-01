use std::io;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::io::Read;
use std::array;
use std::path::Path;
use regex::Regex;
use csv::Reader;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

#[derive(Deserialize, Serialize)]
struct Game{
    id: u8,
    sub_name: String,
    last_save: DateTime<Utc>,
    oxygen: u8,
    player_direction: Direction,
    player_position: (u8, u8, u8),
    real_map: Vec<Vec<Vec<String>>>,
    player_map: Vec<Vec<Vec<String>>>,
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

fn create_save_file() -> std::io::Result<()> {
    let title_pattern = Regex::new(r"^[A-Za-z]{1,12}$").unwrap();
    let mut sub_name = String::new();
    let mut player_map =  vec![vec![vec!["".to_string(); 50]; 50];3];
    player_map[1][42][10] = "player".to_string();

    loop {
        println!("Insira o nome de seu submarino (max 12 chars):");
        sub_name = get_player_input();

        if title_pattern.is_match(&sub_name){
            break
        }else{
            println!("Título inválido")
        }
    }

    let mut cur_game = Game{
        id: get_save_files().expect("Erro ao carregar arquivos salvos.").len() as u8 + 1,
        sub_name: String::from(sub_name),
        last_save: Utc::now(),
        oxygen: 200,
        player_direction: Direction::North,
        player_position: (10, 42, 1),
        real_map: load_map_csv().map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("{:?}", e)))?,
        //player_map: vec![vec![vec!["".to_string(); 50]; 50];3];
        player_map: player_map,
    };

    let save_name = format!("{}-{}.json", cur_game.id, cur_game.sub_name);

    let filepath = format!("saves/{}", save_name);

    let mut file = File::create(filepath)?;
    let data = serde_json::to_string(&cur_game)?;
    file.write_all(data.as_bytes())?;

    println!("Jogo salvo como: {}", save_name);

    game_loop(cur_game);
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

    Ok(match_files)
    
}

fn load_save_menu() ->std::io::Result<Game>{
    let save_files = get_save_files()?;
    let mut input = String::new();

    for sf in &save_files {
        println!("{}", sf);
    }

    loop{
        println!("\nEntre o nome do jogo que deseja carregar:");

        let input = get_player_input();
        for sf in &save_files{
            if sf.contains(&input) {
                let mut game = load_save_file(&sf)?;
                return Ok(game);
            }
        }
        println!("Entrada inválida");
    }

}

fn delete_save_menu() -> Result<(), Box<dyn std::error::Error>>{
    let save_files = get_save_files()?;
    let mut input = String::new();
    let mut confirmation_input = String::new();

    for sf in &save_files {
        println!("{}", sf);
    }

    loop{
        println!("\nEntre o nome do jogo que deseja deletar:");

        input = get_player_input();
        for sf in &save_files{
            if sf.contains(&input) {
                loop{
                    println!("Jogo {} encontrado, desejar excluir o jogo salvo? Essa ação não pode ser desfeita. (s/n)", &sf);
                    confirmation_input = get_player_input();
                    match confirmation_input.as_str(){
                        "s" => {
                            delete_save_file(sf);
                            return Ok(());
                        },
                        "n" => return Ok(()),
                        _ => println!("Confirmação inválida"),
                    }

                }
            }
        }
        println!("Entrada inválida");
    }

}


fn load_save_file(save_name: &str) -> std::io::Result<Game> {

    let filepath = format!("saves/{}", save_name);

    let mut file = File::open(&filepath)?;

    let mut data = String::new();
    file.read_to_string(&mut data)?;

    let loaded_game: Game = serde_json::from_str(&data)?;
    
    println!("Jogo {} carregado com sucesso!", save_name);
    
    Ok(loaded_game)
}

fn delete_save_file(save_name: &str){
    let filepath = format!("saves/{}", save_name);

    fs::remove_file(&filepath);
    
    println!("Jogo {} deletado com sucesso", save_name);
}

fn load_map_csv() -> Result<Vec<Vec<Vec<String>>>, Box<dyn std::error::Error>> {
    let mut base_map = Reader::from_path("assets/base_map.csv")?;
    let mut local_map: Vec<Vec<Vec<String>>> = vec![vec![vec!["".to_string(); 50]; 50];3];

    let mut h_index = 0;

    for (x_index, x_value) in base_map.records().enumerate() {
        let row = x_value?;
        h_index = x_index/50;

        let items: Vec<String> = row.get(0).unwrap_or("").split(';')
            .map(|s| s.to_string())
            .collect();

        for (y_index, y_value) in items.iter().enumerate() {
            local_map[h_index][x_index%50][y_index] = y_value.to_string();
        }
    }
    Ok(local_map)
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
    input.trim().to_string()
}

fn title_screen(){
    loop{
        println!("\n1. Novo jogo \n2. Carregar jogo salvo\n3. Demo Game\n4. Deletar um jogo salvo\n5. Sair");
        let input = get_player_input();
        match input.as_str() {
            "1" =>{
                create_save_file();
                break
            },
            "2" => {
                match load_save_menu(){
                    Ok(game) =>{
                        game_loop(game);
                    },
                    Err(e) =>{
                        println!("Erro ao carregar o jogo: {}", e);
                    },
                }
                break
            },
            "3" =>println!(""),
            "4" =>{
                delete_save_menu();
            },
            "5" =>break,
            _ =>println!("Opção inválida"),
        }
    }

}

fn game_hud(game: &Game){
    let (player_x, player_y, player_z) = game.player_position;
    println!("\nProfundidade:{}", match player_z{
        0 => "Águas razas",
        1 => "Águas profundas",
        2 => "Abismal",
        _ => "Error",
        }
    );
    println!("/-------------------\\") ;
    for y in -6i8..7{
        print!("|");
        for x in -9i8..10{
            let cur_tile_x = player_x as i8 + x;
            let cur_tile_y = player_y as i8 + y;
            if cur_tile_x < 0 || cur_tile_y < 0 || cur_tile_x > 49 || cur_tile_y > 49 {
                print!("#");
            }else{
                match game.player_map[player_z as usize][cur_tile_y as usize][cur_tile_x as usize].as_str(){
                    "borderRock" | "rock" => print!("#"),
                    "n/a" => print!(" "),
                    "treasure" => print!("*"),
                    "player" => {
                        let direction = &game.player_direction;
                        match direction{
                            Direction::North => print!("^"),
                            Direction::South => print!("v"),
                            Direction::East => print!(">"),
                            Direction::West => print!("<"),
                            _ => print!("^"),
                        }
                    },
                    _ => print!(" "),
                }
            }
        }
        println!("|");
    }
    println!("\\-------------------/") ;

    println!("Oxigênio:{}", game.oxygen);
}


fn game_loop(mut game: Game){

    let mut cur_game: &mut Game = &mut game;
    println!("Jogo começado! Digite 'Help' para saber como dirigir o submarino '{}'", &cur_game.sub_name);
    
    loop {
        game_hud(&cur_game);

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
                println!("Nome do submarino:{}", &cur_game.sub_name);
                match load_map_csv(){
                    Ok(map) =>{
                        for h in 0..3{
                            for x in 0..50{
                                for y in 0..50{
                                    match map[h][x][y].as_str(){
                                        "rock" => print!("#"),
                                        "borderRock" => print!("/"),
                                        "treasure" => print!("*"),
                                        "n/a" => print!(" "),
                                        _ => print!("P"),
                                    }
                                }
                                print!("\n");
                            }
                            println!("*");
                        }
                    },
                    Err(e) =>{
                        println!("Erro : {}", e)
                    }
                }
            }
            None => {
                println!("Comando inválido ou ainda não implementado");
            }
        }

        cur_game.oxygen -= 1;
    }
}

fn main() {
    
    if !Path::new("saves").exists() {
        fs::create_dir("saves");
    }

    title_screen();

}
