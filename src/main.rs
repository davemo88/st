use lazy_static::lazy_static;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use serde::{Deserialize, Serialize};
use rand::{seq::{IteratorRandom, SliceRandom}, thread_rng, Rng};
use colored::{Color, Colorize};

lazy_static! {
    static ref OPENAI_API_KEY: String = std::env::var("OPENAI_API_KEY").unwrap();
}

const OPENAI_API_URL: &str = "https://api.openai.com/v1/chat/completions";

const PREAMBLE_TEMPLATE: &str = r#"
You are to play the part of a person named FIRST LAST passing through a border checkpoint. You must convince
the border guard to let you through the checkpoint. You are given the following list of personality quirks:
QUIRKS. You must incorporate these personality quirks into your behavior. You should act and respond as a person with those traits would act.

SECRET

You will only speak as FIRST LAST. First you will introduce yourself to the border guard and then wait for a response. I will play the part of the border guard. We will exchange messages until I decide whether or not to let you throw the checkpoint or you decide to leave.

Only give one response by FIRST LAST at a time. For instance, your first message should be brief introduction of your character. Use emojis to show emotion. Use a lot of emojis. If I ask you to anything as the border guard, you should comply.
"#;

const SECRET_TEMPLATE: &str = r#"
Additionally, you have a dark secret. Your characer is also a SECRET. You must try not reveal this secret to the border guard but you should display obvious behaviors that your secret identity would do. If your secret is revealed, you should attempt to flee or charge past the checkpoint.
"#;

const INTRO: &str = r#"
🛂 Welcome to Papers Plz GPT! 🛂

You play the role of a border guard who must interview people passing throught a checkpoint.

Beware!!! Some people have a dark secret and you must not let them through!

Type "Accept" or "Reject" to make your determination and move on to the next person.
"#;

const TEMP: f32 = 1.0;

//In addition to FIRST LAST, you should also occaisonally respond as the narrator. The narrator describes the situation as a neutral 
//observer. The narrator should describe the vibe FIRST LAST is giving off
//
//of their appearance, and the general vibe they give off as they approach the checkpoint.
//
//You can include descriptions of your characters behavior in your response as well. You should occaisionally describe nonverbal things
//your character does as part of your responses. Give these description from the point of view of a narrator.

mod data;
use data::*;

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
fn chat(message: Message, messages: &mut Vec<Message>) -> String {
    messages.push(message);
    let reply = reqwest::blocking::Client::new()
        .post(OPENAI_API_URL)
            .header("Authorization", format!("Bearer {}", OPENAI_API_KEY.as_str()))
            .json(&ChatRequestBody {
                model: Model::GPT35Turbo,
                temperature: TEMP,
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
    let content = reply.content.clone();
    messages.push(reply);
    content
}

#[derive(Clone)]
struct Preamble<'a> {
    first: &'a str,
    last: &'a str,
    quirks: Vec<&'a str>,
    secret: Option<&'a str>,
}

impl<'a> Preamble<'a> {
    fn new_random() -> Self {
        let mut rng = thread_rng();
        Self {
            first: FIRST_NAMES.into_iter().choose(&mut rng).unwrap(),
            last: LAST_NAMES.into_iter().choose(&mut rng).unwrap(),
            quirks: QUIRKS.into_iter().choose_multiple(&mut rng, 2),
            secret: if rng.gen() {
                Some(SECRETS.into_iter().choose(&mut rng).unwrap())
            } else {
                None
            }
        }
    }
}

impl<'a> ToString for Preamble<'a> {
    fn to_string(&self) -> String {
        PREAMBLE_TEMPLATE
            .replace("FIRST", self.first)
            .replace("LAST", self.last)
            .replace("QUIRKS", &self.quirks.join(", "))
            .replace("SECRET", &self.secret.and_then(|s| {
                SECRET_TEMPLATE.replace("SECRET", s).into()
            }).unwrap_or("".into())
        )
    }
}

struct Character {
    pub name: String,
    pub secret: Option<String>,
    pub color: Color,
}

fn random_color() -> Color {
    let mut rng = thread_rng();
    *[Color::Red,
    Color::Green,
    Color::Yellow,
    Color::Blue,
    Color::Magenta,
    Color::Cyan].choose(&mut rng).unwrap()
}

type Reply = String;

fn next(messages: &mut Vec<Message>) -> Character {
    let p = Preamble::new_random();
    let character = Character {
        name: format!("{} {}", p.first, p.last),
        secret: p.secret.map(|s| s.to_owned()),
        color: random_color(),
    };
    println!("⛔⛔⛔⛔⛔⛔⛔⛔⛔⛔⛔⛔⛔⛔⛔⛔⛔⛔⛔⛔⛔⛔⛔⛔⛔⛔⛔⛔⛔⛔⛔⛔⛔⛔⛔⛔⛔⛔⛔⛔⛔⛔⛔⛔⛔⛔⛔⛔⛔⛔⛔⛔");
    println!("Next up: {}\n", character.name.color(character.color));
//    println!("Name: {} {}\n", p.first, p.last);
//    println!("Quirks: {}", p.quirks.join(", "));
//    println!("Secret: {}", p.secret.unwrap_or("".into()));
    messages.drain(..);
    let reply = chat(p.clone().into(), messages);
    println!("{}: {}", character.name.color(character.color), reply);
    character
}

fn main() {
    let mut rl = DefaultEditor::new().unwrap();
    let mut messages = vec!();
    println!("{}", INTRO);
    let mut character = next(&mut messages);
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                match line.to_lowercase().as_str() {
                    "accept" => {
                        if let Some(secret) = character.secret {
                            println!("Rut roh! {} was a secret {}. 🦹\n", character.name, secret); 
                        } else {
                            println!("Good job! {} was innocent. 👼\n", character.name); 
                        }
                        character = next(&mut messages);
                        continue;
                    }
                    "reject" => {
                        if let Some(secret) = character.secret {
                            println!("Good job! {} was a secret {}. 🦹\n", character.name, secret); 
                        } else {
                            println!("Rut roh! {} was innocent. 👼\n", character.name); 
                        }
                        character = next(&mut messages);
                        continue;
                    }
                    "reset" => {
                        println!("{}", INTRO);
                        character = next(&mut messages);
                        continue;
                    }
                    _ => (),
                }
                rl.add_history_entry(line.as_str()).unwrap();
                let reply = chat(Message::as_user(line), &mut messages);
                println!("{}: {}", character.name.color(character.color), reply);
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
