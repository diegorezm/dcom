//! Dados de exemplo do spike da fase 0. Saem quando o cliente assinar as queries do Convex.

use eframe::egui::Color32;

pub struct Person {
    pub name: &'static str,
    pub color: Color32,
}

pub struct Message {
    pub author: usize,
    pub body: String,
    pub time: String,
}

pub struct Conversation {
    pub name: String,
    pub is_channel: bool,
    pub color: Color32,
    pub time: &'static str,
    pub unread: u32,
    pub subtitle: String,
    pub messages: Vec<Message>,
}

pub struct Space {
    pub name: &'static str,
    pub color: Color32,
    pub unread: u32,
    pub conversations: Vec<Conversation>,
}

pub struct Data {
    pub people: Vec<Person>,
    pub spaces: Vec<Space>,
}

pub const ME: usize = 0;

const LINES: [&str; 8] = [
    "Alguém já testou a versão nova?",
    "Testei agora de manhã e abriu bem rápido, nem deu tempo de pegar o café.",
    "@Bia consegue revisar o resumo antes da reunião de quinta?",
    "Claro! Só me manda o link que eu olho hoje à noite.",
    "Fechado 👍",
    "Pensei em dividir o trabalho em três partes: primeiro a gente fecha o texto, depois separa as imagens e no fim cada um revisa a parte do outro para ninguém ficar sobrecarregado.",
    "Boa ideia, @Caio. Eu fico com as imagens.",
    "Vamos começar às 19h, chega quem puder 🎉",
];

fn messages(count: usize, people: usize) -> Vec<Message> {
    let mut author = 1;
    (0..count)
        .map(|i| {
            // Mantém alguns autores seguidos para exercitar o agrupamento.
            if i % 3 != 1 {
                author = 1 + (i * 7 + i / 5) % (people - 1);
            }
            let minutes = i * 3 % (24 * 60);
            Message {
                author,
                body: LINES[(i * 5 + i / 7) % LINES.len()].to_owned(),
                time: format!("{:02}:{:02}", minutes / 60, minutes % 60),
            }
        })
        .collect()
}

fn channel(
    name: &str,
    color: Color32,
    time: &'static str,
    unread: u32,
    count: usize,
) -> Conversation {
    Conversation {
        name: name.to_owned(),
        is_channel: true,
        color,
        time,
        unread,
        subtitle: "45 membros, 24 online".to_owned(),
        messages: messages(count, 6),
    }
}

fn dm(person: &Person, time: &'static str, unread: u32) -> Conversation {
    Conversation {
        name: person.name.to_owned(),
        is_channel: false,
        color: person.color,
        time,
        unread,
        subtitle: "online".to_owned(),
        messages: messages(30, 6),
    }
}

pub fn data() -> Data {
    let blue = Color32::from_rgb(0xB5, 0xCC, 0xFA);
    let peach = Color32::from_rgb(0xFA, 0xC6, 0x9E);
    let green = Color32::from_rgb(0x5D, 0xAE, 0x6B);
    let yellow = Color32::from_rgb(0xF7, 0xDD, 0x93);
    let red = Color32::from_rgb(0xC9, 0x4F, 0x48);
    let lilac = Color32::from_rgb(0xC9, 0xB6, 0xF5);

    let people = vec![
        Person {
            name: "Você",
            color: lilac,
        },
        Person {
            name: "Ana",
            color: peach,
        },
        Person {
            name: "Bia",
            color: green,
        },
        Person {
            name: "Caio",
            color: blue,
        },
        Person {
            name: "Duda",
            color: yellow,
        },
        Person {
            name: "Enzo",
            color: red,
        },
    ];
    let spaces = vec![
        Space {
            name: "Conversas",
            color: lilac,
            unread: 2,
            conversations: vec![
                dm(&people[1], "4 min", 2),
                dm(&people[2], "15 min", 0),
                dm(&people[3], "9:31", 0),
                dm(&people[4], "ontem", 0),
            ],
        },
        Space {
            name: "Estudos",
            color: blue,
            unread: 7,
            conversations: vec![
                channel("geral", blue, "4 min", 5, 10_000),
                channel("resumos", peach, "15 min", 2, 200),
                channel("provas", green, "9:31", 0, 200),
                channel("avisos", yellow, "ontem", 0, 40),
                channel("off-topic", red, "ontem", 0, 200),
            ],
        },
        Space {
            name: "Amigos",
            color: peach,
            unread: 0,
            conversations: vec![
                channel("geral", green, "ontem", 0, 200),
                channel("jogos", lilac, "ontem", 0, 200),
            ],
        },
        Space {
            name: "Trabalho",
            color: green,
            unread: 0,
            conversations: vec![channel("geral", yellow, "ontem", 0, 200)],
        },
    ];
    Data { people, spaces }
}
