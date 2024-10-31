//#![allow(warnings)]

use std::{
    collections:: BTreeMap, fmt::Debug, fs::File, io::{BufReader, Read}, ops::{Index, IndexMut}, time
};

fn parse_json(){}

fn main() {
    
    let mut file: BufReader<File> = BufReader::new(File::open("./test.json").expect("open failed"));

    let mut json: JSON;

    let mut file_content_string: String = String::new();

    file.read_to_string(&mut file_content_string).expect("not able to read file");
    file_content_string = String::from(file_content_string.trim());
    let mut file_content: Vec<char> = Vec::new();

    file_content = file_content_string.chars().collect();
    match file_content.last()  {
        Some(value) => {
            if *value != ']'  && *value != '}' {
                panic!("json not in proper format or residual data is present at the end");
            }
        }
        None => {
            panic!("json not in proper format");
        }
    }
    //file.read_vectored(bufs)
    let start_time = time::Instant::now();
    let (i,json) = calculate_vector(0, &file_content);
    let parsing_time_duration = start_time.elapsed();
    let start_time = time::Instant::now();
    let stringified_json = json.to_string();
    let stringify_time_duration = start_time.elapsed();

    println!("parsing time duration = {}", parsing_time_duration.as_micros());
    println!("stringifying time duration = {}", stringify_time_duration.as_micros());

     
}


#[derive(Debug)]
enum JSON {
    Integer(f32),
    String(String),
    Vector(Vec<JSON>),
    Map(MyMap),
    Boolean(bool),
    NULL,
}

enum NULL {}

impl JSON {
    fn to_string(&self) -> String{
        match self {
            JSON::Map(my_map) => {
                return Self::map_to_string(my_map);
            }
            JSON::Vector(vector) => {
                return Self::vector_to_string(vector);
            }
            _=> {
                panic!("not a json object");
            }
        }
    }
    fn map_to_string(my_map: &MyMap) -> String {
        let mut ret_value = String::new();
        ret_value.push('{');
        for (key, value) in my_map {
            ret_value.push_str(&Self::string_to_string(key));
            ret_value.push(':');
            match value {
                JSON::Integer(number) => {
                    ret_value.push_str(&Self::number_to_string(number));
                }
                JSON::String(str) => {
                    ret_value.push_str(&Self::string_to_string(str));
                }
                JSON::Map(value_map) => {
                    ret_value.push_str(&Self::map_to_string(value_map));
                }
                JSON::Vector(vector) => {
                    ret_value.push_str(&Self::vector_to_string(vector));
                }
                JSON::Boolean(boolean) => {
                    ret_value.push_str(&Self::boolean_to_string(boolean));
                }
                JSON::NULL => {
                    ret_value.push_str(&String::from("null"));
                }
            }
            ret_value.push(',');
        }
        ret_value.pop();
        ret_value.push('}');
        return ret_value;
    }

    fn string_to_string(str: &String) -> String {
        let mut ret_value = String::from('\"')+&(*str).clone();
        ret_value.push('\"');
        return ret_value;
    }

    fn number_to_string(number: &f32) -> String {
        return (*number).to_string();
    }

    fn vector_to_string(vector: &Vec<JSON>) -> String {
        let mut ret_value = String::new();
        ret_value.push('[');
        for value in vector {
            
            match value {
                JSON::Integer(number) => {
                    ret_value.push_str(&Self::number_to_string(number));
                }
                JSON::String(str) => {
                    ret_value.push_str(&Self::string_to_string(str));
                }
                JSON::Map(value_map) => {
                    ret_value.push_str(&Self::map_to_string(value_map));
                }
                JSON::Vector(vector) => {
                    ret_value.push_str(&Self::vector_to_string(vector));
                }
                JSON::Boolean(boolean) => {
                    ret_value.push_str(&Self::boolean_to_string(boolean));
                }
                JSON::NULL => {
                    ret_value.push_str(&String::from("null"));
                }
            }

            ret_value.push(',');
        }
        ret_value.pop();
        ret_value.push(']');
        return ret_value;
    }

    fn boolean_to_string(boolean: &bool) -> String {
        if *boolean {
            return String::from("true");
        } else {
            return String::from("false");
        }
    }

}

#[derive(Debug)]
struct MyMap {
    map: BTreeMap<String, JSON>,
    fallback: String,
}

impl MyMap {
    fn new() -> MyMap {
        return MyMap {
            map: BTreeMap::new(),
            fallback: String::from("fallback_string"),
        };
    }
    fn get_mut(&mut self, key: &String) -> &mut Self {
        return match self.map.get_mut(key) {
            Some(_) => self,
            None => {
                self.map
                    .insert(key.clone(), JSON::String(self.fallback.clone()));
                return self;
            }
        };
    }
}

impl Index<String> for MyMap {
    type Output = JSON;
    fn index(&self, key: String) -> &Self::Output {
        match self.map.get(&key) {
            Some(value) => value,
            None => {
                return self.map.get(&key).unwrap();
            }
        }
    }
}

