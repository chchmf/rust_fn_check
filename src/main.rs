use serde_json::Value;
use std::time::{Instant};
use std::{io, vec};
use std::fs::File;
use std::io::{BufReader, BufRead};
use std::{error::Error};
use futures::future::try_join_all;
use simple_excel_writer::*;


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>>{
    println!("1 - проверяем ФН\n2 - проверяем ККТ");
    let types: i8 = input()
        .trim()
        .parse()
        .unwrap();
    let model: String;
    if types == 2{
        println!("Введи код модели: ");
        model = input();
    }
    else {
        model = String::from("fn");
    }

    let start = Instant::now();

    let mut vec = vec![];

    let list = open_list();

//    let mut handles: Vec<tokio::task::JoinHandle<()>> = vec![];

    for i in list.into_iter() {
        let m: String = model.clone();
        let types: i8 = types.clone();
        let strng = req(i.replace("\"", ""), m, types);
        vec.push(strng)
        }

    let results = try_join_all(vec).await.unwrap();

//    for i in list {
//        let m: String = model.clone();
//        let types: i8 = types.clone().trim().parse().unwrap();
//        let handle = tokio::spawn(async move{
//            req(i, m, types).await.unwrap();
//        });
//        handles.push(handle);
//    }

//    for handle in handles{
//        handle.await.unwrap();
//    }

    let mut wb = Workbook::create("result.xlsx");
    let mut sheet = wb.create_sheet(
        if types == 1{"fn"}
        else {&model}
    );
    
    wb.write_sheet(&mut sheet, |sheet_writer| {
        let sw = sheet_writer;
        sw.append_row(row!["SN", "Parse result", "Source result"]).unwrap();
        for i in results{
            let (one, two, three) = i;
            sw.append_row(row![one, two, three]).unwrap();
        }
        sw.append_row(row![blank!(3)])
    }).expect("write excel error!");

    wb.close().expect("close excel error!");

    let duration = start.elapsed();
    println!("Закончил за: {:?}. Закрой меня...", duration);
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .unwrap();
    Ok(())
}

fn input() -> String {
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input).unwrap();
    input
}
fn open_list() -> Vec<String> {
    let path = "text.txt";
    let file = File::open(&path).unwrap();
    let buffer = BufReader::new(file);
    let list: Vec<String> = buffer
        .lines().collect::<Result<_, _>>().unwrap();
    list
}

async fn req(i:String, mut model: String, types: i8) -> Result<(String, String, String), (String, String, String)> {
    let url = if types == 1 {
        let fn15m = String::from("996044");
        let fn36m = String::from("996144");
        if &i[..6] == fn15m{model = String::from("0021")}
        else if &i[..6] == fn36m{model = String::from("0022")}
        else{}
        "https://kkt-online.nalog.ru/lkip.html?query=/fn/model/check&factory_number=".to_owned() + &i + "&model_code=" + &model
    }
    else if types == 2 {
        "https://kkt-online.nalog.ru/lkip.html?query=/kkt/model/check&factory_number=".to_owned() + &i + "&model_code=" + &model
    }
    else {
        "0".to_owned()
    };
    let req = match reqwest::get(url).await{
        Ok(o) => o,
        Err(_) => return Ok((i ,String::from("Что-то пошло не так"), String::from("Что-то пошло не так"))),
    };
    let resp = match req
        .text()
        .await{
        Ok(o) => o,
        Err(_) => String::from("Не могу распарсить json"),
    };
    let v: Value = serde_json::from_str(&resp).unwrap();
    let mut results = v["check_result"]
        .to_string();
    if v["check_status"] == 1{
        results = String::from("Готова к работе")
    }
    println!("{}\t{}",i.clone(), results);
    Ok((i, results, v.to_string()))
}