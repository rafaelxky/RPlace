#[derive(Debug,Clone)]
pub struct VarOption{
    pub option: String,
    pub args: Vec<String>,
}
impl VarOption {
    pub fn new(option: String, args: Vec<String>) -> Self { 
        Self { option, args }
    }
    pub fn push_arg(&mut self, arg: String){ 
        self.args.push(arg);
    }
}
#[derive(Debug, Clone)]
pub enum Condition {
    EQUALS,
}
impl Condition {
    pub fn eval(&self, first: &str, sec: &str) -> bool {
        match self {
            Condition::EQUALS => {
                return first == sec;
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum ValueType {
    Literal,
    Var,
}
#[derive(Debug, Clone)]
pub struct Value {
    pub value_type: ValueType,
    pub value: String,
    pub options: Option<Vec<VarOption>>,
}
#[derive(Debug, Clone)]
pub struct TemplateValue {
    pub value: String,
    pub options: Option<Vec<VarOption>>,
}
impl ToString for Value {
    fn to_string(&self) -> String {
        self.value.to_string()
    }
}

#[derive(Debug, Clone)]
pub enum Node {
    // - def template_1
    DEF {
        conditions: Option<Vec<(String, String, Condition)>>,
        defaults: Option<Vec<(String, String)>>,
        name: String,
        body: Box<Node>,
        line: usize,
    },
    // either data or var ($a)
    BODY {
        data: Vec<Node>,
        line: usize,
    },
    // def body
    DATA {
        data: String,
        line: usize,
    },
    // def variables
    VARTEMPLATE {
        val: TemplateValue,
    },
    RARROWVAR {
        name: String,
        default: Option<String>,
    },
    PLACE {
        name: String,
        args: Vec<(String, Value)>,
        line: usize,
    },
    INCLUDE {
        path: String,
        line: usize,
    },
    CREATE {
        path: String,
        content: Option<Box<Node>>,
    },
    DERIVE {
        path: String,
        val: Vec<(String, Value)>,
    },
    MATCH  {
        line: usize,
        var_name: String,
        val: Vec<MatchArm>,
    }
}
impl Node {
    pub fn new_create(path: String, content: Vec<Node>, starting_line: usize) -> Node {
        let body = Node::BODY { data: content, line: starting_line };
        let body = Some(Box::new(body));
        Node::CREATE { content: body, path }
    }
    pub fn var_template<T:ToString>(name: T, options: Option<Vec<VarOption>>) -> Self{
        Self::VARTEMPLATE { val: TemplateValue { value: name.to_string(), options: options } }
    }
}
#[derive(Debug, Clone)]
pub struct MatchArm{
    pub match_value: String,
    pub body: Node,
}
impl MatchArm {
    pub fn new(match_value: String, body: Node)-> Self{
        Self { match_value, body: body }
    }
    pub fn matches(&self, val: String) -> bool {
        self.match_value == *val
    }
}

#[derive(Debug, Clone)]
pub struct ParsingError {
    error_msg: String,
}
impl ParsingError {
    pub fn new(error_msg: String) -> Self{
        Self { error_msg }
    }
    pub fn print_err(&self){
        println!("{}",self.error_msg);
    }
}
#[derive(Debug, Clone)]
pub struct ParsingResult {
    pub nodes: Vec<Node>,
    pub file_path: String,
    pub errors: Vec<ParsingError>,
}
impl ParsingResult {
    pub fn new<T:ToString>(file_path: T) -> Self {
        Self {
            nodes: vec![],
            file_path: file_path.to_string(),
            errors: vec![],
        }
    }
    pub fn new_full(file_path: String, nodes: Vec<Node>, errors: Vec<ParsingError>) -> Self{
        Self { nodes, file_path, errors }
    }
    pub fn push_error(&mut self) {}
    pub fn push(&mut self, node: Node) {
        self.nodes.push(node);
    }
    pub fn set_nodes(&mut self, nodes: Vec<Node>) {
        self.nodes = nodes;
    }
    pub fn remove(&mut self, i: usize) -> Node{
        self.nodes.remove(i)
    }
    pub fn len(&self) -> usize{
        self.nodes.len()
    }
    pub fn extend_errors(&mut self, res: &ParsingResult){
        self.errors.extend(res.errors.clone());
    }
    pub fn new_error(&mut self, err: String){
        self.errors.push(ParsingError::new(err));
    }
}