impl IndexMut<String> for MyMap {
    fn index_mut(&mut self, key: String) -> &mut Self::Output {
        self.get_mut(&key).map.get_mut(&key).unwrap()
    }
}
impl<'a> IntoIterator for &'a MyMap {
    type Item = (&'a String, &'a JSON);
    type IntoIter = std::collections::btree_map::Iter<'a, String, JSON>;

    fn into_iter(self) -> Self::IntoIter {
        self.map.iter()
    }
}
impl Index<String> for JSON {
    type Output = JSON;
    fn index(&self, key: String) -> &Self::Output {
        match self {
            JSON::Map(map) => {
                return &map[key];
            }
            _ => {
                panic!("key {} is not an object",key)
            }
        }
    }
}


impl IndexMut<String> for JSON {
    fn index_mut(&mut self, key: String) -> &mut Self::Output {
        match self {
            JSON::Map(map) => {
                return &mut map[key];
            }
            _ => {
                panic!("key {} is not an object",key)
            }
        }
    }
}

impl Index<usize> for JSON {
    type Output = JSON;
    fn index(&self, key: usize) -> &Self::Output {
        match self {
            JSON::Vector(vector) => {
                &vector[key]
            }
            _=>{
                panic!("not a vector");
            }
        }
    }
}
impl IndexMut<usize> for JSON {
    fn index_mut(&mut self, key: usize) -> &mut Self::Output {
        match self {
            JSON::Vector(vector) => {
                &mut vector[key]
            }
            _=>{
                panic!("not a vector");
            }
        }
    }
}




fn calculate_map(mut i: usize, input: &Vec<char>) -> (usize, JSON) {//TODO
    let mut map: MyMap = MyMap::new();
    let mut key: String = String::new();
    let mut value: JSON = JSON::Integer(-1 as f32);

    i += 1;

    let mut has_key = false;
    let mut has_value = false;


    while i<input.len() {
        match input[i] {
            ' '|'\t'|'\n' => {
                i += 1;
            }
            '"' => {
                if has_key && has_value {
                    panic!("multiple values");
                } else if has_key {
                    panic!("multiple keys");
                } else {
                    (i,key) = calculate_key(i+1, input);
                    has_key = true;
                }
            }
            ':' => {
                if !has_key {
                    panic!("no key but specified for value");
                }
                (i,value) = calculate_value(i+1, input);
                has_value = true;
            }
            ',' => {
                if has_key && has_value {
                    map[key] = value;
                    key = String::new();
                    value = JSON::Integer(-1 as f32);
                } else {
                    panic!("no value or no keyHer");
                }

                i += 1;
                has_key = false;
                has_value = false;
            }
            '}' => {
                if has_key && has_value {
                    map[key] = value;
                    return (i+1, JSON::Map(map));
                }
                else if (!has_key) && (!has_value) {
                    return (i+1, JSON::Map(map));
                }
                else {
                    panic!("ending curly braces has problems");
                }
            }
            _=> {
                panic!("unknown character, {}{}{}{}{}{}{}{}{}{}{}{}",input[i-4],input[i-3],input[i-2],input[i-1],input[i],input[i+1],input[i+2],input[i+3],input[i+4],input[i+5],input[i+6],i);
            }
        }
    }
    panic!("error parsing map ending");
}

fn calculate_key(mut i: usize, input : &Vec<char>) -> (usize, String){

    let mut key: String = String::new();

    while i<input.len() {
        if input[i] == '"' {
            return (i+1, key);
        }
        key.push(input[i]);
        i += 1;
    }
     
    panic!("error parsing string string key");
}

fn calculate_string(mut i:usize, input: &Vec<char>) -> (usize, JSON){

    let mut value: String = String::new();
    let ch = '\\';
    let mut is_backslash_on = false;

    while i<input.len() {
        if input[i] == ch {
            is_backslash_on = !is_backslash_on;
            i += 1;
            value.push(input[i]);
        }
        else if input[i] == '"' && !is_backslash_on {
            return (i+1, JSON::String(value));
        }
        else {
            value.push(input[i]);
            is_backslash_on = false;
            i += 1;
        }
    }
    panic!("error parsing string string");
}

fn calculate_boolean(mut i: usize, input: &Vec<char>) -> (usize, JSON){
    let mut boolean: String = String::new();
    while i < input.len() {
        if input[i].is_alphabetic() {
            boolean.push(input[i]);
            i += 1;
        }
        else if input[i] == ',' || input[i] == '\t' || input[i] == '\n' || input[i] == ' ' || input[i] == '}' || input[i] == ']' {
            return (i,JSON::Boolean(boolean.parse().expect("not a boolean")));
        }
        else{
            panic!("unknown character between number");
        }
    }
    panic!("error parsing boolean");
}

fn calculate_number(mut i:usize, input: &Vec<char>) -> (usize, JSON){
    let mut number: String = String::new();
    while i < input.len() {
        if input[i].is_ascii_digit() || input[i] == '.' {
            number.push(input[i]);
            i += 1;
        }
        else if input[i] == ',' || input[i] == '\t' || input[i] == '\n' || input[i] == ' ' || input[i] == '}' || input[i] == ']' {
            return (i,JSON::Integer(number.parse().expect("not a float")));
        }
        else{
            panic!("unknown character between number");
        }
    }
    panic!("error parsing string number");
}

