use std::fs::*;
use crate::*;
use crate::types::*;

#[derive(Debug)]
pub enum GatherError
{
    IO(io::Error),
    ParseInt(std::num::ParseIntError),
    ReachedEOF,
    MissingExpectedPrefix(String),
    LineTooShortToContainExpected(String),
    MissingExpectedPostfix(String),
    MissingExpectedDividerLine,
    NotVictoryOrDefeat,
    SectionEnded
}

#[derive(Debug)]
enum SectionTitle
{
    SpellCasts,
    DamageToEnemies,
    DamageToWizard,
    ItemsUsed,
    Purchases,
    None
}

pub fn gather_stats_from_file(filename : Box<Path>) -> Result<Realm, GatherError>
{
    println!("In file {}", filename.display());

    let mut line_iter = match read_lines(filename) {
            Ok(val) => val,
            Err(err) => return Err(GatherError::IO(err))
        };

    let mut realm = Realm { ..Default::default() };

    realm.realm_number = match expect_prefix_read_int(&mut line_iter, "Realm ") {
        Ok(val) => val,
        Err(err) => return Err(err)
    };
    
    realm.outcome = match read_victory(&mut line_iter) {
        Ok(val) => val,
        Err(err) => return Err(err)
    };
    
    match expect_exact_line(&mut line_iter, "") {
        Ok(_) => (),
        Err(err) => return Err(err)
    };
    
    match expect_exact_line(&mut line_iter, "Turns taken:") {
        Ok(_) => (),
        Err(err) => return Err(err)
    };
    
    realm.turns_taken_realm = match expect_postfix_read_int(&mut line_iter, " (L)") {
        Ok(val) => val,
        Err(err) => return Err(err)
    };
    
    realm.turns_taken_run = match expect_postfix_read_int(&mut line_iter, " (G)") {
        Ok(val) => val,
        Err(err) => return Err(err)
    };
    
    match expect_exact_line(&mut line_iter, "") {
        Ok(_) => (),
        Err(err) => return Err(err)
    };
    
    loop {
        match read_section_title(&mut line_iter) {
            Ok(title) =>
                match title
                {
                    SectionTitle::SpellCasts =>
                        realm.spell_casts = match read_hashmap_prefix_until_empty(&mut line_iter) {
                            Ok(val) => Some(val),
                            Err(err) => return Err(err)
                        },
                    SectionTitle::DamageToEnemies =>
                        realm.damage_to_enemies = match read_hashmap_postfix_until_empty(&mut line_iter) {
                            Ok(val) => Some(val),
                            Err(err) => return Err(err)
                        },
                    SectionTitle::DamageToWizard =>
                        realm.damage_to_wizard = match read_hashmap_postfix_until_empty(&mut line_iter) {
                            Ok(val) => Some(val),
                            Err(err) => return Err(err)
                        },
                    SectionTitle::ItemsUsed =>
                        realm.items_used = match read_hashmap_prefix_until_empty(&mut line_iter) {
                            Ok(val) => Some(val),
                            Err(err) => return Err(err)
                        },
                    SectionTitle::Purchases =>
                        realm.purchases = match read_hashset_until_empty(&mut line_iter) {
                            Ok(val) => Some(val),
                            Err(err) => return Err(err)
                        },
                    SectionTitle::None => return Ok(realm),
                },
            Err(err) => return Err(err)
        }
    }
    // unreachable but left here for posterity
    //Ok(realm)
}

fn read_section_title(
    line_iter : &mut impl Iterator<Item=Result<String, std::io::Error>>)
    -> Result<SectionTitle, GatherError>
{
    match parse(line_iter,
        |line| {
            if line.len() < 1
            {
                return Err(GatherError::LineTooShortToContainExpected(String::from("section title")))
            }
            
            if line.starts_with("Spell Casts:") {
                return Ok(SectionTitle::SpellCasts);
            }
            
            if line.starts_with("Damage to Wizard:") {
                return Ok(SectionTitle::DamageToWizard);
            }
            
            if line.starts_with("Damage to Enemies:") {
                return Ok(SectionTitle::DamageToEnemies);
            }
            
            if line.starts_with("Items Used:") {
                return Ok(SectionTitle::ItemsUsed);
            }
            
            if line.starts_with("Purchases:") {
                return Ok(SectionTitle::Purchases);
            }
            
            Ok(SectionTitle::None)
        }
    ) {
        Ok(val) => Ok(val),
        Err(e) => match e {
            GatherError::SectionEnded | GatherError::ReachedEOF => return Ok(SectionTitle::None),
            _ => return Err(e)
        }
    }
}

fn read_hashset_until_empty(
    line_iter : &mut impl Iterator<Item=Result<String, std::io::Error>>)
    -> Result<HashSet<String>, GatherError>
{
    let mut result = HashSet::new();
    
    loop {
        let line_result = parse(line_iter,
            |line| {
                if line.len() < 1
                {
                    return Err(GatherError::SectionEnded);
                } else {
                    return Ok(line);
                }
            }
        );
        
        match line_result {
            Ok(key) => result.insert(key),
            Err(e) => match e {
                    GatherError::SectionEnded | GatherError::ReachedEOF => return Ok(result),
                    _ => return Err(e) }
        };
    }
}

