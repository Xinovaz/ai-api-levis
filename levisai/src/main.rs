#![allow(warnings)]

use reqwest;
use reqwest::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use serde::{Serialize, Deserialize};
use serde_json;
use std::fs;
use std::io;

const MODEL: &str = "gpt-4";
const SYS: &str = include_str!("sys_cmd.txt");

const API_KEY: &str = "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";

#[derive(Debug, Serialize, Deserialize, Clone)]
struct APIRequest {
    model: String,
    messages: Vec<APIMessage>,
    // functions: None
    // function_call: None,
    temperature: f64,
    top_p: f64,
    n: u8,
    stream: bool,
    stop: Option<String>,
    max_tokens: u64,
    presence_penalty: f64,
    frequency_penalty: f64,
    // logit_bias: None,
    user: String,
}

impl APIRequest {
    fn default(model: String, messages: Vec<APIMessage>) -> APIRequest {
        APIRequest {
            model,
            messages,
            temperature: 0.35,
            top_p: 1.0,
            n: 1,
            stream: false,
            stop: None,
            max_tokens: 1024,
            presence_penalty: 0.0,
            frequency_penalty: 0.0,
            user: "TestingLevisAI".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct APIMessage {
    role: String,
    content: String,
    // name: None,
    // function_call: None
}

impl APIMessage {
    fn new(role: String, content: String) -> APIMessage {
        APIMessage {
            role,
            content,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct APIResponse {
    id: String,
    object: String,
    created: i64,
    choices: Vec<APIResponseChoice>,
    usage: APIResponseUsageStatistics,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct APIResponseChoice {
    index: u8,
    message: APIMessage,
    finish_reason: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct APIResponseUsageStatistics {
    prompt_tokens: u64,
    completion_tokens: u64,
    total_tokens: u64,
}

#[tokio::main]
async fn main() {
    let system_command = SYS.to_string();
    let input = fs::read_to_string("./analyze.txt").unwrap();

    let message = APIMessage::new("user".to_string(), input);

    let sysp = APIMessage::new("system".to_string(), system_command);

    let mut watch_tech: bool = true;

    let client = reqwest::Client::new();

    let mut messages: Vec<APIMessage> = vec![sysp, message];

    println!("[ANALYZING INPUT]");
    loop {
        let params: APIRequest = APIRequest::default(MODEL.to_string(), messages.clone());

        let pstr = serde_json::to_string_pretty(&params).unwrap();

        println!("\n[SENDING DATA]\n");
        let result = client.post("https://api.openai.com/v1/chat/completions")
            .header(ACCEPT, "*/*")
            .header(CONTENT_TYPE, "application/json")
            .header(AUTHORIZATION, format!("Bearer {}", API_KEY))
            .body(pstr)
            .send().await
            .unwrap()
            .text().await.unwrap();

        println!("[RECEIVED DATA]\n");

        if watch_tech {
            println!("########\n\nInformation sent: {:#?}\n\nInformation received: {:#?}\n\n#######\n",
                &params, &result);
        }

        watch_tech = false;

        let preoutput = &serde_json::from_str::<APIResponse>(&result)
            .unwrap()
            .choices.get(0).unwrap().message.clone();

        let output = &mut preoutput.content.clone();

        output.push_str("\n\n↓↓ INPUT BELOW ↓↓");

        println!("{}", output);

        messages.push(preoutput.clone());

        let next_in = line_in();

        messages.push(APIMessage::new("user".to_string(), next_in));
    }
}

fn line_in() -> String {
    let mut content = String::new();
    let stdin = io::stdin();
    let _ = stdin.read_line(&mut content);
    content
}