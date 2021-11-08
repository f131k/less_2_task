use std::io;
use regex::Regex;
use less_2_task::{Stack, Queue};

// Типы доступных токенов (лексем)
#[derive(Debug, Copy, Clone, PartialEq)]
enum TokenType {
    NumberInt,
    NumberFloat,
    UnaryOperator,
    BinaryOperator,
    Function,
    OpenedParenthesis,
    ClosedParenthesis,
    ArgumentSeparator,
    Whitespaces,
}

// Определим кортеж для удобства работы - (Тип токена, "символьное представление")
type Token = (TokenType, String);

// Ассоциативность оператора
#[derive(Clone, Copy, PartialEq)]
enum OperatorAssociation {
    LeftAssociation,
    RightAssociatoin,
}

// Псевдоним для наглядности
type OperatorOrder = u8;

// Определим тип для определения действий
type Operator<'a>= (&'a str, OperatorOrder, OperatorAssociation);

// Список известных (поддерживаемых операторов)
static KNOWNS_OPERATORS: &'static [Operator] = &[
    ("POS", 1, OperatorAssociation::RightAssociatoin),
    ("NEG", 1, OperatorAssociation::RightAssociatoin),
    ("/", 2, OperatorAssociation::LeftAssociation),
    ("*", 2, OperatorAssociation::LeftAssociation),
    ("%", 2, OperatorAssociation::LeftAssociation),
    ("+", 3, OperatorAssociation::LeftAssociation),
    ("-", 3, OperatorAssociation::LeftAssociation),
    ("<<", 4, OperatorAssociation::LeftAssociation),
    (">>", 4, OperatorAssociation::LeftAssociation),
];

// Список известных токенов и соответствующих им шаблонов поиска в исходной строке
static KNOWNS_TOKENS: &'static [(TokenType, &str)] = &[
    (TokenType::OpenedParenthesis, r"^(\()"),
    (TokenType::ClosedParenthesis, r"^(\))"),
    (TokenType::Function, r"^[a-zA-Z]+"),
    (TokenType::BinaryOperator, r"^([\+\-/\*]{1,1})|(<{2,2})|(>{2,2})"),
    (TokenType::UnaryOperator, r"^([\+\-]{1,1})"),
    (TokenType::NumberFloat, r"^(\d+\.\d+)"),
    (TokenType::NumberInt, r"^(\d+)"),
    (TokenType::ArgumentSeparator, r"^(,{1,1})"),
    (TokenType::Whitespaces, r"^(\s+)"),
];

// Получаем информацию об операторе из таблицы
fn get_op_info(op: &str) -> Option<(OperatorOrder, OperatorAssociation)> {
    for operator in KNOWNS_OPERATORS {
        if op == operator.0 {
            return Some((operator.1, operator.2));
        }
    }

    None
}

// Определяем, нужно ли выталкивать из стека имеющийся там оператор
fn need_op_pop_from_stack(op1: &str, op2: &str) -> bool {
    let (op1_prio, op1_associo) = get_op_info(op1).unwrap();
    let (op2_prio, _) = get_op_info(op2).unwrap();
    // Если приоритет op2 выше или равен приоритету op1 и при этом op1 является левоассоциативным
    if op2_prio < op1_prio ||
        (op2_prio == op1_prio && op1_associo == OperatorAssociation::LeftAssociation) {
            return true;
        }

    false
}

// Разбиваем входную строку на токены (лексемы)
fn tokerize(in_string: &str) -> Result<Vec<Token>, char> {
    let permissible_tokens = [TokenType::NumberFloat, TokenType::NumberInt, TokenType::ClosedParenthesis];
    let mut tokens : Vec<Token> = Vec::new();
    let mut target_string = in_string;
    let mut error : bool = false;
    while !target_string.is_empty() && !error {
        let strlen_before = target_string.len();
        for tok in KNOWNS_TOKENS {
            let rgx : Regex = Regex::new(tok.1).unwrap();
            match rgx.captures(target_string) {
                None => continue,
                Some(captions) => {
                    let mut value = &captions[0];
                    // Т.к. унарные + и - не отличимы при разборе от бинарных, то
                    //  необходимы дополнительные проверки:
                    //  если есть последний разобранный токен и он число или закрывающая скобка, то
                    //  данный токен это унарный оператор, иначе - бинарный
                    if tok.0 == TokenType::BinaryOperator {
                        let last = tokens.last();
                        if last == None || !permissible_tokens.contains(&last.unwrap().0) {
                            continue;
                        }
                    } else if tok.0 == TokenType::UnaryOperator {
                        // Дополнительно, чтобы при вычислении выражения отличать бинарные + и -
                        // от унарных переименуем унарные в соответствующие операторы
                        value = match value {
                            "+" => "POS",
                            "-" => "NEG",
                            _ => "",
                        };
                    }
                    tokens.push((tok.0, value.to_string()));
                    target_string = target_string.strip_prefix(&captions[0]).unwrap();
                }
            }
        }
        error = strlen_before == target_string.len();
    }

    if error {
        return Err(target_string.chars().next().unwrap());
    }

    Ok(tokens)
}