//for just purpose of inclusion
fn calculate_null(mut i:usize, input:&Vec<char>) -> (usize,JSON){ 
    let mut null = String::new();
    while i < input.len() {
        if input[i].is_alphabetic() {
            null.push(input[i]);
            i += 1;
        }
        else if input[i] == ',' || input[i] == '\t' || input[i] == '\n' || input[i] == ' ' || input[i] == '}' || input[i] == ']' {
            if null == String::from("null") {
                return (i,JSON::NULL);
            }
            panic!("not null value");
        }
        else{
            panic!("unknown character between null");
        }
    }
    panic!("error parsing null");

}

fn calculate_vector(mut i:usize, input: &Vec<char>) -> (usize, JSON){

    let mut vector: Vec<JSON> = Vec::new();
    let mut value: JSON = JSON::Integer(-1 as f32);

    i += 1;

    let mut has_value = false;


    while i<input.len() {
        match input[i] {
            '"' => {
                if has_value {
                    panic!("multiple values string");
                } else {
                    (i,value) = calculate_string(i+1, input);
                    has_value = true;
                }
            }
            ',' => {
                if  has_value {
                    vector.push(value);
                    value = JSON::Integer(-1 as f32);
                } else {
                    panic!("no value or no key");
                }
                has_value = false;
                i += 1;
            }
            ']' => {
                if has_value {
                    vector.push(value);
                }
                return (i+1, JSON::Vector(vector));
            }
            't'|'f' => {
                if has_value {
                    (i,value) = calculate_boolean(i, input);
                }
                panic!("key should be wrapped in double quotes");
            }
            '{' => {
                if has_value {
                    panic!("multiple values, object");
                }
                (i,value) = calculate_map(i, input);
                has_value = true;
            }
            ' ' | '\t' | '\n' => {
                i += 1;
            }
            c => {
                if c.is_ascii_digit() {
                    return calculate_number(i, input);
                }
                panic!("unknown character vector, {:#?}{}{}{}{}{}{}",value,input[i-4],input[i-3],input[i-2],input[i-1],c,i);
            }
        }
    }
    panic!("error parsing string array");
}



fn calculate_value(mut i:usize, input: &Vec<char>) -> (usize, JSON) {
    while i<input.len() {
        match input[i] {
            '{' => {
                return calculate_map(i, input);
            },
            '"' => {
                return calculate_string(i+1, input);
            },
            '[' => {
                return  calculate_vector(i, input);
            },
            't'|'f' => {
                return calculate_boolean(i, input);
            }
            'n' => {
                return calculate_null(i, input);
            }
            ' ' | '\t' | '\n' => {
                i += 1;
            }
            c => {
                if c.is_ascii_digit() {
                    return calculate_number(i, input);
                }
                else {
                    panic!("unknown character recognized {}{}",c,i);//TODO boolean data
                }
            }
        }
        
    }
    panic!("error parsing value")
}
/* 

vector c

fn rec(position i, vector) -> (integer, JSON ) {
    while(vec[i] != '{'){
        i++;
    }
    map;
    key;
    open = false;
    while(i<vector.size()){
        if(!open){
            if(vec[i] == '}'){
                return json::new(map);
            }
            if(vec[i] == ':'){
                (i,map[key]) = rec(i+1,vector);
                key = new string;
            }
            if(vec[i] == '"'){
                i++;
                open = true;
                continue;
            }
            i++;
            continue;
        }
        if(open){
            if(vec[i] == '"'){
                open = false;
                i++;
                continue;
            }
            key.push(vector[i]);
            i++;
            continue;
        }
    }
    panic("json not in format");
}

ll reckey(position i, vector){

    while(i<vector.size()){
        
    }

}

ll recvalue(position i, vector){
}



*/





/*

let json = String::from("jj");
let x: BTreeMap<String, String> = BTreeMap::new();
let mut y = JSON::Map(MyMap::new());
y[String::from("randome key")] = JSON::String(String::from("randome value"));
y[String::from("internal object")] = JSON::Map(MyMap::new());
y[5.to_string()] = JSON::Integer(5);
y[String::from("internal object")][String::from("internal key")] = JSON::Vector(vec![JSON::Integer(1), JSON::Integer(2)]);
match &mut y[String::from("internal object")][String::from("internal key")] {
    JSON::Vector(ref mut vector) => {
        vector.push(JSON::String(String::from("value")));
        }
        _ => {}
        } 
        y[String::from("internal object")][String::from("internal key")][2] = JSON::Integer(7);
        println!("Hello, world!,{:#?}\n{:#?}", x, y);
        println!("{:#?}", y[String::from("randome key")]);
        println!("{:#?}", y[String::from("internal object")][String::from("internal key")]);
        println!("{:#?}",json);



*/
