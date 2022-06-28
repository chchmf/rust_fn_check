
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
    let types = input();
    println!("Введи код модели: ");
    let model = input();
    let start = Instant::now();

    let mut vec = vec![];

    let list = open_list();
//    let mut handles: Vec<tokio::task::JoinHandle<()>> = vec![];

    for i in list {
        let m: String = model.clone();
        let types: i8 = types.clone().trim().parse().unwrap();
        let strng = req(i, m, types);
        vec.push(strng)
        }

    let results = try_join_all(vec).await?;

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
    let mut sheet = wb.create_sheet(&model);
    
    wb.write_sheet(&mut sheet, |sheet_writer| {
        let sw = sheet_writer;
        sw.append_row(row!["SN", "Parse result", "Source result"]);
        for i in results{
            let (one, two, three) = i;
            sw.append_row(row![one, two, three]);
        }
        sw.append_row(row![blank!(3)])
    }).expect("write excel error!");

    wb.close().expect("close excel error!");

    let duration = start.elapsed();
    println!("Закончил за: {:?}. Закрой меня...", duration);
    let mut input = String::new();
    io::stdin().read_line(&mut input);
    Ok(())
}

fn input() -> String {
    let mut input = String::new();
    io::stdin().read_line(&mut input);
    input
}
fn open_list() -> Vec<String> {
    let path = "text.txt";
    let file = File::open(&path).unwrap();
    let buffer = BufReader::new(file);
    let list: Vec<String> = buffer.lines().collect::<Result<_, _>>().unwrap();
    list
}

async fn req(i:String, model:String, types: i8) -> Result<(String, String, String), Box<dyn Error>> {
    let url = if types == 1 {"https://kkt-online.nalog.ru/lkip.html?query=/fn/model/check&factory_number=".to_owned() + &i + "&model_code=" + &model}
    else if types == 2 {"https://kkt-online.nalog.ru/lkip.html?query=/kkt/model/check&factory_number=".to_owned() + &i + "&model_code=" + &model}
    else {"s".to_owned()};
    let resp = reqwest::get(url)
        .await?
        .text()
        .await?;
    let v: Value = serde_json::from_str(&resp).unwrap();
    println!("{}\t{}",i.clone(), v["check_result"]);
    let mut vecc = v.to_string();
    Ok((i, v["check_result"].to_string(), vecc))
}