// Выполняем преобразования списка входных токенов в запись ОПН согласно алгоритму
// сортировочной станции Дейкстры
fn convert_to_rpn<'a>(token_list: Vec<Token>) -> Result<Queue<Token>, &'a str> {
    let mut output: Queue<Token> = Queue::new();
    let mut stack: Stack<Token> = Stack::new();
    for tok in token_list {
        match tok.0 {
            TokenType::NumberInt | TokenType::NumberFloat => {
                // Если токен — число, то добавить его в очередь вывода
                output.enqueue(tok);
            },
            TokenType::Function => {
                // Если токен — функция, то поместить его в стек
                stack.push(tok);
            },
            TokenType::ArgumentSeparator => {
                // Если токен — разделитель аргументов функции (например запятая):
                //     Пока токен на вершине стека не открывающая скобка:
                //         Переложить оператор из стека в выходную очередь.
                while !stack.is_empty() && stack.peek().unwrap().0 != TokenType::OpenedParenthesis {
                    let op = stack.pop().unwrap();
                    output.enqueue(op);
                }
                // Если стек закончился до того, как был встречен токен открывающая скобка,
                //   то в выражении пропущен разделитель аргументов функции (запятая),
                //   либо пропущена открывающая скобка.
                if stack.is_empty() {
                    return Err("в выражении пропущен разделитель аргументов функции (запятая), либо пропущена открывающая скобка");
                }
            },
            TokenType::BinaryOperator | TokenType::UnaryOperator => {
                // Если токен — оператор op1, то:
                //     Пока присутствует на вершине стека токен оператор op2,
                //       чей приоритет выше или равен приоритету op1,
                //       и при равенстве приоритетов op1 является левоассоциативным:
                //         Переложить op2 из стека в выходную очередь;
                let mut last = stack.peek();
                while last != None &&
                    (last.unwrap().0 == TokenType::BinaryOperator) &&
                    need_op_pop_from_stack(&tok.1, &last.unwrap().1) {
                        let op = stack.pop().unwrap();
                        output.enqueue(op);
                        last = stack.peek();
                    }
                // Положить op1 в стек.
                stack.push(tok);
            },
            TokenType::OpenedParenthesis => {
                // Если токен — открывающая скобка, то положить его в стек
                stack.push(tok);
            },
            TokenType::ClosedParenthesis => {
                // Если токен — закрывающая скобка:
                //     Пока токен на вершине стека не открывающая скобка
                //         Переложить оператор из стека в выходную очередь.
                while !stack.is_empty() && stack.peek().unwrap().0 != TokenType::OpenedParenthesis {
                    let op = stack.pop().unwrap();
                    output.enqueue(op);
                }

                // Если стек закончился до того, как был встречен токен открывающая скобка, то в выражении пропущена скобка.
                if stack.is_empty() {
                    return Err("в выражении пропущена скобка");
                } else {
                    // Выкинуть открывающую скобку из стека, но не добавлять в очередь вывода.
                    let _ = stack.pop();
                    // Если токен на вершине стека — функция, переложить её в выходную очередь.
                    if !stack.is_empty() && stack.peek().unwrap().0 == TokenType::Function {
                        let op = stack.pop().unwrap();
                        output.enqueue(op);
                    }
                }
            },
            TokenType::Whitespaces => println!("А вот такого быть не должно"),
        }
    }

    // Если больше не осталось токенов на входе:
    // Пока есть токены операторы в стеке:
    let mut last = stack.peek();
    while last != None {
        // Если токен оператор на вершине стека — открывающая скобка, то в выражении пропущена скобка.
        if last.unwrap().0 == TokenType::OpenedParenthesis {
            return Err("в выражении пропущена скобка");
        }

        // Переложить оператор из стека в выходную очередь.
        let op = stack.pop().unwrap();
        output.enqueue(op);
        last = stack.peek();
    }

    Ok(output)
}


