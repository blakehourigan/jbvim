# jbvim

## Description
A vim inspired editor written in rust.

## Features
- **Modal Editing:** As found in vim, jbvim includes normal, insert, and visual modes. This allows the user to easily navigate a document using vim keybinds in normal mode, insert text in insert mode, and highlight, move, or delete sections of text in visual mode.
- **Gap Buffer:**  A built-from-scratch gap buffer data structure was created to hold the contents of the file and allow for efficient insertion and deletion around the user cursor.


## Software Requirements

- The final version of this software was written and tested only using Arch Linux, the Alacritty terminal environment, and the fish shell. Other software may not play nicely with the **terminol** crate that I developed to interact with the terminal's standard output. 


## Installation
As of this moment, none of these crates have been published to crates.io. This may happen at a later date for supporting crates such as **terminol**. As such, one can install and run this program by following these steps:

1. **Clone this repository to a local directory on your linux machine**
   - As long as you have [git](https://git-scm.com/) installed, this can be done by executing the following in your local terminal:
```sh 
git clone https://github.com/blakehourigan/jbvim.git
```
2. **Ensure that the Rust environment and supporting tools are installed on your machine**
   - It is critical that cargo is installed to ensure that dependencies of the program will be installed at the proper version. Ensure that you have the latest tools installed by visiting Rust's [getting started](https://www.rust-lang.org/learn/get-started) page. The current Rust verion at the time of writing is version 1.84.
3. ** CD into the cloned directory and run the program by executing**
```sh 
cd /dir-you-cloned-to
```
```sh 
cargo run
```



## Usage
- 

## Contributing
Contributions to this project are welcome! Please fork the repository and submit a pull request with your improvements.

## License
This project is licensed under the MIT License.

## Contact
For any queries or suggestions, feel free to reach out to the project maintainer at Houriganb@pm.me.

