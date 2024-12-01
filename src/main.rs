use std::io;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::io::Read;
use std::array;
use std::path::Path;
use regex::Regex;
use csv::Reader;
use std::thread;
use std::time::Duration;
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
        oxygen: 240,
        player_direction: Direction::North,
        player_position: (11, 42, 1),
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

    println!("\nA muitas décadas atrás, o Barão Matthew Sheldrake afundou junto de seu návio nessas águas");
    println!("Reza a lenda, que ele levava consigo toda sua fortuna dentro de um cofre dourado...");
    println!("Com o seu submarino '{}', cabe a você encontrar este tesouro!", &cur_game.sub_name);

    game_loop(cur_game, false);
    Ok(())
}

fn update_save_file(game: &Game) -> std::io::Result<()>{
    let save_files = get_save_files()?;

    let save_name = format!("{}-{}.json", game.id, game.sub_name);
    let filepath = format!("saves/{}", save_name);

    let mut file = File::create(filepath)?;
    let data = serde_json::to_string(&game)?;
    file.write_all(data.as_bytes())?;

    println!("Jogo salvo com sucesso!");
    Ok(())
}

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

fn load_demo_csv() -> Result<Vec<String>, Box<dyn std::error::Error>>{
    let mut demo_csv = Reader::from_path("assets/demo.csv")?;
    let csv_rec = demo_csv.records().next().ok_or("Erro ao carregar demo.csv")??;
    let demo_inputs: Vec<String> = csv_rec.get(0).unwrap_or("").split(';').map(|s| s.to_string()).collect();

    Ok(demo_inputs)
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
                        game_loop(game, false);
                    },
                    Err(e) =>{
                        println!("Erro ao carregar o jogo: {}", e);
                    },
                }
                break
            },
            "3" =>{
                let mut player_map =  vec![vec![vec!["".to_string(); 50]; 50];3];
                player_map[1][42][10] = "player".to_string();

                let demo_game = Game{
                    id: get_save_files().expect("Erro ao carregar arquivos salvos.").len() as u8 + 1,
                    sub_name: "Demo".to_string(),
                    last_save: Utc::now(),
                    oxygen: 240,
                    player_direction: Direction::North,
                    player_position: (11, 42, 1),
                    real_map: load_map_csv().map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("{:?}", e))).expect("Falha ao carregar mapa"),
                    //player_map: vec![vec![vec!["".to_string(); 50]; 50];3];
                    player_map: player_map,
                };

                game_loop(demo_game, true);
            },
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

fn move_sub(game: &mut Game,dir: Direction){
    let (cur_x, cur_y, cur_z) = game.player_position;
    let (mut next_x, mut next_y, mut next_z) = (cur_x, cur_y, cur_z);

    print_movement(&dir);

    match dir {
        Direction::North => {
            game.player_direction = dir;
            if next_y >= 1{
                next_y -= 1;
            }else{
                println!("Área fora dos parâmetros dá missão! retornando...");
                return;
            }
        },
        Direction::South => {
            next_y += 1;
            game.player_direction = dir;
    },
        Direction::East => {
            next_x += 1;
            game.player_direction = dir;
    },
        Direction::West => {
            game.player_direction = dir;
            if next_x >= 1{
                next_x -= 1;
            }else{
                println!("Área fora dos parâmetros dá missão! retornando...");
                return;
            }
    },
        Direction::Up => {
            if next_z >=1{
                next_z -= 1;
            }else{
                println!("Altitude máxima já alcançada! Impossível ascender mais...");
                return;
            }
        },
        Direction::Down => next_z += 1,
    }

    if next_x < 0 || next_x > 49 || next_y < 0 || next_y > 49{
        println!("Área fora dos parâmetros dá missão! retornando...");
        return;
    }else if next_z > 2{
        println!("Profundidade máxima já alcançada! Impossível descer mais...");
        return;
    } else if game.real_map[next_z as usize][next_y as usize][next_x as usize] == "rock" || game.real_map[next_z as usize][next_y as usize][next_x as usize] == "borderRock"{
        println!("O submarino se bate em uma rocha! Ouch!");
    } else if game.real_map[next_z as usize][next_y as usize][next_x as usize] == "treasure"{
        println!("O submarino se bate contra algo precioso! Tesouro detectado nas redondezas!");
    } else {
        game.real_map[cur_z as usize][cur_y as usize][cur_x as usize] = "n/a".to_string();
        game.real_map[next_z as usize][next_y as usize][next_x as usize] = "player".to_string();
        game.player_map[cur_z as usize][cur_y as usize][cur_x as usize] = "n/a".to_string();
        game.player_map[next_z as usize][next_y as usize][next_x as usize] = "player".to_string();
        game.player_position = (next_x as u8, next_y as u8, next_z as u8);

    }
}

