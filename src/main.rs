use std::str::Chars;
type Nodes = Vec<Vec<Node>>;
type Edges = Vec<Edge>;

fn main() {
    let mut edges: Vec<Edge> = Vec::new();
    let mut nodes: Vec<Vec<Node>> = Vec::new();
    for _ in 0..26 {
        let mut col = Vec::new();
        for _ in 0..26 {
            col.push(Node::new());
        }
        nodes.push(col);
    }
    update_node((0, 0), String::from("1"), &mut nodes, &mut edges);
    update_node((0, 1), String::from("2"), &mut nodes, &mut edges);
    update_node((0, 2), String::from("3"), &mut nodes, &mut edges);
    update_node((0, 3), String::from("4"), &mut nodes, &mut edges);
    update_node((0, 4), String::from("5"), &mut nodes, &mut edges);
    update_node((1, 0), String::from("=AVG(A1:A5)"), &mut nodes, &mut edges);
    update_node((1, 1), String::from("B1"), &mut nodes, &mut edges);
    println!("{edges:?}");
}

fn update_node(location: (usize, usize), mut input: String, nodes: &mut Nodes, edges: &mut Edges) {
    edges.retain(|x| x.to != location);
    let mut val = 0;
    if let Ok(num) = input.parse::<u64>() {
        val = num;
    } else if input.starts_with("=") {
        input.remove(0);
        let mut chunk = String::new();
        let mut chars = input.chars();
        while let Some(ch) = chars.next() {
            if ch == '(' {
                break;
            }
            chunk.push(ch);
        }
        let operation = Operation::from_str(chunk.clone()).unwrap();
        let func = Function::new(operation);

        let (value, _) =
            calculate_function(chars.as_str().to_string(), func, nodes, edges, location);
        val = value;
        println!("Computed Value: {value}");
    }
    let node = nodes
        .get_mut(location.0)
        .unwrap()
        .get_mut(location.1)
        .unwrap();
    node.input = input.clone();
    node.total = val;
    let dep = edges
        .iter_mut()
        .filter(|x| x.from == location)
        .map(|x| x.clone())
        .collect::<Vec<Edge>>();
    let mut thing = Vec::new();
    for each in dep {
        thing.push(each.clone());
        update_node(
            each.to,
            nodes
                .get(each.to.0)
                .unwrap()
                .get(each.to.1)
                .unwrap()
                .input
                .clone(),
            nodes,
            edges,
        )
    }
}

fn calculate_function(
    instring: String,
    mut func: Function,
    nodes: &mut Nodes,
    edges: &mut Edges,
    location: (usize, usize),
) -> (u64, String) {
    let mut input = instring.chars();
    let item_breaks = ['(', ')', ':', ','];
    let mut chunk = String::new();
    let mut start = String::new();
    let mut charstr = String::new();
    'outer: while let Some(ch) = input.next() {
        if item_breaks.contains(&ch) {
            match ch {
                '(' => {
                    let operation = Operation::from_str(chunk.clone()).unwrap();
                    let new_func = Function::new(operation);
                    let (total, leftovers) = calculate_function(
                        input.as_str().to_string(),
                        new_func,
                        nodes,
                        edges,
                        location,
                    );
                    charstr = leftovers;
                    input = charstr.chars();
                    func.items.push(total);
                }
                ':' => {
                    start = chunk.clone();
                }
                _ => {
                    if start.len() > 0 {
                        let mut loc_nums = calculate_range(start.chars(), chunk.chars());
                        for each in loc_nums.iter() {
                            if edges.contains(&Edge {
                                from: location,
                                to: *each,
                            }) {
                                println!("Circular dependency found!");
                                return (0, "".into());
                            };
                            edges.push(Edge {
                                from: *each,
                                to: location,
                            });
                        }
                        let mut nums = loc_nums
                            .iter_mut()
                            .map(|x| nodes.get(x.0).unwrap().get(x.1).unwrap().total)
                            .collect::<Vec<u64>>();
                        func.items.append(&mut nums);
                        start = "".into();
                    } else if let Ok(num) = chunk.parse::<u64>() {
                        func.items.push(num);
                    } else if is_cell(chunk.clone().chars()) {
                        let location = calculate_position(chunk.chars());
                        func.items.push(
                            nodes
                                .get(location.0)
                                .unwrap()
                                .get(location.1)
                                .unwrap()
                                .total,
                        );
                    }
                    if ch == ')' {
                        break 'outer;
                    }
                }
            }
            chunk = "".into();
        } else {
            chunk.push(ch);
        }
    }
    (func.total(), input.as_str().to_string())
}

fn calculate_range(start: Chars, end: Chars) -> Vec<(usize, usize)> {
    let (x_start, y_start) = calculate_position(start);
    let (x_end, y_end) = calculate_position(end);
    let mut locations = Vec::new();
    for x in x_start..=x_end {
        for y in y_start..=y_end {
            locations.push((x, y));
        }
    }
    locations
}

fn calculate_position(mut input: Chars) -> (usize, usize) {
    let alpha = ('A'..='Z').collect::<Vec<char>>();
    let mut x_raw = String::new();
    let mut y_raw = String::new();
    while let Some(ch) = input.next() {
        if !ch.is_numeric() {
            x_raw.push(ch);
        } else {
            y_raw.push(ch);
        }
    }
    let x = alpha.iter().position(|c| c.to_string() == x_raw).unwrap();
    let y = y_raw.parse::<usize>().unwrap() - 1;
    (x, y)
}

fn is_cell(mut ch: Chars) -> bool {
    if ch.any(|x| x.is_numeric()) && ch.any(|x| x.is_ascii_alphabetic()) {
        return true;
    }
    false
}

#[derive(Debug)]
struct Node {
    input: String,
    total: u64,
}

impl Node {
    fn new() -> Self {
        Self {
            input: "".into(),
            total: 0,
        }
    }
}

struct Location {
    x: usize,
    y: usize,
}

struct Function {
    op: Operation,
    items: Vec<u64>,
}

impl Function {
    fn new(op: Operation) -> Self {
        Self {
            op,
            items: Vec::new(),
        }
    }
    fn total(&self) -> u64 {
        match self.op {
            Operation::Sum => self.items.iter().sum(),
            Operation::Average => {
                let top = self.items.iter().sum::<u64>();
                return top / self.items.len() as u64;
            }
            _ => 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Edge {
    from: (usize, usize),
    to: (usize, usize),
}

#[derive(Debug, Clone)]
enum Operation {
    Sum,
    Average,
    Count,
    Max,
    Min,
    If,
    Vlookup,
    Concat,
    Substitute,
}

impl Operation {
    fn from_str(fname: String) -> Result<Operation, String> {
        match fname.as_str() {
            "SUM" => Ok(Operation::Sum),
            "AVG" => Ok(Operation::Average),
            _ => Err(String::from("Invalid Function Type")),
        }
    }
}