// Вычисление известных бинарных операторов
fn calc_binary_operator(op: &str, arg1: &Token, arg2: &Token) -> String {
    let arg1 = arg1.1.parse::<f32>().unwrap();
    let arg2 = arg2.1.parse::<f32>().unwrap();
    match op {
        "+" => return format!("{0:.2}", arg1 + arg2),
        "-" => return format!("{0:.2}", arg1 - arg2),
        "/" => return format!("{0:.2}", arg1 / arg2),
        "*" => return format!("{0:.2}", arg1 * arg2),
        "<<" => return format!("{0:.2}", ((arg1 as i32) << (arg2 as i32)) as f32),
        ">>" => return format!("{0:.2}", ((arg1 as i32) >> (arg2 as i32)) as f32),
        _ => "".to_string(),
    }
}

// Вычисление известных унарных операторов
fn calc_unary_operator(op: &str, arg: &Token) -> String {
    let arg = arg.1.parse::<f32>().unwrap();
    match op {
        "POS" => return format!("{0:.2}", arg),
        "NEG" => return format!("{0:.2}", -1.0 * arg),
        _ => "".to_string(),
    }
}

// Вычисление выражения и вывод на консоль и самого выражения, и результата
fn calc_and_print<'a>(mut output: Queue<Token>) -> Result<String, &'a str> {
    let mut calculate_stack : Stack<Token> = Stack::new();
    while !output.is_empty() {
        let out = output.dequeue();
        print!("{} ", out.1);
        match out.0 {
            TokenType::NumberFloat | TokenType::NumberInt => {calculate_stack.push(out);},
            TokenType::BinaryOperator => {
                if let Some(arg2) = calculate_stack.pop() {
                    if let Some(arg1) = calculate_stack.pop() {
                        let res = calc_binary_operator(&out.1, &arg1, &arg2);
                        calculate_stack.push((TokenType::NumberFloat, res));
                        continue;
                    }
                }
                return Err("Выходная очередь сформирована неправильно");
            },
            TokenType::UnaryOperator => {
                if let Some(arg) = calculate_stack.pop() {
                    let res = calc_unary_operator(&out.1, &arg);
                    calculate_stack.push((TokenType::NumberFloat, res));
                    continue;
                }
                return Err("Выходная очередь сформирована неправильно");
            },
            TokenType::Function => {
            },
            _ => {
                return Err("Выходная очередь сформирована неправильно");
            },
        }
    }

    if calculate_stack.is_empty() {
        return Err("");
    }

    let result = calculate_stack.pop().unwrap();
    if !calculate_stack.is_empty() ||
        result.0 != TokenType::NumberFloat {
            return Err("");
        }

    let result = result.1;
    Ok(result)
}

// Процесс преобразования состоит из 3 основных этапов
fn process(input : &String) -> Result<String,String> {
    // для унификации удалим все пробелы из строки
    let trimmed = &input.trim().replace(" ", "");
    // 1. Разбиваем входную строку на токены (лексемы)
    let tokens = match tokerize(trimmed) {
        Ok(tokens) => tokens,
        Err(why) => return Err(format!(" {1:>0$} неизвестная лексема!", input.find(why).unwrap(), "^")),
    };

    // 2. Преобразуем список входных токенов в список в ОПН
    let output = match convert_to_rpn(tokens) {
        Ok(output) => output,
        Err(why) => return Err(format!("\r{}", why)),
    };

    // 3. Вычисляем результат выражения
    let result = match calc_and_print(output) {
        Ok(result) => format!("\nРезультат: {}", result),
        Err(why) => return Err(format!("\r{}", why)),
    };

    Ok(result)
}

fn main() {
    print_help();
    loop {
        let stdin = io::stdin();
        let mut input = String::new();
        println!("Введите выражение:");
        stdin.read_line(&mut input).expect("Не удалось прочитать строку");
        match process(&input) {
            Ok(result) => println!("{}", result),
            Err(why) => println!("{}", why),
        };

        match request_to_continue() {
            true => continue,
            false => break,
        }
    }
}

fn print_help() {
    println!("Данная программа преобразует арифметическую операцию записанную в инфиксной форме в запись обратной польской нотации и вычисляет её.\nПоддерживаемые операции:");
    println!("  унарные:");
    println!("    '+'");
    println!("    '-'");
    println!("  бинарные:");
    println!("    '+'");
    println!("    '-'");
    println!("    '/'");
    println!("    '*'");
    println!("Для выхода нажмите <Ctrl+C>");
}

fn request_to_continue() -> bool {
    let mut answer = String::new();
    println!("Продолжить (Д/н)");
    io::stdin().read_line(&mut answer).expect("Не удалось прочитать строку");
    match answer.trim() {
        "y" | "Y" | "Д" | "д" => {return true},
        "n" | "N" | "Н" | "н" => {return false},
        _ => {
            println!("Некорректный ввод. Закрываемся..");
        },
    }

    false
}
