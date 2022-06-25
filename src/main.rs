use serde_json::Value;
use std::time::{Instant};
use std::io;
use std::fs::File;
use std::io::{BufReader, BufRead};

#[tokio::main]
async fn main() {
    let model = input_model();
    let start = Instant::now();
    let list = open_list();
    let mut handles = vec![];
    for i in list {
        let m: String = model.clone();
        let handle = tokio::spawn(async move{
            req(i, m).await;
        });
        handles.push(handle);
    }
    for handle in handles{
        handle.await.unwrap();
    }
    let duration = start.elapsed();
    println!("Закончил за: {:?}. Закрой меня", duration);
}
fn input_model() -> String {
    let mut input = String::new();
    println!("Введи код модели: ");
    io::stdin().read_line(&mut input);
    input
}
fn open_list() -> Vec<String> {
    let path = "text.txt";
    let file = match File::open(&path){
        Err(_) => panic!("couldn't open"),
        Ok(file) => file,
    };
    let buffer = BufReader::new(file);
    let list: Vec<String> = buffer.lines().collect::<Result<_, _>>().unwrap();
    list
}


async fn req(i:String, model:String) -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://kkt-online.nalog.ru/lkip.html?query=/fn/model/check&factory_number=".to_owned() + &i + "&model_code=" + &model;
    let resp = reqwest::get(url)
        .await?
        .text()
        .await?;
    let v: Value = match serde_json::from_str(&resp){
        Err(_) => panic!("couldn't parse json"),
        Ok(v) => v,
    };
    println!("{} {}",i, v);
    Ok(())
}