pub fn tokenize(expr: String) -> Vec<String> {
    let replaced = expr
        .replace("\n", "")
        .replace("(", " ( ")
        .replace(")", " ) ");
    replaced.split_whitespace().map(|x| x.to_string()).collect()
}
