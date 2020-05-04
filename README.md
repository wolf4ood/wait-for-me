<h1 align="center">wait-for-me</h1>
<div align="center">
  <strong>
    Async CountDownLatch
  </strong>
</div>

<br />

<div align="center">
  <a href="https://github.com/wolf4ood/wait-for-me/actions?query=workflow%3ATests">
    <img src="https://github.com/wolf4ood/wait-for-me/workflows/Tests/badge.svg"
    alt="Tests status" />
  </a>
  
  <a href="https://coveralls.io/github/wolf4ood/wait-for-me?branch=master">
    <img src="https://coveralls.io/repos/github/wolf4ood/wait-for-me/badge.svg?branch=master"
    alt="Coverage status" />
  </a>
  <a href="https://crates.io/crates/wait-for-me">
    <img src="https://img.shields.io/crates/d/wait-for-me.svg?style=flat-square"
      alt="Download" />
  </a>
  <a href="https://docs.rs/wait-for-me">
    <img src="https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square"
      alt="docs.rs docs" />
  </a>
  
</div>


# Install


Install from [crates.io](https://crates.io)


```
[dependencies]
wait-for-me = "0.1"
```


# Example


with [smol](https://github.com/stjepang/smol)


```
use wait_for_me::CountDownLatch;
use smol::Task;


#[smol_potat::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let latch = CountDownLatch::new(10);
    for _ in 0..10 {
        let latch1 = latch.clone();
        Task::spawn(async move {
            latch1.count_down().await;
        }).detach();
    }
    latch.wait().await;


    Ok(())
}
```


