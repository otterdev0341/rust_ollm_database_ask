use rust_db_talk::{tools::text_to_chain::TextToSqlChain, trait_req_impl::chain::Chain};
use std::io::{self, Write};


#[tokio::main]
async fn main() {
  let processor = TextToSqlChain::initialize().await.unwrap();
  let mut user_input = String::new();
  print!("How Can I help you?");
  io::stdout().flush().unwrap();
  io::stdin().read_line(&mut user_input).expect("Failed to read line");
  let output = processor.run(user_input)
    .await
    .unwrap();

  println!("{}",output);

}
