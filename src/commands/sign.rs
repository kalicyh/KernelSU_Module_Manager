use owo_colors::OwoColorize;

pub fn execute() {
    println!("{} {}", "📋", "设置签名pem文件".cyan());
    // TODO: 实现sign
}

// $ zakosign key new yourkey.pem
// $ zakosign sign example.zip --key yourkey.pem --output example.signed.zip