fn read_hashmap_prefix_until_empty(
    line_iter : &mut impl Iterator<Item=Result<String, std::io::Error>>)
    -> Result<HashMap<String, usize>, GatherError>
{
    let mut result = HashMap::new();
    
    loop {
        let line_result = parse(line_iter,
            |line| {
                let index = match line.find(": ")
                {
                    Some(i) => i,
                    None => return Err(GatherError::SectionEnded)
                };
                
                // there's better ways to do this
                let (_,post) = line.split_at(index + 2);
                let (key,_) = line.split_at(index);
                
                match post.parse::<usize>()
                {
                    Ok(val) => Ok((String::from(key), val)),
                    Err(err) => Err(GatherError::ParseInt(err))
                }
            }
        );
        
        match line_result {
            Ok((key, value)) => result.insert(key, value),
            Err(e) => match e {
                        GatherError::SectionEnded | GatherError::ReachedEOF => return Ok(result),
                        _ => return Err(e) }
        };
    }
}

fn read_hashmap_postfix_until_empty(
    line_iter : &mut impl Iterator<Item=Result<String, std::io::Error>>)
    -> Result<HashMap<String, usize>, GatherError>
{
    let mut result = HashMap::new();
    
    loop {
        let line_result = parse(line_iter,
            |line| {
                let index = match line.find(' ')
                {
                    Some(i) => i,
                    None => return Err(GatherError::SectionEnded)
                };
                
                
                // there's better ways to do this
                let (_,key) = line.split_at(index + 1);
                let (pre,_) = line.split_at(index);
                
                match pre.parse::<usize>()
                {
                    Ok(val) => Ok((String::from(key), val)),
                    Err(err) => Err(GatherError::ParseInt(err))
                }
            }
        );
        
        match line_result {
            Ok((key, value)) => result.insert(key, value),
            Err(e) => match e {
                        GatherError::SectionEnded | GatherError::ReachedEOF => return Ok(result),
                        _ => return Err(e) }
        };
    }
}

fn expect_exact_line(
    line_iter : &mut impl Iterator<Item=Result<String, std::io::Error>>,
    expected : &str)
    -> Result<(), GatherError>
{
    parse(line_iter,
        |line| {
            if expected.len() != line.len()
            {
                return Err(GatherError::LineTooShortToContainExpected(expected.to_string()))
            }
            
            if !line.starts_with(expected) {
                return Err(GatherError::MissingExpectedDividerLine);
            }
            
            Ok(())
        }
    )
}

fn expect_prefix_read_int(
    line_iter : &mut impl Iterator<Item=Result<String, std::io::Error>>,
    prefix : &str)
    -> Result<i64, GatherError>
{
    parse(line_iter,
        |line| {
            if !line.starts_with(prefix) {
                return Err(GatherError::MissingExpectedPrefix(prefix.to_string()));
            }
            
            let (_,post) = line.split_at(prefix.len());
            
            
            match post.parse::<i64>()
            {
                Ok(val) => Ok(val),
                Err(err) => Err(GatherError::ParseInt(err))
            }
        }
    )
}

fn expect_postfix_read_int(
    line_iter : &mut impl Iterator<Item=Result<String, std::io::Error>>,
    postfix : &str)
    -> Result<i64, GatherError>
{
    parse(line_iter,
        |line| {
            let len = line.len();
            let post_len = postfix.len();
            
            if len < post_len
            {
                return Err(GatherError::LineTooShortToContainExpected(postfix.to_string()))
            }
            
            
            let (pre,post) = line.split_at(len - post_len);
            
            if !post.starts_with(postfix) {
                return Err(GatherError::MissingExpectedPostfix(postfix.to_string()));
            }
            
            match pre.parse::<i64>()
            {
                Ok(val) => Ok(val),
                Err(err) => Err(GatherError::ParseInt(err))
            }
        }
    )
}

fn read_victory(
    line_iter : &mut impl Iterator<Item=Result<String, std::io::Error>>)
    -> Result<Outcome, GatherError>
{
    let parser_fn = |line : String| {
            let prefix = "Outcome: ";
            
            if !line.starts_with(prefix) {
                return Err(GatherError::MissingExpectedPrefix(prefix.to_string()));
            }
            
            let (_,post) = line.split_at(prefix.len());
            
            if post.starts_with("DEFEAT")
            {
                return Ok(Outcome::Defeat);
            }
            
            if post.starts_with("VICTORY")
            {
                return Ok(Outcome::Victory);
            }
            
            return Err(GatherError::NotVictoryOrDefeat);
        };
    
    
    // if the prefix is missing, it might've been a challenge mode or a weekly, so we try twice in that case
    match parse(line_iter, parser_fn) {
        Err(GatherError::MissingExpectedPrefix(_)) => parse(line_iter, parser_fn),
        other => other
    }
}

fn parse<T, F>(
    line_iter : &mut impl Iterator<Item=Result<String, std::io::Error>>,
    f : F)
    -> Result<T, GatherError>
    where F : FnOnce(String) -> Result<T, GatherError>
{
    let line = match line_iter.next() {
        None => return Err(GatherError::ReachedEOF),
        Some(val) => match val {
            Ok(inner) => inner,
            Err(err) => return Err(GatherError::IO(err))
        }
    };
    
    f(line)
}


fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}
