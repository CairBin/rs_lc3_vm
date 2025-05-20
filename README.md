# rs_lc3_vm

## 描述

基于Rust实现的LC3虚拟机，支持跨平台。

依赖仅有[Crossterm](https://docs.rs/crossterm/latest/crossterm/)

## 使用方式

开发环境下可以在cargo后面跟命令行参数，正式环境编译后同理
```sh
cargo run [img-files] ...
```

## 其他

项目参考自[github.com/mhashim6/LC3-Virtual-Machine](https://github.com/mhashim6/LC3-Virtual-Machine)以及文章[Write your Own Virtual Machine](https://justinmeiners.github.io/lc3-vm/)