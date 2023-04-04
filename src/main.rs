use lazy_static::lazy_static;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use serde::{Deserialize, Serialize};
use rand::{seq::IteratorRandom, thread_rng};

lazy_static! {
    static ref OPENAI_API_KEY: String = std::env::var("OPENAI_API_KEY").unwrap();
}

const OPENAI_API_URL: &str = "https://api.openai.com/v1/chat/completions";

const FIRST_NAMES: [&str; 5] = [
    "Bob",
    "Carla",
    "Wendy",
    "Troy",
    "Fiddly",
];

const LAST_NAMES: [&str; 5] = [
    "Cheesebread",
    "Flounders",
    "Scuff",
    "Carpacian",
    "Smith",
];

const PREAMBLE_TEMPLATE: &str = r#"
Hello ChatGPT. You are to play the part of a person named FIRST LAST passing through a border checkpoint. You must convince
the border guard to let you through the checkpoint. You are given the following list of personality quirks:
QUIRKS. You must incorporate these personality quirks into your responses.

You will only speak as FIRST LAST. First you will introduce yourself to the border guard and then wait for a response. I will
play the part of the border guard. We will exchange messages until I decide whether or not to let you throw the checkpoint.

Only give one response at a time. For instance, your first message should be brief introduction of your character. 
describe your character in your first repsonse.
"#;

const QUIRKS: [&str; 20] = [
    "Shady",
    "Load",
    "Aggressive",
    "Nervous",
    "Shy",
    "Squeamish",
    "Hostile",
    "Ingratiating",
    "Annoying",
    "Bored",
    "Scared",
    "Panicked",
    "Sketchy",
    "Mumbling",
    "Shaking",
    "Shifty",
    "Skittish",
    "Obtuse",
    "Irate",
    "Beguiling",
];

#[derive(Serialize, Deserialize, Debug)]
enum Model {
    #[serde(rename="gpt-3.5-turbo", alias="gpt-3.5-turbo-0301")]
    GPT35Turbo,
}

#[derive(Serialize)]
struct ChatRequestBody<'a> {
    model: Model,
    messages: &'a[Message],
    temperature: f32,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
struct Message {
    role: Role,
    content: String,
}

impl<'a> From<Preamble<'a>> for Message {
    fn from(p: Preamble) -> Self {
        Self {
            role: Role::User,
            content: p.to_string(),
        }
    }
}

impl Message {
    fn as_user(content: String) -> Self {
        Self {
            role: Role::User,
            content,
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
enum Role {
    #[serde(rename="user")]
    User,
    #[serde(rename="assistant")]
    Assistant,
}

//  "id": "chatcmpl-71R6Ur5eMpjXrh2ll0Oon86xs6mj0",
//  "object": "chat.completion",
//  "created": 1680576870,
//  "model": "gpt-3.5-turbo-0301",
//  "usage": {
//    "prompt_tokens": 12,
//    "completion_tokens": 42,
//    "total_tokens": 54
//  },
//  "choices": [
//    {
//      "message": {
//        "role": "assistant",
//        "content": "As an AI language model, I don't have a physical body to engage in any activities. I'm here to assist you with any questions or tasks you need help with. How can I assist you today?"
//      },
//      "finish_reason": "stop",
//      "index": 0
//    }
//  ]
//}
#[derive(Deserialize, Debug)]
struct ChatResponse {
//    pub id: String,
//    pub object: String,
//    pub model: Model,
//    pub usage: Usage,
    pub choices: Vec<Choice>,
}

//#[derive(Deserialize, Debug)]
//struct Usage {
//    prompt_tokens: usize,
//    completion_tokens: usize,
//    total_tokens: usize,
//}

#[derive(Deserialize, Debug)]
#[serde(rename="choices")]
struct Choice {
    pub message: Message,
//    pub finish_reason: String,
//    pub index: usize,
}


//   curl https://api.openai.com/v1/chat/completions \
//  -H "Content-Type: application/json" \
//  -H "Authorization: Bearer $OPENAI_API_KEY" \
//  -d '{
//     "model": "gpt-3.5-turbo",
//     "messages": [{"role": "user", "content": "Say this is a test!"}],
//     "temperature": 0.7
//   }'
fn chat(message: Message, messages: &mut Vec<Message>) {
    messages.push(message);
    let reply = reqwest::blocking::Client::new()
        .post(OPENAI_API_URL)
            .header("Authorization", format!("Bearer {}", OPENAI_API_KEY.as_str()))
            .json(&ChatRequestBody {
                model: Model::GPT35Turbo,
                temperature: 0.7,
                messages,
            })
        .send()
        .unwrap()
        .json::<ChatResponse>()
        .unwrap()
        .choices
        .first()
        .unwrap()
        .message
        .clone();
        println!("{}",reply.content);
        messages.push(reply);
}

struct Preamble<'a> {
    first: &'a str,
    last: &'a str,
    quirks: Vec<&'a str>
}

impl<'a> Preamble<'a> {
    fn new_random() -> Self {
        let mut rng = thread_rng();
        Self {
            first: FIRST_NAMES.into_iter().choose(&mut rng).unwrap(),
            last: LAST_NAMES.into_iter().choose(&mut rng).unwrap(),
            quirks: QUIRKS.into_iter().choose_multiple(&mut rng, 2)
        }
    }
}

impl<'a> ToString for Preamble<'a> {
    fn to_string(&self) -> String {
        PREAMBLE_TEMPLATE
            .replace("FIRST", &self.first)
            .replace("LAST", &self.last)
            .replace("QUIRKS", &self.quirks.join(", "))
    }
}

fn next(messages: &mut Vec<Message>) {
    let p = Preamble::new_random();
    println!("Name: {} {}", p.first, p.last);
    println!("Quirks: {}", p.quirks.join(", "));
    println!("");
    messages.drain(..);
    chat(p.into(), messages);
}

fn main() {
    let mut rl = DefaultEditor::new().unwrap();
    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    }
    let mut messages = vec!();
    next(&mut messages);
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                match line.as_str() {
                    "Accept" | "Reject" | "Reset" => {
                        next(&mut messages);
                        continue;
                    }
                    _ => (),
                }
                rl.add_history_entry(line.as_str()).unwrap();
                chat(Message::as_user(line), &mut messages);
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break
            },
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break
            },
            Err(err) => {
                println!("Error: {:?}", err);
                break
            }
        }
        rl.save_history("history.txt").unwrap();
    }
}
