
mod parse;
mod types;
mod hashmap;

use crate::parse::*;
use crate::types::{Run,Realm, MergedRuns, Outcome};
use crate::hashmap::*;
use crate::hashmap::HashMapExtensions;

use clap::{crate_authors, crate_version};

use std::fs;
use std::path::*;
use std::io;
use std::io::BufRead;
use std::collections::HashMap;
use std::collections::HashSet;
use std::cmp;

use clap::{Arg, App};

fn main() {
    let matches = App::new("Rift Wizard Stats")
                          .version(crate_version!())
                          .author(crate_authors!())
                          .about("Turns Rift Wizard logs into stats")
                          .arg(
                                Arg::with_name("SAVELOCATION")
                                    .help("Location of Rift Wizard's saves folder")
                                    // TODO - check registry for folder and so on?
                                    .default_value(r#"C:\Program Files (x86)\Steam\steamapps\common\Rift Wizard\RiftWizard\saves\"#)
                                    .index(1)
                                    .multiple(true),
                            )
                          .get_matches();
    
    let save_location_list : Vec<_> = matches.values_of("SAVELOCATION").unwrap().collect();
    
    let all_runs =
    {
        let mut all_runs = None;
        for save_location in save_location_list {
            all_runs = {
                let rift_wiz_save_folder = PathBuf::from(save_location);
                
                let result = read_all_saves(rift_wiz_save_folder.into_boxed_path(), all_runs);
                
                match result {
                    // TODO - add proper error handling
                    Err(err) => {println!("\nerror: {:?}", err); panic!("TODO add proper error handling"); },
                    Ok(r) => Some(r)
                }
            };
        }
        
        all_runs.unwrap()
    };
    
    print_merged_runs_info(all_runs);
}

#[derive(Debug)]
enum ReadSaveError
{
    Io(io::Error),
    Gather(GatherError),
    IllegalFilename,
    InvalidDirectory(String)
}

fn read_all_saves(save_folder : Box<Path>, merged_runs : Option<MergedRuns>) -> Result<MergedRuns, ReadSaveError>
{
    if !save_folder.is_dir() {
        return Err(ReadSaveError::InvalidDirectory(
            save_folder.to_str().unwrap_or("failed to stringify Save Folder").to_string()
        ));
    }
    
    let mut merged_runs = match merged_runs {
        Some(r) => r,
        None => MergedRuns { ..Default::default() },
    };
    
    for entry in fs::read_dir(save_folder).map_err(ReadSaveError::Io)? {
        let path = entry.map_err(ReadSaveError::Io)?.path();
        
        if path.is_dir() {
            let result = read_save(path.into_boxed_path());
            
            match result {
                Err(err) => println!("\nrealm read error: {:?}", err),
                Ok(r) => { merged_runs = merged_runs.merge_run(generate_run_report(r)); } 
            }
        }
    }
    
    Ok(merged_runs)
}

fn read_save(save_folder : Box<Path>) -> Result<Vec<Realm>, ReadSaveError>
{
    if !save_folder.is_dir() {
        return Err(ReadSaveError::InvalidDirectory(
            save_folder.to_str().unwrap_or("failed to stringify Save Folder").to_string()
        ));
    }
    
    let mut realms = Vec::with_capacity(1);
    
    for entry in fs::read_dir(save_folder).map_err(ReadSaveError::Io)? {
        let path = entry.map_err(ReadSaveError::Io)?.path();
        
        if path.is_file() {
            match path.file_name() {
                Some(filename) => {
                    if filename.to_str().ok_or(ReadSaveError::IllegalFilename)?.starts_with("stats")
                        && filename.to_str().ok_or(ReadSaveError::IllegalFilename)?.ends_with(".txt")
                    {
                        let result = gather_stats_from_file(path.into_boxed_path()).map_err(ReadSaveError::Gather)?;
                        
                        println!("{:?}", result);
                        
                        realms.push(result);
                        
                        /*match result {
                            Err(err) => println!("{:?}", err),
                            Ok(realm) => println!("{:?}", realm?)
                        }*/
                    } else {
                        println!("skipping {}", filename.to_str().unwrap());
                    }
                },
                None => println!("{} is not a file?", path.display())
            }
        
        //let filename = rift_wiz_save_folder.join("21/stats.level_25.txt").into_boxed_path();
            
        }
    }
    
    Ok(realms)
}

fn print_merged_runs_info(merged_runs : MergedRuns)
{
    println!("");
    println!("====================");
    println!("MERGED RUNS");
    println!("{:?}", merged_runs);
    
    let total_runs = merged_runs.num_victory + merged_runs.num_defeat + merged_runs.num_abandoned + merged_runs.num_unknown;
    println!("runs: {} won, {} lost, {} abandoned, {} unknown / {} total", merged_runs.num_victory, merged_runs.num_defeat, merged_runs.num_abandoned, merged_runs.num_unknown, total_runs);
    
    {
        let dmg = lazy_init(merged_runs.damage_to_enemies);
    
        println!("====================================");
        println!("DAMAGE TO ENEMIES (TOP 10 SUM OF ALL RUNS)");
        print_top_ten(&dmg);
        
        println!("");
        println!("mean damage per turn: {}", get_mean_per_turn(&dmg, merged_runs.turns_taken));
        println!("");
    }
    
    {
        let dmg = lazy_init(merged_runs.damage_to_wizard);
        println!("===================================");
        println!("DAMAGE TO WIZARD (TOP 10 SUM OF ALL RUNS)");
        print_top_ten(&dmg);
        
        println!("");
        println!("mean damage per turn: {}", get_mean_per_turn(&dmg, merged_runs.turns_taken));
        println!("");
    }
}

fn generate_run_report(mut realms : Vec<Realm>) -> Run
{
    realms.sort_by(|a, b| { a.realm_number.cmp(&b.realm_number) });
    
    let mut run = Run {
        realms : realms.len() as i64,
        outcome : Outcome::Victory,
        ..Default::default() };
    
    
    for realm in realms {
        if run.outcome == Outcome::Victory {
            match realm.outcome {
                Outcome::Unknown => run.outcome = Outcome::Unknown,
                Outcome::Defeat => run.outcome = Outcome::Defeat,
                _ => (),
            }
        }
        run.turns_taken_run = cmp::max(run.turns_taken_run, realm.turns_taken_run);
        run.spell_casts = run.spell_casts.merge_add(realm.spell_casts);
        run.damage_to_enemies = run.damage_to_enemies.merge_add(realm.damage_to_enemies);
        run.damage_to_wizard = run.damage_to_wizard.merge_add(realm.damage_to_wizard);
        run.items_used = run.items_used.merge_add(realm.items_used);
    }
    
    if run.outcome == Outcome::Victory && run.realms < 25
    {
        run.outcome = Outcome::Abandoned;
    }
    
    run
}



fn get_mean_per_turn(map : &HashMap<String, usize>, turns : i64) -> f64
{
    let mut running_sum = 0;
    for (_key, value) in map.iter() {
        running_sum += value;
    }
    
    running_sum as f64 / (turns as f64)
}

fn print_top_ten(map : &HashMap<String, usize>)
{
    let mut v = Vec::with_capacity(map.len());
    
    for (key, _) in map.iter() {
        v.push(key.clone())
    }
    
    v.sort_by(|a, b| { map.get(b).unwrap().cmp(map.get(a).unwrap()) });
    
    for key in v.iter().take(10) {
        println!("{} - {}", key, map.get(key).unwrap());
    }
}