fn shoot_missile(game: &mut Game){
    let (cur_x , cur_y, cur_z) = game.player_position;
    let (mut x_loc, mut y_loc) = (0,0);

    println!("\nO submarino dispara um torpedo a frente!");

    for it in 1..11{
        match game.player_direction {
            Direction::North | Direction::Up | Direction::Down => y_loc = -it  as i8,
            Direction::South => y_loc = it,
            Direction::East => x_loc = it,
            Direction::West => x_loc = -it  as i8,
        }

        if game.real_map[cur_z as usize][(cur_y as i8 + y_loc as i8) as usize][(cur_x as i8 + x_loc as i8) as usize] == "n/a".to_string(){
            continue;
        }else if game.real_map[cur_z as usize][(cur_y as i8 + y_loc as i8) as usize][(cur_x as i8 + x_loc as i8) as usize] == "borderRock".to_string(){
            println!("O torpedo se bate em uma rocha robusta! Nenhum dano parece ter ocorrido...");
            return;
        }else if game.real_map[cur_z as usize][(cur_y as i8 + y_loc as i8) as usize][(cur_x as i8 + x_loc as i8) as usize] == "rock".to_string(){
            println!("O torpedo atinge uma rocha e a destrói!");
            game.real_map[cur_z as usize][(cur_y as i8 + y_loc as i8) as usize][(cur_x as i8 + x_loc as i8) as usize] = "n/a".to_string();
            return;
        }
    }
    println!("Você não ouve o som do torpedo, deve ter viajado muito longe...");
}


fn run_scan(game: &mut Game){
    let (player_x, player_y, player_z) = game.player_position;
    println!("\nEscaneando arredores...");
    for y in -6i8..7{
        for x in -9i8..10{
            let cur_tile_x = player_x as i8 + x;
            let cur_tile_y = player_y as i8 + y;
            if !(cur_tile_x < 0 || cur_tile_y < 0 || cur_tile_x > 49 || cur_tile_y > 49) {
                game.player_map[player_z as usize][cur_tile_y as usize][cur_tile_x as usize] = game.real_map[player_z as usize][cur_tile_y as usize][cur_tile_x as usize].clone();
            }
        }
    }
}

fn capture_item(game: &Game) -> bool {
    let (player_x, player_y, player_z) = game.player_position;
    println!("O submarino tenta capturar algo a sua frente...");

    match game.player_direction {
        Direction::North | Direction::Up | Direction::Down => {
            if game.real_map[player_z as usize][(player_y - 1) as usize][player_x as usize] == "treasure"{
                return true;
            }
        },
        Direction::South => {
            if game.real_map[player_z as usize][(player_y + 1) as usize][player_x as usize] == "treasure"{
                return true;
            }
        },
        Direction::East => {
            if game.real_map[player_z as usize][player_y as usize][(player_x + 1) as usize] == "treasure"{
                return true;
            }
        },
        Direction::West => {
            if game.real_map[player_z as usize][player_y as usize][(player_x - 1) as usize] == "treasure"{
                return true;
            }
        },
    }

    println!("Nada interessante encontrado...");
    return false;
}

fn print_help(){
    println!("Lista de comandos:");
    println!("Move [] - Move o návio na direção específicada\nOpções: North: Move o submarino na direção norte\n------> South: Move o submarino na direção sul\n------> East: Move o submarino na direção leste\n------> West: Move o submarino na direção oeste\n------> Up: Sobe o submarino 1 nível\n------> Down: Desce o submarino 1 nível");
    println!("Scan - Detecta obstáculos e items ao redor do návio");
    println!("Shoot - Dispara um torpedo a frente do návio, útil para se livrar de rochas");
    println!("Capture - Pega um item a frente do návio, use para obter o tesouro!");
    println!("Save - Salva o progresso do seu jogo");
    println!("Help - Você já sabe o que isso faz!");
    println!("Quit - Termina a execução do jogo");
    println!("OBS: Os comandos aqui expostos NÃO são case-sensitive");
}


fn game_loop(mut game: Game, is_demo : bool){
    println!("\nJogo começado! Digite 'Help' para saber como dirigir o submarino '{}'", &game.sub_name);
    
    let mut demo_steps = 0;

    loop {
        game_hud(&game);

        if game.oxygen <= 0{
            println!("Seu oxigênio acaba!");
            println!("O submarino rapidamente ascende para a superfície, e uma equipe de resgate lhe ajuda a sair");
            println!("O cofre nunca foi encontrado...");
            println!("FIM DE JOGO");
            break;
        }

        let mut input = "".to_string();
        if !is_demo{
            input = get_player_input();
        }else{
            thread::sleep(Duration::from_millis(500));
            let demo_inputs = load_demo_csv().expect("REASON");
            if demo_steps+1 > demo_inputs.len(){
                println!("Simulação falha, fechando o jogo...");
                break;
            }
            input = demo_inputs[demo_steps].clone();
            println!("{}",input);
            demo_steps += 1;
        }
        match match_player_input(&input) {
            Some(Action::Move(dir)) => {
                move_sub(&mut game, dir);
            }
            Some(Action::Scan) => {
                run_scan(&mut game);
            }
            Some(Action::Shoot) =>{
                shoot_missile(&mut game);
            }
            Some(Action::Capture) =>{
                if capture_item(&game){
                    println!("Sucesso! Você obteve o 'Cofre de Matthew Sheldrake'!");
                    println!("Após algum tempo, o submarino retorna a superfície...");
                    println!("Com sua nova fortuna, você vive uma vida próspera e luxuosa!");
                    println!("FIM DE JOGO");
                    break;
                }
            }
            Some(Action::Quit) =>{
                println!("Encerrando o jogo...");
                break;
            }
            Some(Action::Save) =>{
                game.last_save = Utc::now();
                update_save_file(&game);
            }
            Some(Action::Help) =>{
                print_help();
            }
            None => {
                println!("Comando inválido ou ainda não implementado");
            }
        }

        game.oxygen -= 1;
    }
}

fn main() {
    
    if !Path::new("saves").exists() {
        fs::create_dir("saves");
    }

    title_screen();

}
